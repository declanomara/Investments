use tungstenite::{connect, stream::MaybeTlsStream, WebSocket, Message as WsMessage};
use std::error::Error;
use std::fmt;
use crate::kraken::objects::{Message, MethodRequest, MethodParams};

#[derive(Debug)]
pub enum StreamError {
    WebSocketError(tungstenite::Error),
    SubscriptionError(String),
    DeserializationError(serde_json::Error),
    SerializationError(serde_json::Error),
}

impl fmt::Display for StreamError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StreamError::WebSocketError(e) => write!(f, "WebSocket error: {}", e),
            StreamError::SubscriptionError(msg) => write!(f, "Subscription error: {}", msg),
            StreamError::DeserializationError(e) => write!(f, "Deserialization error: {}", e),
            StreamError::SerializationError(e) => write!(f, "Serialization error: {}", e),
        }
    }
}

impl Error for StreamError {}

// MARKET DATA STREAM
pub struct MarketDataStream {
    socket: WebSocket<MaybeTlsStream<std::net::TcpStream>>,
}

impl MarketDataStream {
    pub fn subscribe(&mut self, channel: &str, symbols: &[String]) -> Result<(), StreamError> {
        let subscribe_message = MethodRequest {
            method: "subscribe".to_string(),
            params: MethodParams {
                channel: channel.to_string(),
                symbol: symbols.to_vec(),
            },
        };

        let json = serde_json::to_string(&subscribe_message)
            .map_err(|e| StreamError::SerializationError(e))?;

        self.socket.send(WsMessage::Text(json.into()))
            .map_err(StreamError::WebSocketError)?;

        Ok(())
    }

    pub fn as_objects(self) -> MarketDataObjectStream<Self> {
        MarketDataObjectStream::new(self)
    }
}

impl Iterator for MarketDataStream {
    type Item = Result<String, StreamError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.socket.read() {
            Ok(msg) => {
                match msg {
                    WsMessage::Text(text) => Some(Ok(text.to_string())),
                    WsMessage::Ping(_) => {
                        if let Err(e) = self.socket.send(WsMessage::Pong(vec![].into())) {
                            Some(Err(StreamError::WebSocketError(e)))
                        } else {
                            self.next()
                        }
                    }
                    WsMessage::Pong(_) => self.next(),
                    WsMessage::Close(_) => None,
                    _ => self.next(),
                }
            }
            Err(e) => Some(Err(StreamError::WebSocketError(e))),
        }
    }
}

pub struct MarketDataStreamBuilder {
    channel: Option<String>,
    symbols: Option<Vec<String>>,
}

impl MarketDataStreamBuilder {
    pub fn new() -> Self {
        MarketDataStreamBuilder {
            channel: None,
            symbols: None,
        }
    }

    pub fn channel(mut self, channel: String) -> Self {
        self.channel = Some(channel);
        self
    }

    pub fn symbols(mut self, symbols: Vec<String>) -> Self {
        self.symbols = Some(symbols);
        self
    }

    pub fn build(self) -> Result<MarketDataStream, StreamError> {
        let (socket, _) = connect("wss://ws.kraken.com/v2")
            .map_err(|e| StreamError::WebSocketError(e))?;

        let mut stream = MarketDataStream { socket };

        if let (Some(channel), Some(symbols)) = (self.channel, self.symbols) {
            stream.subscribe(&channel, &symbols)?;
        }

        Ok(stream)
    }
}

pub struct MarketDataObjectStream<I> {
    inner: I,
}

impl<I> MarketDataObjectStream<I> {
    pub fn new(inner: I) -> Self {
        Self { inner }
    }
}

impl<I> Iterator for MarketDataObjectStream<I>
where
    I: Iterator<Item = Result<String, StreamError>>
{
    type Item = Result<Message, StreamError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next() {
            Some(Ok(json_str)) => {
                match serde_json::from_str(&json_str) {
                    Ok(message) => Some(Ok(message)),
                    Err(e) => Some(Err(StreamError::DeserializationError(e))),
                }
            }
            Some(Err(e)) => Some(Err(e)),
            None => None,
        }
    }
}