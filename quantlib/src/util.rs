use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::oanda::objects::Settings;

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

#[derive(Serialize, Deserialize, Debug)]
pub struct TradingConfig {
    pub instruments: Vec<String>,
    pub model: String,

    #[serde(flatten)]
    #[serde(rename = "modelConfig")]
    pub model_config: serde_json::Value,
}

impl TradingConfig {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        log::info!("Loading config from {:?}", path.as_ref());
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let config = serde_json::from_reader(reader)?;
        Ok(config)
    }
}
