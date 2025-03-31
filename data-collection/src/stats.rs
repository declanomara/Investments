use std::time::Instant;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use quantlib::kraken::objects::{Message, ChannelMessage};

#[derive(Debug)]
pub struct Stats {
    // Message counts
    total_messages: u64,
    messages_per_channel: HashMap<String, u64>,
    messages_per_symbol: HashMap<String, u64>,
    
    // Connection health
    connection_start_time: DateTime<Utc>,
    last_message_time: Option<DateTime<Utc>>,
    reconnection_attempts: u32,
    
    // Rate tracking
    message_rate_start: Instant,
    messages_since_rate_start: u64,
    
    // Error tracking
    malformed_messages: u64,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            total_messages: 0,
            messages_per_channel: HashMap::new(),
            messages_per_symbol: HashMap::new(),
            connection_start_time: Utc::now(),
            last_message_time: None,
            reconnection_attempts: 0,
            message_rate_start: Instant::now(),
            messages_since_rate_start: 0,
            malformed_messages: 0,
        }
    }

    pub fn update(&mut self, message: &Message) {
        self.total_messages += 1;
        self.messages_since_rate_start += 1;
        self.last_message_time = Some(Utc::now());

        // Update channel counts
        let channel = match message {
            Message::Heartbeat(msg) => &msg.channel,
            Message::Status(msg) => &msg.channel,
            Message::MethodRequest(msg) => &msg.params.channel,
            Message::MethodResponse(msg) => &msg.result.channel,
            Message::ChannelMessage(msg) => match msg {
                ChannelMessage::Ticker(msg) => &msg.channel,
                ChannelMessage::Book(msg) => match msg {
                    quantlib::kraken::objects::BookMessage::Snapshot(msg) => &msg.channel,
                    quantlib::kraken::objects::BookMessage::Update(msg) => &msg.channel,
                },
            },
        };
        
        *self.messages_per_channel.entry(channel.clone()).or_insert(0) += 1;

        // Update symbol counts for relevant messages
        if let Message::ChannelMessage(msg) = message {
            let symbol = match msg {
                ChannelMessage::Ticker(msg) => &msg.data[0].symbol,
                ChannelMessage::Book(msg) => match msg {
                    quantlib::kraken::objects::BookMessage::Snapshot(msg) => &msg.data[0].symbol,
                    quantlib::kraken::objects::BookMessage::Update(msg) => &msg.data[0].symbol,
                },
            };
            *self.messages_per_symbol.entry(symbol.clone()).or_insert(0) += 1;
        }
    }

    pub fn record_malformed_message(&mut self) {
        self.malformed_messages += 1;
    }

    pub fn record_reconnection(&mut self) {
        self.reconnection_attempts += 1;
    }

    pub fn get_stats(&self) -> String {
        let elapsed = self.message_rate_start.elapsed();
        let messages_per_second = if elapsed.as_secs() > 0 {
            self.messages_since_rate_start as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };

        let uptime = Utc::now() - self.connection_start_time;
        let last_message = self.last_message_time
            .map(|t| format!("{} seconds ago", (Utc::now() - t).num_seconds()))
            .unwrap_or_else(|| "never".to_string());

        format!(
            "\nData Collection Statistics:
Total Messages: {}
Messages/Second: {:.2}
Uptime: {} seconds
Last Message: {}
Reconnection Attempts: {}
Malformed Messages: {}

Messages by Channel:
{}

Messages by Symbol:
{}",
            self.total_messages,
            messages_per_second,
            uptime.num_seconds(),
            last_message,
            self.reconnection_attempts,
            self.malformed_messages,
            self.format_hashmap(&self.messages_per_channel),
            self.format_hashmap(&self.messages_per_symbol)
        )
    }

    fn format_hashmap(&self, map: &HashMap<String, u64>) -> String {
        map.iter()
            .map(|(k, v)| format!("  {}: {}", k, v))
            .collect::<Vec<_>>()
            .join("\n")
    }
} 