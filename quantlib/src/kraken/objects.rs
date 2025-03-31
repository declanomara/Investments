use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use serde::de::IntoDeserializer;
use rust_decimal::Decimal;
#[allow(unused_imports)]
use rust_decimal_macros::dec;

#[derive(Debug, PartialEq)]
pub enum Message {
    Heartbeat(HeartbeatMessage),
    Status(StatusMessage),
    MethodRequest(MethodRequest),
    MethodResponse(MethodResponse),
    ChannelMessage(ChannelMessage),
}

impl Serialize for Message {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Message::Heartbeat(msg) => msg.serialize(serializer),
            Message::Status(msg) => msg.serialize(serializer),
            Message::MethodRequest(msg) => msg.serialize(serializer),
            Message::MethodResponse(msg) => msg.serialize(serializer),
            Message::ChannelMessage(msg) => msg.serialize(serializer),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct HeartbeatMessage {
    pub channel: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct StatusMessage {
    pub channel: String,
    pub data: Vec<StatusData>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct StatusData {
    pub version: String,
    pub system: String,
    pub api_version: String,
    pub connection_id: u64,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct MethodRequest {
    pub method: String,
    pub params: MethodParams,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct MethodParams {
    pub channel: String,
    pub symbol: Vec<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct MethodResponse {
    pub method: String,
    pub result: MethodResult,
    pub success: bool,
    pub time_in: String,
    pub time_out: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct MethodResult {
    pub channel: String,
    pub event_trigger: Option<String>,
    pub snapshot: bool,
    pub symbol: String,
}

#[derive(Debug, PartialEq)]
pub enum ChannelMessage {
    Ticker(TickerMessage),
    Book(BookMessage),
}

impl Serialize for ChannelMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ChannelMessage::Ticker(msg) => msg.serialize(serializer),
            ChannelMessage::Book(msg) => msg.serialize(serializer),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TickerMessage {
    pub channel: String,
    pub data: Vec<TickerData>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TickerData {
    pub symbol: String,
    pub bid: Decimal,
    pub bid_qty: Decimal,
    pub ask: Decimal,
    pub ask_qty: Decimal,
    pub last: Decimal,
    pub volume: Decimal,
    pub vwap: Decimal,
    pub low: Decimal,
    pub high: Decimal,
    pub change: Decimal,
    pub change_pct: Decimal,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum BookMessage {
    Snapshot(BookSnapshot),
    Update(BookUpdate),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct BookSnapshot {
    pub channel: String,
    #[serde(rename = "type")]
    pub message_type: String,
    pub data: Vec<BookData>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct BookUpdate {
    pub channel: String,
    #[serde(rename = "type")]
    pub message_type: String,
    pub data: Vec<BookData>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct BookData {
    pub symbol: String,
    pub bids: Vec<OrderLevel>,
    pub asks: Vec<OrderLevel>,
    pub checksum: u64,
    pub timestamp: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct OrderLevel {
    pub price: Decimal,
    pub qty: Decimal,
}

impl<'de> Deserialize<'de> for Message {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        
        // First handle all method-related messages
        if let Some(method) = value.get("method") {
            // Method response has both method and result
            if value.get("result").is_some() {
                return MethodResponse::deserialize(value.into_deserializer())
                    .map(Message::MethodResponse)
                    .map_err(serde::de::Error::custom);
            }
            
            // Method request has method and params
            if value.get("params").is_some() {
                if method == "subscribe" {
                    return MethodRequest::deserialize(value.into_deserializer())
                        .map(Message::MethodRequest)
                        .map_err(serde::de::Error::custom);
                }
            }
            
            // If we have a method field but don't match any method message type,
            // return an error immediately
            return Err(serde::de::Error::custom("Invalid method message structure"));
        }

        // Then handle channel messages - only if we don't have a method field
        if let Some(channel) = value.get("channel") {
            match channel.as_str() {
                Some("heartbeat") => {
                    HeartbeatMessage::deserialize(value.into_deserializer())
                        .map(Message::Heartbeat)
                        .map_err(serde::de::Error::custom)
                }
                Some("status") => {
                    StatusMessage::deserialize(value.into_deserializer())
                        .map(Message::Status)
                        .map_err(serde::de::Error::custom)
                }
                Some("book") => {
                    // For book messages, we need to check the type field
                    if let Some(msg_type) = value.get("type") {
                        match msg_type.as_str() {
                            Some("snapshot") => {
                                BookSnapshot::deserialize(value.into_deserializer())
                                    .map(|msg| Message::ChannelMessage(ChannelMessage::Book(BookMessage::Snapshot(msg))))
                                    .map_err(serde::de::Error::custom)
                            }
                            Some("update") => {
                                BookUpdate::deserialize(value.into_deserializer())
                                    .map(|msg| Message::ChannelMessage(ChannelMessage::Book(BookMessage::Update(msg))))
                                    .map_err(serde::de::Error::custom)
                            }
                            _ => Err(serde::de::Error::custom("Invalid book message type")),
                        }
                    } else {
                        Err(serde::de::Error::missing_field("type"))
                    }
                }
                Some("ticker") => {
                    TickerMessage::deserialize(value.into_deserializer())
                        .map(|msg| Message::ChannelMessage(ChannelMessage::Ticker(msg)))
                        .map_err(serde::de::Error::custom)
                }
                _ => Err(serde::de::Error::custom("Unknown channel")),
            }
        } else {
            Err(serde::de::Error::custom("Message must have either 'method' or 'channel' field"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_heartbeat_deserialization() {
        let json = json!({
            "channel": "heartbeat"
        });
        
        let message: Message = serde_json::from_value(json).unwrap();
        assert!(matches!(message, Message::Heartbeat(_)));
        
        if let Message::Heartbeat(msg) = message {
            assert_eq!(msg.channel, "heartbeat");
        } else {
            panic!("Expected Heartbeat message");
        }
    }

    #[test]
    fn test_status_deserialization() {
        let json = json!({
            "channel": "status",
            "data": [{
                "version": "2.0.9",
                "system": "online",
                "api_version": "v2",
                "connection_id": 1875109707426151_u64
            }]
        });
        
        let message: Message = serde_json::from_value(json).unwrap();
        assert!(matches!(message, Message::Status(_)));
        
        if let Message::Status(msg) = message {
            assert_eq!(msg.channel, "status");
            assert_eq!(msg.data.len(), 1);
            let status = &msg.data[0];
            assert_eq!(status.version, "2.0.9");
            assert_eq!(status.system, "online");
            assert_eq!(status.api_version, "v2");
            assert_eq!(status.connection_id, 1875109707426151);
        } else {
            panic!("Expected Status message");
        }
    }

    #[test]
    fn test_method_request_deserialization() {
        let json = json!({
            "method": "subscribe",
            "params": {
                "channel": "ticker",
                "symbol": ["BTC/USD"]
            }
        });
        
        let message: Message = serde_json::from_value(json).unwrap();
        assert!(matches!(message, Message::MethodRequest(_)));
        
        if let Message::MethodRequest(msg) = message {
            assert_eq!(msg.method, "subscribe");
            assert_eq!(msg.params.channel, "ticker");
            assert_eq!(msg.params.symbol, vec!["BTC/USD"]);
        } else {
            panic!("Expected MethodRequest message");
        }
    }

    #[test]
    fn test_method_response_deserialization() {
        let json = json!({
            "method": "subscribe",
            "result": {
                "channel": "ticker",
                "event_trigger": "trades",
                "snapshot": true,
                "symbol": "BTC/USD"
            },
            "success": true,
            "time_in": "2026-03-30T15:57:13.700934Z",
            "time_out": "2025-03-30T15:57:13.700984Z"
        });
        
        let message: Message = serde_json::from_value(json).unwrap();
        assert!(matches!(message, Message::MethodResponse(_)));
        
        if let Message::MethodResponse(msg) = message {
            assert_eq!(msg.method, "subscribe");
            assert!(msg.success);
            assert_eq!(msg.result.channel, "ticker");
            assert_eq!(msg.result.symbol, "BTC/USD");
        } else {
            panic!("Expected MethodResponse message");
        }
    }

    #[test]
    fn test_ticker_deserialization() {
        let json = json!({
            "channel": "ticker",
            "data": [{
                "symbol": "BTC/USD",
                "bid": "82724.9",
                "bid_qty": "1.03281699",
                "ask": "82725.0",
                "ask_qty": "7.90323505",
                "last": "82725.0",
                "volume": "839.19198788",
                "vwap": "82845.8",
                "low": "81655.0",
                "high": "83500.0",
                "change": "342.3",
                "change_pct": "0.42"
            }]
        });
        
        let message: Message = serde_json::from_value(json).unwrap();
        assert!(matches!(message, Message::ChannelMessage(ChannelMessage::Ticker(_))));
        
        if let Message::ChannelMessage(ChannelMessage::Ticker(msg)) = message {
            assert_eq!(msg.channel, "ticker");
            assert_eq!(msg.data.len(), 1);
            let ticker = &msg.data[0];
            assert_eq!(ticker.symbol, "BTC/USD");
            assert_eq!(ticker.bid, dec!(82724.9));
            assert_eq!(ticker.ask, dec!(82725.0));
        } else {
            panic!("Expected Ticker message");
        }
    }

    #[test]
    fn test_book_snapshot_deserialization() {
        let json = json!({
            "channel": "book",
            "type": "snapshot",
            "data": [{
                "symbol": "BTC/USD",
                "bids": [
                    {"price": 82721.9, "qty": 0.00016716},
                    {"price": 82716.9, "qty": 2.95141551}
                ],
                "asks": [
                    {"price": 82722.0, "qty": 3.63503536},
                    {"price": 82722.5, "qty": 3.02215343}
                ],
                "checksum": 2270016479_u64
            }]
        });
        
        let message: Message = serde_json::from_value(json).unwrap();
        assert!(matches!(message, Message::ChannelMessage(ChannelMessage::Book(BookMessage::Snapshot(_)))));
        
        if let Message::ChannelMessage(ChannelMessage::Book(BookMessage::Snapshot(msg))) = message {
            assert_eq!(msg.channel, "book");
            assert_eq!(msg.data.len(), 1);
            let book = &msg.data[0];
            assert_eq!(book.symbol, "BTC/USD");
            assert_eq!(book.bids.len(), 2);
            assert_eq!(book.asks.len(), 2);
            assert_eq!(book.checksum, 2270016479);
        } else {
            panic!("Expected Book Snapshot message");
        }
    }

    #[test]
    fn test_book_update_deserialization() {
        let json = json!({
            "channel": "book",
            "type": "update",
            "data": [{
                "symbol": "BTC/USD",
                "bids": [],
                "asks": [
                    {"price": 82722.0, "qty": 3.94426838}
                ],
                "checksum": 571581324,
                "timestamp": "2025-03-30T16:00:47.637703Z"
            }]
        });
        
        let message: Message = serde_json::from_value(json).unwrap();
        assert!(matches!(message, Message::ChannelMessage(ChannelMessage::Book(BookMessage::Update(_)))));
        
        if let Message::ChannelMessage(ChannelMessage::Book(BookMessage::Update(msg))) = message {
            assert_eq!(msg.channel, "book");
            assert_eq!(msg.data.len(), 1);
            let book = &msg.data[0];
            assert_eq!(book.symbol, "BTC/USD");
            assert!(book.bids.is_empty());
            assert_eq!(book.asks.len(), 1);
            assert_eq!(book.checksum, 571581324);
            assert!(book.timestamp.is_some());
        } else {
            panic!("Expected Book Update message");
        }
    }

    #[test]
    fn test_invalid_message() {
        let json = json!({
            "channel": "unknown_channel"
        });
        
        let result = Message::deserialize(json.into_deserializer());
        assert!(result.is_err());
    }

    #[test]
    fn test_roundtrip_serialization() {
        let messages = vec![
            Message::Heartbeat(HeartbeatMessage {
                channel: "heartbeat".to_string(),
            }),
            Message::Status(StatusMessage {
                channel: "status".to_string(),
                data: vec![StatusData {
                    version: "2.0.9".to_string(),
                    system: "online".to_string(),
                    api_version: "v2".to_string(),
                    connection_id: 1875109707426151,
                }],
            }),
            Message::MethodRequest(MethodRequest {
                method: "subscribe".to_string(),
                params: MethodParams {
                    channel: "ticker".to_string(),
                    symbol: vec!["BTC/USD".to_string()],
                },
            }),
            Message::MethodResponse(MethodResponse {
                method: "subscribe".to_string(),
                result: MethodResult {
                    channel: "ticker".to_string(),
                    event_trigger: Some("trades".to_string()),
                    snapshot: true,
                    symbol: "BTC/USD".to_string(),
                },
                success: true,
                time_in: "2026-03-30T15:57:13.700934Z".to_string(),
                time_out: "2025-03-30T15:57:13.700984Z".to_string(),
            }),
            Message::ChannelMessage(ChannelMessage::Ticker(TickerMessage {
                channel: "ticker".to_string(),
                data: vec![TickerData {
                    symbol: "BTC/USD".to_string(),
                    bid: dec!(82724.9),
                    bid_qty: dec!(1.03281699),
                    ask: dec!(82725.0),
                    ask_qty: dec!(7.90323505),
                    last: dec!(82725.0),
                    volume: dec!(839.19198788),
                    vwap: dec!(82845.8),
                    low: dec!(81655.0),
                    high: dec!(83500.0),
                    change: dec!(342.3),
                    change_pct: dec!(0.42),
                }],
            })),
        ];

        for message in messages {
            let json = serde_json::to_value(&message).unwrap();
            println!("Serialized JSON: {}", serde_json::to_string_pretty(&json).unwrap());
            let deserialized: Message = serde_json::from_value(json).unwrap();
            assert_eq!(message, deserialized);
        }
    }
}
