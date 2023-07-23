use reqwest::header::{HeaderMap, HeaderValue};

use serde::Deserialize;
use serde_json::Deserializer;

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
    #[serde(deserialize_with = "deserialize_number_from_string")]
    #[serde(rename = "closeoutBid")]
    pub closeout_bid: f64,

    #[serde(deserialize_with = "deserialize_number_from_string")]
    #[serde(rename = "closeoutAsk")]
    pub closeout_ask: f64,

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

pub struct PriceStream {
    pub response: reqwest::Response,
    pub buffer: Vec<u8>,
}

impl PriceStream {
    pub async fn new(instruments: &[String], settings: &OandaSettings) -> Self {
        let response = Self::initialize_price_stream(instruments, &settings).await.unwrap();
        let buffer = Vec::new();

        PriceStream {
            response,
            buffer
        }
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

    async fn next_items(&mut self) -> Vec<StreamItem> {
        let chunk = self.response.chunk().await.unwrap();
        let items = self.parse_chunk(&chunk.unwrap()).await;
        items
    }
}

impl Iterator for PriceStream {
    type Item = Result<StreamItem, Box<dyn std::error::Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        let items = futures::executor::block_on(self.next_items());
        for item in items {
            return Some(Ok(item));
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
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub units: f64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    #[serde(rename = "unrealizedPL")]
    pub unrealized_pl: f64,
}

fn deserialize_number_from_string<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: &str = serde::Deserialize::deserialize(deserializer)?;
    s.parse::<f64>().map_err(serde::de::Error::custom)
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
