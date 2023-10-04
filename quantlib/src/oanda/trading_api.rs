use reqwest::header::{HeaderMap, HeaderValue};

use crate::oanda::objects::API_URL;
use crate::oanda::objects::{Price, Response, Position, PositionResponse, OandaSettings};


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
