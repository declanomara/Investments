use std::fmt;

use serde::{Serialize, Deserialize};

// Enum to represent the two types of messages (with `method` or `channel`)
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum KrakenMessage {
    MethodResponse(MethodResponse), // For messages with `method`
    ChannelMessage(ChannelMessage), // For messages with `channel`
}

// Struct for method-based response (like a subscribe response)
#[derive(Debug, Serialize, Deserialize)]
pub struct MethodResponse {
    pub method: String,
    pub result: Option<MethodResult>,
    pub success: bool,
    #[serde(rename = "time_in")]
    pub time_in: String,
    #[serde(rename = "time_out")]
    pub time_out: String,
}

// Struct to represent the "result" field in method responses
#[derive(Debug, Serialize, Deserialize)]
pub struct MethodResult {
    pub channel: Option<String>,
    pub event_trigger: Option<String>,
    pub snapshot: Option<bool>,
    pub symbol: Option<String>,
}

// Struct for channel-based messages (like heartbeat, ticker, and status)
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "channel")]
pub enum ChannelMessage {
    #[serde(rename = "heartbeat")]
    Heartbeat(Heartbeat),
    #[serde(rename = "ticker")]
    Ticker(Ticker),
    #[serde(rename = "status")]
    Status(Status),
    #[serde(rename = "book")]
    OrderBook(OrderBook),
}

// Struct for the heartbeat message
#[derive(Debug, Serialize, Deserialize)]
pub struct Heartbeat;

// Struct for the ticker message
#[derive(Debug, Serialize, Deserialize)]
pub struct Ticker {
    #[serde(rename = "type")]
    pub message_type: String, // e.g., "snapshot"
    pub data: Vec<TickerData>,
}

// Struct to represent the ticker data
#[derive(Debug, Serialize, Deserialize)]
pub struct TickerData {
    pub symbol: String,
    pub bid: f64,
    pub bid_qty: f64,
    pub ask: f64,
    pub ask_qty: f64,
    pub last: f64,
    pub volume: f64,
    pub vwap: f64,
    pub low: f64,
    pub high: f64,
    pub change: f64,
    #[serde(rename = "change_pct")]
    pub change_pct: f64,
}

// Struct for the status message
#[derive(Debug, Serialize, Deserialize)]
pub struct Status {
    #[serde(rename = "type")]
    pub message_type: String, // e.g., "update"
    pub data: Vec<StatusData>,
}

// Struct to represent the data in the status message
#[derive(Debug, Serialize, Deserialize)]
pub struct StatusData {
    pub api_version: String,
    pub connection_id: u64,
    pub system: String,
    pub version: String,
}

// Struct for the order book message
#[derive(Debug, Serialize, Deserialize)]
pub struct OrderBook {
    #[serde(rename = "type")]
    pub message_type: String, // e.g., "snapshot"
    pub data: Vec<OrderBookData>,
}

// Struct to represent the data in the order book message
#[derive(Debug, Serialize, Deserialize)]
pub struct OrderBookData {
    pub symbol: String,
    pub checksum: u64,
    pub bids: Vec<OrderBookLevel>,
    pub asks: Vec<OrderBookLevel>,
}

// Struct to represent a level in the order book
#[derive(Debug, Serialize, Deserialize)]
pub struct OrderBookLevel {
    pub price: f64,
    pub qty: f64,
}

impl fmt::Debug for KrakenMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            KrakenMessage::MethodResponse(method_response) => write!(f, "{:?}", method_response),
            KrakenMessage::ChannelMessage(channel_message) => write!(f, "{:?}", channel_message),
        }
    }
}