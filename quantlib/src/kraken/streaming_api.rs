use tungstenite::{connect, stream::MaybeTlsStream, WebSocket, Message};
use serde_json::json;
use crate::kraken::objects::KrakenMessage;


// PRICE STREAM
pub struct PriceStream {
    pub socket: WebSocket<MaybeTlsStream<std::net::TcpStream>>
}

impl PriceStream {
    pub fn subscribe(&mut self, instruments: Vec<String>) {
        let subscribe_message = json!({
            "method": "subscribe",
            "params": {
                "channel": "ticker",
                "symbol": instruments
            }
        }).to_string();

        self.socket.send(Message::Text(subscribe_message.into())).expect("Failed to subscribe to Kraken WebSocket API");
    }
}

impl Iterator for PriceStream {
    type Item = KrakenMessage;

    fn next(&mut self) -> Option<Self::Item> {
        let msg = self.socket.read().expect("Error reading message");
        Some(serde_json::from_str(&msg.to_text().unwrap()).expect(format!("Error deserializing message: {}", msg).as_str()))
    }
}

pub struct PriceStreamBuilder {
    instruments: Option<Vec<String>>,
    url: Option<String>
}

impl PriceStreamBuilder {
    pub fn new() -> PriceStreamBuilder {
        PriceStreamBuilder {
            instruments: None,
            url: None
        }
    }

    pub fn instruments(mut self, instruments: Vec<String>) -> PriceStreamBuilder {
        self.instruments = Some(instruments);
        self
    }

    pub fn url(mut self, url: String) -> PriceStreamBuilder {
        self.url = Some(url);
        self
    }

    pub fn build(self) -> PriceStream {
        // Defaults:
        // - instruments: ["BTC/USD"]
        // - url: "wss://ws.kraken.com/v2"
        
        let instruments = self.instruments.unwrap_or(vec!["BTC/USD".to_string()]);
        let url = self.url.unwrap_or("wss://ws.kraken.com/v2".to_string());
        
        // Open a WebSocket connection to the Kraken API
        let (socket, response) = connect(url).expect("Failed to connect to Kraken WebSocket API");
        println!("Connected to Kraken WebSocket API: {:#?}", response);

        let mut price_stream = PriceStream {
            socket
        };
        price_stream.subscribe(instruments);
        price_stream
    }
}


// ORDER BOOK STREAM

pub struct OrderBookStream {
    pub socket: WebSocket<MaybeTlsStream<std::net::TcpStream>>
}

impl OrderBookStream {
    pub fn subscribe(&mut self, instruments: Vec<String>) {
        let subscribe_message = json!({
            "method": "subscribe",
            "params": {
                "channel": "book",
                "symbol": instruments
            }
        }).to_string();

        self.socket.send(Message::Text(subscribe_message.into())).expect("Failed to subscribe to Kraken WebSocket API");
    }
}

impl Iterator for OrderBookStream {
    type Item = KrakenMessage;

    fn next(&mut self) -> Option<Self::Item> {
        let msg = self.socket.read().expect("Error reading message");
        Some(serde_json::from_str(&msg.to_text().unwrap()).expect(format!("Error deserializing message: {}", msg).as_str()))
    }
}

pub struct OrderBookStreamBuilder {
    instruments: Option<Vec<String>>,
    url: Option<String>
}

impl OrderBookStreamBuilder {
    pub fn new() -> OrderBookStreamBuilder {
        OrderBookStreamBuilder {
            instruments: None,
            url: None
        }
    }

    pub fn instruments(mut self, instruments: Vec<String>) -> OrderBookStreamBuilder {
        self.instruments = Some(instruments);
        self
    }

    pub fn url(mut self, url: String) -> OrderBookStreamBuilder {
        self.url = Some(url);
        self
    }

    pub fn build(self) -> OrderBookStream {
        // Defaults:
        // - instruments: ["BTC/USD"]
        // - url: "wss://ws.kraken.com/v2"
        
        let instruments = self.instruments.unwrap_or(vec!["BTC/USD".to_string()]);
        let url = self.url.unwrap_or("wss://ws.kraken.com/v2".to_string());
        
        // Open a WebSocket connection to the Kraken API
        let (socket, response) = connect(url).expect("Failed to connect to Kraken WebSocket API");
        println!("Connected to Kraken WebSocket API: {:#?}", response);

        let mut order_book_stream = OrderBookStream {
            socket
        };
        order_book_stream.subscribe(instruments);
        order_book_stream
    }
}