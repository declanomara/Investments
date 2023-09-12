use reqwest::header::{HeaderMap, HeaderValue};

use serde::Deserialize;
use serde_json::Deserializer;

use tokio::time::timeout;

use std::io::Write;

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
struct Response {
    prices: Vec<Price>,
}


#[derive(Debug, Deserialize)]
pub struct Price {
    #[serde(deserialize_with = "deserialize_f32_from_string")]
    #[serde(rename = "closeoutBid")]
    pub bid: f32,

    #[serde(deserialize_with = "deserialize_f32_from_string")]
    #[serde(rename = "closeoutAsk")]
    pub ask: f32,

    pub time: String,
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
    Heartbeat(Heartbeat)
}

async fn initialize_price_stream(instruments: &[String], settings: &OandaSettings) -> Result<reqwest::Response, Box<dyn std::error::Error>> {
    let instrument_list = instruments.join(",");
    let authorization = format!("Bearer {}", &settings.authorization);
    let account_id = &settings.account_id;

    let endpoint = format!("/v3/accounts/{}/pricing/stream?instruments={}", account_id, instrument_list);
    let url = format!("{}{}", STREAMING_URL, endpoint);

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_str(authorization.as_str())?);
    
    let response = reqwest::Client::new()
        .get(&url)
        .headers(headers)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("Received non-success status code: {}", response.status()).into());
    }
    
    Ok(response)
}

pub struct PriceStream {
    pub response: reqwest::Response,
    pub buffer: Vec<u8>,
}

impl PriceStream {
    pub async fn new(instruments: &[String], settings: &OandaSettings) -> Self {
        let response = initialize_price_stream(instruments, &settings).await.unwrap();
        let buffer = Vec::new();

        PriceStream {
            response,
            buffer
        }
    }

    async fn parse_chunk(&mut self, chunk: &[u8]) -> Vec<StreamItem> {
        self.buffer.extend_from_slice(&chunk);
        let stream = Deserializer::from_slice(&self.buffer);
        let mut stream = stream.into_iter::<StreamItem>();

        let mut items = Vec::new();
        while let Some(result) = stream.next() {
            match result {
                Ok(item) => {
                    // println!("Buffer: {:?}", std::str::from_utf8(&self.buffer).unwrap());
                    items.push(item);
                }
                Err(err) => {
                    println!("Error parsing JSON: {}", err);
                    println!("Buffer: {:?}", std::str::from_utf8(&self.buffer).unwrap());
                }
            }
        }

        self.buffer.clear();
        items
    }

    async fn next_items(&mut self) -> Result<Vec<StreamItem>, Box<dyn std::error::Error>> {
        let chunk = self.response.chunk().await?;
        if let Some(chunk) = chunk {
            let items = self.parse_chunk(&chunk).await;
            return Ok(items);
        }
        else {
            return Err("Received empty chunk".into());
        }
    }
}

impl Iterator for PriceStream {
    type Item = Result<StreamItem, Box<dyn std::error::Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        let items = futures::executor::block_on(self.next_items());
        match items {
            Ok(items) => {
                for item in items {
                    return Some(Ok(item));
                }
            },
            Err(err) => {
                return Some(Err(err));
            }
        }
        None
    }
}

// Logging price stream (might be a better solution than two different types of PriceStreams... maybe should be trait?)
// TODO: Refactor to use trait
// Same as PriceStream, but also logs to data files:
// - raw.log: Raw JSON data from Oanda
// - bin/{instrument}.bin: Binary data for each instrument including timestamp, bid, and ask

pub struct LoggingPriceStream {
    pub response: reqwest::Response,
    pub timeout_duration: u64,
    pub buffer: Vec<u8>,

    pub log_path: String,

    pub raw_log_writer: std::io::BufWriter<std::fs::File>,
    pub bin_log_writers: std::collections::HashMap<String, std::io::BufWriter<std::fs::File>>,
}

