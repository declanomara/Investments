// This file contains the definitions for the generic objects like OrderBook, Ticker, etc.

use rust_decimal::Decimal;
use std::collections::BTreeMap;
use chrono::{DateTime, Utc};
use crate::kraken::objects::{BookMessage, OrderLevel};


pub struct OrderBook {
    pub symbol: String,
    pub bids: BTreeMap<Decimal, Decimal>,
    pub asks: BTreeMap<Decimal, Decimal>,
    pub last_update: Option<DateTime<Utc>>,
}

impl OrderBook {
    pub fn new(symbol: &str) -> Self {
        Self { symbol: symbol.to_string(), bids: BTreeMap::new(), asks: BTreeMap::new(), last_update: None }
    }
    
    fn update_levels(&mut self, bids: &[OrderLevel], asks: &[OrderLevel]) {
        // Remove zero quantity levels
        for level in bids {
            if level.qty > Decimal::ZERO {
                self.bids.insert(level.price, level.qty);
            } else {
                self.bids.remove(&level.price);
            }
        }
        for level in asks {
            if level.qty > Decimal::ZERO {
                self.asks.insert(level.price, level.qty);
            } else {
                self.asks.remove(&level.price);
            }
        }
    }
    
    pub fn update(&mut self, book: BookMessage) {
        match book {
            BookMessage::Snapshot(snapshot) => {
                self.bids.clear();
                self.asks.clear();
                self.last_update = Some(Utc::now());
                for data in snapshot.data {
                    if data.symbol == self.symbol {
                        self.update_levels(&data.bids, &data.asks);
                    }
                }
            }
            BookMessage::Update(update) => {
                for data in update.data {
                    if data.symbol == self.symbol {
                        self.update_levels(&data.bids, &data.asks);
                    }
                }
            }
        }
    }
}