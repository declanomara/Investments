use reqwest::header::{HeaderMap, HeaderValue};

use serde_json::Deserializer;

use tokio::time::timeout;

use std::io::Write;

use crate::oanda::objects::{STREAMING_URL, API_URL};
use crate::oanda::objects::{Price, Response, StreamItem, OandaSettings, Position, PositionResponse};


