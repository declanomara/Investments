use crate::oanda;

pub fn read_settings() -> Result<oanda::Settings, Box<dyn std::error::Error>> {
    let settings = std::fs::read_to_string("settings.json")?;
    serde_json::from_str(&settings).map_err(|e| e.into())
}