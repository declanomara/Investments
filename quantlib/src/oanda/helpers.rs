pub fn deserialize_f64_from_string<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: &str = serde::Deserialize::deserialize(deserializer)?;
    s.parse::<f64>().map_err(serde::de::Error::custom)
}

pub fn deserialize_f32_from_string<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: &str = serde::Deserialize::deserialize(deserializer)?;
    s.parse::<f32>().map_err(serde::de::Error::custom)
}

pub fn deserialize_time_in_millis_from_string<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: &str = serde::Deserialize::deserialize(deserializer)?;

    // Parse time string into milliseconds since UNIX epoch
    // OANDA timestamps are in RFC3339 format: "2023-09-15T20:58:00.145575162Z"
    let datetime = chrono::DateTime::parse_from_rfc3339(s)
        .map_err(|e| serde::de::Error::custom(format!("Failed to parse datetime: {}", e)))?;
    let millis_since_epoch = datetime.timestamp_millis() as u64;
    Ok(millis_since_epoch)
}