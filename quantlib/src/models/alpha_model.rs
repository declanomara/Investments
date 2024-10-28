use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use crate::oanda::objects::Price;

pub struct TradingSignal {
    pub amount: f64,
}

pub trait AlphaModel {
    fn tick(&mut self, price: &Price) -> Option<TradingSignal>;
    fn to_vec(&self) -> Vec<f64>;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Random {
    pub buy_threshold: f64,
    pub sell_threshold: f64,
    #[serde(skip)]
    pub rng: rand::rngs::ThreadRng,
}

impl AlphaModel for Random {
    fn tick(&mut self, _price: &Price) -> Option<TradingSignal> {
        let signal = self.rng.gen_range(0.0..1.0);
        if signal > self.buy_threshold {
            Some(TradingSignal { amount: 1.0 })
        } else if signal < self.sell_threshold {
            Some(TradingSignal { amount: -1.0 })
        } else {
            None
        }
    }

    fn to_vec(&self) -> Vec<f64> {
        vec![self.buy_threshold, self.sell_threshold]
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub struct DiamondHands;

impl AlphaModel for DiamondHands {
    fn tick(&mut self, _price: &Price) -> Option<TradingSignal> {
        Some(TradingSignal { amount: 1.0 })
    }

    fn to_vec(&self) -> Vec<f64> {
        vec![]
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub model_name: String,
    pub instruments: Vec<String>,
    pub parameters: HashMap<String, serde_json::Value>,
}

pub fn read_config(config_path: &str) -> Config {
    let mut file = File::open(config_path).expect("Unable to open config file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read config file");

    serde_json::from_str(&contents).expect("Unable to parse config file")
}

pub fn create_model_from_config(config: &Config) -> Box<dyn AlphaModel> {
    match config.model_name.as_str() {
        "random" => {
            let buy_threshold = config
                .parameters
                .get("buy_threshold")
                .and_then(|v| v.as_f64())
                .expect("buy_threshold is required for Random model");
            let sell_threshold = config
                .parameters
                .get("sell_threshold")
                .and_then(|v| v.as_f64())
                .expect("sell_threshold is required for Random model");

            Box::new(Random {
                buy_threshold,
                sell_threshold,
                rng: thread_rng(),
            })
        }
        "diamond_hands" => Box::new(DiamondHands),
        _ => panic!("Unknown model type"),
    }
}