impl LoggingPriceStream {
    pub async fn new(instruments: &[String], log_path: &str, timeout_duration: u64, settings: &OandaSettings) -> Self {
        let response = initialize_price_stream(instruments, &settings).await.unwrap();
        let buffer = Vec::new();

        let raw_log_path = format!("{}/raw.log", log_path);
        let mut options = std::fs::OpenOptions::new();
        let raw_log_file = options.append(true).create(true).open(raw_log_path).unwrap_or_else(|err| {
            panic!("Failed to open raw log file: {}", err);
        });
        let raw_log_writer = std::io::BufWriter::new(raw_log_file);
        
        let bin_log_writers: std::collections::HashMap<String, std::io::BufWriter<std::fs::File>>= std::collections::HashMap::new();

        LoggingPriceStream {
            response,
            timeout_duration,
            buffer,
            log_path: log_path.to_string(),
            raw_log_writer,
            bin_log_writers,
        }
    }

    pub fn flush(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.raw_log_writer.flush()?;
        for (_, writer) in self.bin_log_writers.iter_mut() {
            writer.flush()?;
        }
        Ok(())
    }

    pub async fn log_price(&mut self, price: &Price) {
        // TODO: Parse timestamp from price
        let timestamp: u64 = 0;
    
        let bin_log_writer = self.bin_log_writers.entry(price.instrument.clone()).or_insert_with(|| {
            let path = format!("{}/bin/{}.bin", self.log_path, price.instrument);
            let mut options = std::fs::OpenOptions::new();
            let file = options.append(true).create(true).open(path).unwrap_or_else(|err| {
                panic!("Failed to open binary log file: {}", err);
            });

            // TODO: Determine optimal buffer size
            let writer = std::io::BufWriter::with_capacity(8 * 32, file);
            writer
        });
    
        if let Err(err) = bin_log_writer.write_all(&timestamp.to_be_bytes()) {
            panic!("Failed to write timestamp to binary log file: {}", err);
        }
        if let Err(err) = bin_log_writer.write_all(&price.bid.to_be_bytes()) {
            panic!("Failed to write bid price to binary log file: {}", err);
        }
        if let Err(err) = bin_log_writer.write_all(&price.ask.to_be_bytes()) {
            panic!("Failed to write ask price to binary log file: {}", err);
        }
    }

    pub async fn log_raw(&mut self, chunk: &[u8]) {
        self.raw_log_writer.write_all(&chunk).unwrap();
    }

    async fn parse_chunk(&mut self, chunk: &[u8]) -> Vec<StreamItem> {
        self.buffer.extend_from_slice(&chunk);
        let stream = Deserializer::from_slice(&self.buffer);
        let mut stream = stream.into_iter::<StreamItem>();

        let mut items = Vec::new();
        while let Some(result) = stream.next() {
            match result {
                Ok(item) => {
                    items.push(item);
                }
                Err(err) => {
                    println!("Error parsing JSON: {}", err);
                    println!("Buffer: {:?}", std::str::from_utf8(&self.buffer).unwrap());
                }
            }
        }

        self.buffer.clear();
        items
    }

    pub async fn next_items(&mut self, timeout_duration: u64) -> Result<Vec<StreamItem>, Box<dyn std::error::Error>> {
        let chunk = timeout(
            std::time::Duration::from_millis(timeout_duration),
            self.response.chunk()
        ).await??; // ?? because reqwest may return an error, and timeout may return an error

        if let Some(chunk) = chunk {
            self.log_raw(&chunk).await;
            let items = self.parse_chunk(&chunk).await;
            return Ok(items);
        }
        else {
            return Err("Received empty chunk".into());
        }
    }

}


