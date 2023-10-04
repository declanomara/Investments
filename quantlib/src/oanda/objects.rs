use serde::Deserialize;

use crate::oanda::helpers::{deserialize_f32_from_string, deserialize_time_in_millis_from_string, deserialize_f64_from_string};

pub const STREAMING_URL: &str = "https://stream-fxpractice.oanda.com";
pub const API_URL: &str = "https://api-fxpractice.oanda.com";

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub instruments: Vec<String>,
    pub units: f64,
    pub oanda: OandaSettings,
}

#[derive(Debug, Deserialize)]
pub struct OandaSettings {
    pub account_id: String,
    pub authorization: String,
}

#[derive(Debug, Deserialize)]
pub struct Response {
    pub prices: Vec<Price>,
}

#[derive(Debug, Deserialize)]
pub struct Price {
    #[serde(deserialize_with = "deserialize_f32_from_string")]
    #[serde(rename = "closeoutBid")]
    pub bid: f32,

    #[serde(deserialize_with = "deserialize_f32_from_string")]
    #[serde(rename = "closeoutAsk")]
    pub ask: f32,

    #[serde(deserialize_with = "deserialize_time_in_millis_from_string")]
    pub time: u64,
    pub instrument: String,
}

#[derive(Debug, Deserialize)]
pub struct Heartbeat {
    pub time: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum StreamItem {
    Price(Price),
    Heartbeat(Heartbeat),
}

#[derive(Debug, Deserialize)]
pub struct PositionResponse {
    pub positions: Vec<Position>,
}

#[derive(Debug, Deserialize)]
pub struct Position {
    pub instrument: String,
    pub long: PositionDetails,
    pub short: PositionDetails,
}

#[derive(Debug, Deserialize)]
pub struct PositionDetails {
    #[serde(deserialize_with = "deserialize_f64_from_string")]
    pub units: f64,
    #[serde(deserialize_with = "deserialize_f64_from_string")]
    #[serde(rename = "unrealizedPL")]
    pub unrealized_pl: f64,
}

impl Position {
    pub fn units(&self) -> f64 {
        let net = self.long.units + self.short.units;
        println!(
            "Net units: {}+{}={}",
            self.long.units, self.short.units, net
        );
        net
    }

    pub fn unrealized_pl(&self) -> f64 {
        self.long.unrealized_pl + self.short.unrealized_pl
    }
}