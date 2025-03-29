use crate::oanda::objects::Settings;
use crate::kraken::objects::OrderBookData;

pub fn read_settings() -> Result<Settings, Box<dyn std::error::Error>> {
    let settings = std::fs::read_to_string("settings.json")?;
    serde_json::from_str(&settings).map_err(|e| e.into())
}

pub fn generate_timestamp() -> String {
    let now = chrono::Utc::now();
    now.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn generate_timestamp_filename() -> String {
    let now = chrono::Utc::now();
    now.format("%Y-%m-%d_%H-%M-%S").to_string()
}

pub fn generate_order_book(data: OrderBookData) -> String {
    // Create a graph of the order book
    let mut order_book_prices = Vec::new();
    let mut order_book_volumes = Vec::new();
    for level in data.bids.iter() {
        order_book_prices.push(level.price);
        order_book_volumes.push(level.qty);
    }

    // Normalize the volumes from 0 to 1
    let max_volume = order_book_volumes.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let min_volume = order_book_volumes.iter().cloned().fold(f64::INFINITY, f64::min);
    let normalized_volumes: Vec<f64> = order_book_volumes.iter().map(|x| (x - min_volume) / (max_volume - min_volume)).collect();

    // Create the order book graph
    let mut order_book_graph = String::new();
    for i in 0..order_book_prices.len() {
        order_book_graph.push_str(&format!("{},{}\n", order_book_prices[i], normalized_volumes[i]));
    }

    order_book_graph
}
