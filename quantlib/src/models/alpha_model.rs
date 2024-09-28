use rand::Rng;

use crate::oanda::objects::Price;
use crate::{models::TradingSignal, util::TradingConfig};
use serde::{Deserialize, Serialize};

pub trait AlphaModel {
    fn tick(&mut self, price: &Price) -> Result<Option<TradingSignal>, Box<dyn std::error::Error>>;
    fn from_config(config: &TradingConfig) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized;
}

pub enum AlphaModels {
    Random(RandomStrategy),
    ExponentialMovingAverage(ExponentialMovingAverage),
}

impl AlphaModel for AlphaModels {
    fn tick(&mut self, price: &Price) -> Result<Option<TradingSignal>, Box<dyn std::error::Error>> {
        match self {
            AlphaModels::Random(strategy) => strategy.tick(price),
            AlphaModels::ExponentialMovingAverage(strategy) => strategy.tick(price),
        }
    }

    fn from_config(config: &TradingConfig) -> Result<Self, Box<dyn std::error::Error>> {
        println!("{:?}", config);
        match config.model.as_str() {
            "random" => {
                let rng = rand::thread_rng();
                let strategy = RandomStrategy {
                    buy_threshold: config.model_config["buyThreshold"].as_f64().unwrap(),
                    sell_threshold: config.model_config["sellThreshold"].as_f64().unwrap(),
                    rng,
                };
                Ok(AlphaModels::Random(strategy))
            }
            "ema" => {
                let slow_ma_weight = config.model_config["slowWeight"].as_f64().unwrap();
                let fast_ma_weight = config.model_config["fastWeight"].as_f64().unwrap();
                let strategy = ExponentialMovingAverage::new(slow_ma_weight, fast_ma_weight);
                Ok(AlphaModels::ExponentialMovingAverage(strategy))
            }
            _ => panic!("Unknown model: {}", config.model),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExponentialMovingAverage {
    #[serde(rename = "slowWeight")]
    slow_ma_weight: f64,
    #[serde(rename = "fastWeight")]
    fast_ma_weight: f64,

    #[serde(skip)]
    slow_ma: f64,
    #[serde(skip)]
    fast_ma: f64,
}

impl ExponentialMovingAverage {
    pub fn new(slow_ma_weight: f64, fast_ma_weight: f64) -> Self {
        ExponentialMovingAverage {
            slow_ma_weight,
            fast_ma_weight,
            slow_ma: -1.0,
            fast_ma: -1.0,
        }
    }

    pub fn tick(
        &mut self,
        price: &Price,
    ) -> Result<Option<TradingSignal>, Box<dyn std::error::Error>> {
        let mut signal = None;

        // If we don't have a slow or fast moving average yet, set them to the current price
        if self.slow_ma < 0.0 || self.fast_ma < 0.0 {
            self.fast_ma = price.ask as f64;
            self.slow_ma = price.ask as f64;
            return Ok(None);
        }

        // Calculate the new moving averages
        let new_slow_ma =
            self.slow_ma_weight * price.ask as f64 + (1.0 - self.slow_ma_weight) * self.slow_ma;
        let new_fast_ma =
            self.fast_ma_weight * price.ask as f64 + (1.0 - self.fast_ma_weight) * self.fast_ma;

        // If the fast moving average crosses above the slow moving average, buy
        if new_fast_ma > new_slow_ma && self.fast_ma < self.slow_ma {
            signal = Some(TradingSignal {
                instrument: price.instrument.clone(),
                forecast: 1.0,
            });
        } else if new_fast_ma < new_slow_ma && self.fast_ma > self.slow_ma {
            signal = Some(TradingSignal {
                instrument: price.instrument.clone(),
                forecast: -1.0,
            });
        }

        self.slow_ma = new_slow_ma;
        self.fast_ma = new_fast_ma;
        Ok(signal)
    }
}

// A simple weighted consensus model that takes the average of all the signals of a collection of models
pub struct WeightedConsensus {
    models: Vec<AlphaModels>,
    weights: Vec<f64>,
}

impl WeightedConsensus {
    pub fn new() -> Self {
        WeightedConsensus {
            models: Vec::new(),
            weights: Vec::new(),
        }
    }

    pub fn add_model(mut self, model: AlphaModels, weight: f64) -> Self {
        self.models.push(model);
        self.weights.push(weight);
        self
    }

    // TODO: Remove the clone() calls; there must be a better way (should instrument be included in the TradingSignal? Maybe whatever is calling the model should know the instruments since models are agnostic to instruments?)
    pub fn tick(
        &mut self,
        price: &Price,
    ) -> Result<Option<TradingSignal>, Box<dyn std::error::Error>> {
        // Iterate over all the models and get their signals
        let mut signals = Vec::new();
        for model in &mut self.models {
            let model_signal = model.tick(price)?;
            if let Some(model_signal) = model_signal {
                signals.push(model_signal);
            } else {
                signals.push(TradingSignal {
                    instrument: price.instrument.clone(),
                    forecast: 0.0,
                });
            }
        }

        // Calculate the weighted average of the signals
        let mut sum = 0.0;
        for i in 0..signals.len() {
            sum += signals[i].forecast * self.weights[i];
        }
        let average = sum / self.weights.iter().sum::<f64>();

        // Return the average signal, if it's not zero
        if average != 0.0 {
            return Ok(Some(TradingSignal {
                instrument: price.instrument.clone(),
                forecast: average,
            }));
        } else {
            return Ok(None);
        }
    }
}

// A simple random strategy that randomly buys or sells based on configurable thresholds
#[derive(Serialize, Deserialize, Debug)]
pub struct RandomStrategy {
    #[serde(rename = "buyThreshold")]
    pub buy_threshold: f64,
    #[serde(rename = "sellThreshold")]
    pub sell_threshold: f64,
    #[serde(skip)]
    pub rng: rand::rngs::ThreadRng,
}

impl RandomStrategy {
    pub fn tick(
        &mut self,
        price: &Price,
    ) -> Result<Option<TradingSignal>, Box<dyn std::error::Error>> {
        let signal;
        let random: f64 = self.rng.gen();

        if random < self.buy_threshold {
            signal = Some(TradingSignal {
                instrument: price.instrument.clone(),
                forecast: 1.0,
            });
        } else if random > self.sell_threshold {
            signal = Some(TradingSignal {
                instrument: price.instrument.clone(),
                forecast: -1.0,
            });
        } else {
            signal = None;
        }

        Ok(signal)
    }
}
