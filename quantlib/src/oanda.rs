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
    Heartbeat(Heartbeat),
}

async fn initialize_price_stream(
    instruments: &Vec<String>,
    settings: &OandaSettings,
) -> Result<reqwest::Response, Box<dyn std::error::Error>> {
    let instrument_list = instruments.join(",");
    let authorization = format!("Bearer {}", &settings.authorization);
    let account_id = &settings.account_id;

    let endpoint = format!(
        "/v3/accounts/{}/pricing/stream?instruments={}",
        account_id, instrument_list
    );
    let url = format!("{}{}", STREAMING_URL, endpoint);

    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_str(authorization.as_str())?,
    );

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

// TODO: Handle empty chunks (refer to LoggingPriceStream)
// Requires some refactoring to handle ownership of reference to settings
// Might be better to make OandaSettings cloneable
pub struct PriceStream {
    pub response: reqwest::Response,
    pub buffer: Vec<u8>,
}

impl PriceStream {
    pub async fn new(instruments: Vec<String>, settings: &OandaSettings) -> Self {
        let response = initialize_price_stream(&instruments, &settings)
            .await
            .unwrap();
        let buffer = Vec::new();

        PriceStream { response, buffer }
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
        } else {
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
            }
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

pub struct LoggingPriceStream<'a> {
    // Used for streaming data from OANDA
    pub response: reqwest::Response,
    pub buffer: Vec<u8>,
    pub buffered_items: std::collections::VecDeque<StreamItem>,

    // Config options
    pub log_path: String,
    pub timeout_duration: u64,
    pub settings: &'a OandaSettings,
    pub instruments: Vec<String>,

    // File writers
    pub raw_log_writer: std::io::BufWriter<std::fs::File>,
    pub bin_log_writers: std::collections::HashMap<String, std::io::BufWriter<std::fs::File>>,
}

impl<'a> LoggingPriceStream<'a> {
    // TODO: Refactor to return Result instead of panicking
    pub async fn new(
        instruments: Vec<String>,
        log_path: &str,
        timeout_duration: u64,
        settings: &'a OandaSettings,
    ) -> Result<LoggingPriceStream<'a>, Box<dyn std::error::Error>> {
        // Open connection to OANDA
        let response = initialize_price_stream(&instruments, &settings).await?;
        let buffer = Vec::new();
        let buffered_items = std::collections::VecDeque::new();

        // Create buffered writers for raw data
        let raw_log_path = format!("{}/raw.log", log_path);
        let mut options = std::fs::OpenOptions::new();
        let raw_log_file = options.append(true).create(true).open(raw_log_path)?;
        let raw_log_writer = std::io::BufWriter::new(raw_log_file);

        // Create hashmap to store buffered writers for binary data, but don't open files yet
        // Binary files will be opened when the first price for each instrument is received
        let bin_log_writers: std::collections::HashMap<String, std::io::BufWriter<std::fs::File>> =
            std::collections::HashMap::new();

