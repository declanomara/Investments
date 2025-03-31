mod config;
mod stats;

use quantlib::kraken::{MarketDataStreamBuilder};
use quantlib::util::generate_timestamp_filename;
use std::fs::File;
use std::io::{BufWriter, Write};
use clap::Parser;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments and load configuration
    let args = config::CliArgs::parse();
    let config = config::Config::load(args)?;
    
    println!("Starting data collection with configuration:");
    println!("  Subscriptions:");
    for (channel, symbols) in &config.subscriptions {
        println!("    {}: {:?}", channel, symbols);
    }
    println!("  Data directory: {:?}", config.data_dir);
    println!("  Monitor socket: {:?}", config.monitor_socket);

    // Create a new file for this session
    let filename = format!("{}/kraken_{}.jsonl", config.data_dir.display(), generate_timestamp_filename());
    let file = File::create(&filename)?;
    let mut writer = BufWriter::new(file);

    print!("Building market data stream...");
    let mut stream = MarketDataStreamBuilder::new().build()?;

    // Subscribe to each channel with its symbols
    for (channel, symbols) in &config.subscriptions {
        stream.subscribe(channel, symbols)?;
    }
    println!("success");

    // Initialize statistics
    let mut stats = stats::Stats::new();
    let mut last_stats_update = std::time::Instant::now();
    let stats_update_interval = Duration::from_secs(5);

    let mut message_count = 0;

    for msg in stream {
        match msg {
            Ok(json_str) => {
                // Write raw JSON to file
                writeln!(writer, "{}", json_str)?;
                message_count += 1;

                // Parse message for stats
                match serde_json::from_str::<quantlib::kraken::objects::Message>(&json_str) {
                    Ok(message) => stats.update(&message),
                    Err(_) => stats.record_malformed_message(),
                }

                // Flush periodically
                if message_count % 100 == 0 {
                    writer.flush()?;
                }

                // Update stats display periodically
                if last_stats_update.elapsed() >= stats_update_interval {
                    println!("\n{}", stats.get_stats());
                    last_stats_update = std::time::Instant::now();
                }
            }
            Err(e) => {
                eprintln!("Error receiving message: {}", e);
                stats.record_malformed_message();
            }
        }
    }

    println!("\nData collection complete. Messages saved to: {}", filename);
    Ok(())
} 