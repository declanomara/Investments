use crate::models::TradingSignal;
use crate::oanda::objects::Price;

pub trait AlphaModel {
    fn tick(&mut self, price: &Price) -> Result<Option<TradingSignal>, Box<dyn std::error::Error>>;
}

pub struct ExponentialMovingAverage {
    instrument: String,

    slow_ma_weight: f32,
    fast_ma_weight: f32,

    slow_ma: f32,
    fast_ma: f32,
}

impl ExponentialMovingAverage {
    pub fn new(instrument: String, slow_ma_weight: f32, fast_ma_weight: f32) -> Self {
        ExponentialMovingAverage {
            instrument,
            slow_ma_weight,
            fast_ma_weight,
            slow_ma: -1.0,
            fast_ma: -1.0,
        }
    }
}

impl AlphaModel for ExponentialMovingAverage {
    fn tick(&mut self, price: &Price) -> Result<Option<TradingSignal>, Box<dyn std::error::Error>> {
        let mut signal = None;

        // If we don't have a slow or fast moving average yet, set them to the current price
        if self.slow_ma < 0.0 || self.fast_ma < 0.0 {
            self.fast_ma = price.ask;
            self.slow_ma = price.ask;
            return Ok(None);
        }

        // Calculate the new moving averages
        let new_slow_ma =
            self.slow_ma_weight * price.ask + (1.0 - self.slow_ma_weight) * self.slow_ma;
        let new_fast_ma =
            self.fast_ma_weight * price.ask + (1.0 - self.fast_ma_weight) * self.fast_ma;

        // If the fast moving average crosses above the slow moving average, buy
        if new_fast_ma > new_slow_ma && self.fast_ma < self.slow_ma {
            signal = Some(TradingSignal {
                instrument: self.instrument.clone(),
                forecast: 1.0,
            });
        } else if new_fast_ma < new_slow_ma && self.fast_ma > self.slow_ma {
            signal = Some(TradingSignal {
                instrument: self.instrument.clone(),
                forecast: -1.0,
            });
        }

        self.slow_ma = new_slow_ma;
        self.fast_ma = new_fast_ma;
        Ok(signal)
    }
}

pub struct SimpleMovingAverage {
    instrument: String,
    period: usize,
    prices: Vec<f32>,
}

impl SimpleMovingAverage {
    pub fn new(instrument: String, period: usize) -> Self {
        SimpleMovingAverage {
            instrument,
            period,
            prices: Vec::new(),
        }
    }
}

impl AlphaModel for SimpleMovingAverage {
    fn tick(&mut self, price: &Price) -> Result<Option<TradingSignal>, Box<dyn std::error::Error>> {
        let signal;

        // Add the current price to the list of prices
        self.prices.push(price.ask);

        // If we don't have enough prices yet, return None
        if self.prices.len() < self.period {
            return Ok(None);
        }

        // If we have too many prices, remove the oldest price
        if self.prices.len() > self.period {
            self.prices.remove(0);
        }

        // Calculate the average of the prices
        let mut sum = 0.0;
        for price in &self.prices {
            sum += price;
        }
        let average = sum / self.prices.len() as f32;

        // If the current price is above the average, buy
        if price.ask > average {
            signal = Some(TradingSignal {
                instrument: self.instrument.clone(),
                forecast: 1.0,
            });
        } else {
            signal = Some(TradingSignal {
                instrument: self.instrument.clone(),
                forecast: -1.0,
            });
        }

        Ok(signal)
    }
}

// A simple weighted consensus model that takes the average of all the signals of a collection of models
pub struct WeightedConsensus {
    models: Vec<Box<dyn AlphaModel>>,
    weights: Vec<f64>,
}

impl WeightedConsensus {
    pub fn new() -> Self {
        WeightedConsensus {
            models: Vec::new(),
            weights: Vec::new(),
        }
    }

    pub fn add_model(mut self, model: Box<dyn AlphaModel>, weight: f64) -> Self {
        self.models.push(model);
        self.weights.push(weight);
        self
    }
}

impl AlphaModel for WeightedConsensus {
    fn tick(&mut self, price: &Price) -> Result<Option<TradingSignal>, Box<dyn std::error::Error>> {
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