impl Iterator for LoggingPriceStream {
    type Item = Result<StreamItem, Box<dyn std::error::Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        let items = futures::executor::block_on(
            self.next_items(self.timeout_duration)
        );
        match items {
            Ok(items) => {
                for item in items {
                    match &item {
                        StreamItem::Price(price) => {
                            futures::executor::block_on(self.log_price(price));
                        },
                        _ => {}
                    }
                    return Some(Ok(item));
                }
            },
            Err(err) => {
                return Some(Err(err));
            }
        }
        None
    }
}


pub async fn get_latest_prices(instruments: &[String], settings: &OandaSettings) -> Result<Vec<Price>, Box<dyn std::error::Error>> {
    let instrument_list = instruments.join(",");
    let authorization = format!("Bearer {}", &settings.authorization);
    let account_id = &settings.account_id;
    
    let endpoint = format!("/v3/accounts/{}/pricing?instruments={}", account_id, instrument_list);
    let url = format!("{}{}", API_URL, endpoint);
    
    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_str(authorization.as_str())?);
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));

    let body = reqwest::Client::new()
        .get(&url)
        .headers(headers)
        .send()
        .await?
        .text()
        .await?;

    let response: Response = serde_json::from_str(&body).unwrap();
    let prices = response.prices;

    Ok(prices)
}

pub async fn place_market_order(instrument: &str, units: f64, settings: &OandaSettings) -> Result<(), Box<dyn std::error::Error>> {
    let authorization = format!("Bearer {}", &settings.authorization);
    let account_id = &settings.account_id;
    
    let endpoint = format!("/v3/accounts/{}/orders", account_id);
    let url = format!("{}{}", API_URL, endpoint);

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_str(authorization.as_str())?);
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));

    let body = format!("{{\"order\": {{\"units\": \"{}\", \"instrument\": \"{}\", \"timeInForce\": \"FOK\", \"type\": \"MARKET\", \"positionFill\": \"DEFAULT\"}}}}", units, instrument);

    let response = reqwest::Client::new()
        .post(&url)
        .headers(headers)
        .body(body)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("Received non-success status code: {}", response.status()).into());
    }

    // TODO: Parse response body for order ID
    Ok(())
}

#[derive(Debug, Deserialize)]
struct PositionResponse {
    positions: Vec<Position>,
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
        println!("Net units: {}+{}={}", self.long.units, self.short.units, net);
        net
    }

    pub fn unrealized_pl(&self) -> f64 {
        self.long.unrealized_pl + self.short.unrealized_pl
    }
}

fn deserialize_f64_from_string<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: &str = serde::Deserialize::deserialize(deserializer)?;
    s.parse::<f64>().map_err(serde::de::Error::custom)
}

fn deserialize_f32_from_string<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: &str = serde::Deserialize::deserialize(deserializer)?;
    s.parse::<f32>().map_err(serde::de::Error::custom)
}

pub async fn get_positions(settings: &OandaSettings) -> Result<Vec<Position>, Box<dyn std::error::Error>> {
    let authorization = format!("Bearer {}", &settings.authorization);
    let account_id = &settings.account_id;

    let endpoint = format!("/v3/accounts/{}/positions", account_id);
    let url = format!("{}{}", API_URL, endpoint);

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_str(authorization.as_str())?);
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));

    let response = reqwest::Client::new()
        .get(&url)
        .headers(headers)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("Received non-success status code: {}", response.status()).into());
    }

    let body = response.text().await?;

    let json_response = serde_json::from_str::<PositionResponse>(&body);

    if json_response.is_err() {
        println!("API Response: {}", body);
        return Err(format!("Error parsing JSON: {}", json_response.err().unwrap()).into());
    }
    let positions = json_response.unwrap().positions;

    Ok(positions)
}

pub async fn get_position(instrument: &str, settings: &OandaSettings) -> Result<Position, Box<dyn std::error::Error>> {
    let positions = get_positions(settings).await?;
    let position = positions.into_iter().find(|p| p.instrument == instrument);

    if position.is_none() {
        return Err(format!("No position found for instrument {}", instrument).into());
    }

    Ok(position.unwrap())
}