        Ok(LoggingPriceStream {
            response,
            buffer,
            buffered_items,

            timeout_duration,
            settings,
            instruments,
            log_path: log_path.to_string(),

            raw_log_writer,
            bin_log_writers,
        })
    }

    pub async fn refresh_connection(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Refresh connection by closing the current one and opening a new one
        self.response = initialize_price_stream(&self.instruments, &self.settings).await?;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Buffered writers need to be flushed before closing to avoid losing data
        // Buffer size is relatively large (8KB)
        self.raw_log_writer.flush()?;
        for (_, writer) in self.bin_log_writers.iter_mut() {
            writer.flush()?;
        }
        Ok(())
    }

    pub async fn log_price(&mut self, price: &Price) {
        // TODO: Parse timestamp from price
        let timestamp: u64 = 0;

        // Attempt to get buffered writer for instrument from hashmap, otherwise create a new one
        let bin_log_writer = self
            .bin_log_writers
            .entry(price.instrument.clone())
            .or_insert_with(|| {
                // Create binary log file for instrument with append and create flags
                let path = format!("{}/bin/{}.bin", self.log_path, price.instrument);
                let mut options = std::fs::OpenOptions::new();
                let file = options
                    .append(true)
                    .create(true)
                    .open(path)
                    .unwrap_or_else(|err| {
                        panic!("Failed to open binary log file: {}", err);
                    });

                // Optimal buffer size is likely 8KB as 4KB is the default page size on most systems
                // 8KB = 250 32 byte records, unlikely to be less than 1 second of data
                let writer = std::io::BufWriter::with_capacity(8 * 1024, file);
                writer
            });
        
        // TODO: Standardize Price to binary format conversion in quantlib
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
        // Write the entire raw response to a file unmodified
        // In the future, we may want to parse the response differently, so we don't want to lose any data
        self.raw_log_writer.write_all(&chunk).unwrap();
    }

    async fn parse_chunk(&mut self, chunk: &[u8]) -> Vec<StreamItem> {
        // Move chunk into buffer
        self.buffer.extend_from_slice(&chunk);

        // Parse buffer into stream of StreamItems (Price or Heartbeat) using serde_json
        let stream = Deserializer::from_slice(&self.buffer);
        let mut stream = stream.into_iter::<StreamItem>();

        let mut items = Vec::new();
        let mut last_parsed_index = 0;
        while let Some(result) = stream.next() {
            match result {
                Ok(item) => {
                    items.push(item);
                    last_parsed_index = stream.byte_offset();
                }
                Err(err) => {
                    println!("Error parsing JSON: {}", err);
                    println!("Buffer: {}", std::str::from_utf8(&self.buffer).unwrap());
                }
            }
        }

        // Remove parsed JSON strings from buffer
        // self.buffer.drain(..last_parsed_index)
        
        // TODO: Remove this debug code and replace with above line
        let total_buffer = std::str::from_utf8(&self.buffer).unwrap().to_string();
        let parsed: Vec<u8> = self.buffer.drain(..last_parsed_index).collect();
        let parsed = std::str::from_utf8(parsed.as_slice()).unwrap();
        let remaining_buffer = std::str::from_utf8(&self.buffer).unwrap();
        
        if remaining_buffer != "\n" {
            println!("Parsed buffer unexpectedly.");
            println!("Total buffer: {:?}", total_buffer);
            println!("Parsed {} bytes", last_parsed_index);
            println!("Bytes parsed: {:?}", parsed);
            println!("Remaining buffer: {:?}", remaining_buffer);
        }

        items
    }

    pub async fn next_items(
        &mut self,
        timeout_duration: u64,
    ) -> Result<Vec<StreamItem>, Box<dyn std::error::Error>> {
        // Get next chunk from OANDA, timeout after timeout_duration milliseconds
        let chunk = timeout(
            std::time::Duration::from_millis(timeout_duration),
            self.response.chunk(),
        )
        .await??; // ?? because reqwest may return an error, and timeout may return an error

        if let Some(chunk) = chunk {
            // Log raw response before parsing
            self.log_raw(&chunk).await;

            // TEMPORARY TESTING CHUNK PARSING
            // Split chunk in half and parse each half sequentially to ensure we're not losing data on chunk boundaries
            // let half = chunk.len() / 2;
            // let first_half = &chunk[..half];
            // let second_half = &chunk[half..];
            // let first_items = self.parse_chunk(&first_half).await;
            // let second_items = self.parse_chunk(&second_half).await;
            // let mut items = first_items;
            // items.extend(second_items);
            // return Ok(items);


            let items = self.parse_chunk(&chunk).await;
            return Ok(items);
        } else {
            return Err("Received empty chunk".into());
        }
    }
}

impl<'a> Iterator for LoggingPriceStream<'a> {
    type Item = Result<StreamItem, Box<dyn std::error::Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        // If there are any buffered items, return them first
        if let Some(item) = self.buffered_items.pop_front() {
            return Some(Ok(item));
        }

        // Otherwise, get next chunk from OANDA and parse it
        let items = futures::executor::block_on(self.next_items(self.timeout_duration));
        match items {
            Ok(items) => {
                for item in items {
                    // Log prices to binary files
                    match &item {
                        StreamItem::Price(price) => {
                            futures::executor::block_on(self.log_price(price));
                        }
                        _ => {}
                    }

                    // Add all items to buffer to be returned by next() calls
                    self.buffered_items.push_back(item);
                }
            }
            Err(err) => {
                return Some(Err(err));
            }
        }

        // Return first item from buffer, if any
        if let Some(item) = self.buffered_items.pop_front() {
            return Some(Ok(item));
        }
        else {
            return None;
        }
    }
}

pub async fn get_latest_prices(
    instruments: &[String],
    settings: &OandaSettings,
) -> Result<Vec<Price>, Box<dyn std::error::Error>> {
    let instrument_list = instruments.join(",");
    let authorization = format!("Bearer {}", &settings.authorization);
    let account_id = &settings.account_id;

    let endpoint = format!(
        "/v3/accounts/{}/pricing?instruments={}",
        account_id, instrument_list
    );
    let url = format!("{}{}", API_URL, endpoint);

    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_str(authorization.as_str())?,
    );
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

pub async fn place_market_order(
    instrument: &str,
    units: f64,
    settings: &OandaSettings,
) -> Result<(), Box<dyn std::error::Error>> {
    let authorization = format!("Bearer {}", &settings.authorization);
    let account_id = &settings.account_id;

    let endpoint = format!("/v3/accounts/{}/orders", account_id);
    let url = format!("{}{}", API_URL, endpoint);

    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_str(authorization.as_str())?,
    );
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

pub async fn get_positions(
    settings: &OandaSettings,
) -> Result<Vec<Position>, Box<dyn std::error::Error>> {
    let authorization = format!("Bearer {}", &settings.authorization);
    let account_id = &settings.account_id;

    let endpoint = format!("/v3/accounts/{}/positions", account_id);
    let url = format!("{}{}", API_URL, endpoint);

    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_str(authorization.as_str())?,
    );
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

pub async fn get_position(
    instrument: &str,
    settings: &OandaSettings,
) -> Result<Position, Box<dyn std::error::Error>> {
    let positions = get_positions(settings).await?;
    let position = positions.into_iter().find(|p| p.instrument == instrument);

    if position.is_none() {
        return Err(format!("No position found for instrument {}", instrument).into());
    }

    Ok(position.unwrap())
}
