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
