use quantlib;
use quantlib::logging;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// Ensure output directory exists
fn validate_output_directory(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create the directory if it doesn't exist
    if !std::path::Path::new(path).exists() {
        log::info!("Creating output directory at {}...", path);
        std::fs::create_dir_all(path)?;
    }

    // Create the raw.log file within the output directory
    let raw_log_path = format!("{}raw.log", path);
    if !std::path::Path::new(&raw_log_path).exists() {
        log::info!("Creating raw.log file at {}...", raw_log_path);
        std::fs::File::create(&raw_log_path)?;
    }
    log::info!("Saving raw data to {}...", raw_log_path);

    // Create bin/ directory within the output directory
    let bin_path = format!("{}bin/", path);
    if !std::path::Path::new(&bin_path).exists() {
        log::info!("Creating bin/ directory at {}...", bin_path);
        std::fs::create_dir_all(&bin_path)?;
    }
    log::info!("Saving binary files to {}...", bin_path);
    Ok(())
}

fn handle_error(e: Box<dyn std::error::Error>) {
    match e.downcast_ref::<tokio::time::error::Elapsed>() {
        Some(_elapsed_error) => {
            // Handle the elapsed error here
            log::error!("Connection timed out.");
        }
        None => {
            // Handle other errors here
            log::error!("{}", e);
        }
    }
}

fn get_instruments() -> Vec<String> {
    vec![
        "AUD_CAD".to_string(),
        "AUD_CHF".to_string(),
        "AUD_HKD".to_string(),
        "AUD_JPY".to_string(),
        "AUD_NZD".to_string(),
        "AUD_SGD".to_string(),
        "AUD_USD".to_string(),
        "CAD_CHF".to_string(),
        "CAD_HKD".to_string(),
        "CAD_JPY".to_string(),
        "CAD_SGD".to_string(),
        "CHF_HKD".to_string(),
        "CHF_JPY".to_string(),
        "CHF_ZAR".to_string(),
        "EUR_AUD".to_string(),
        "EUR_CAD".to_string(),
        "EUR_CHF".to_string(),
        "EUR_CZK".to_string(),
        "EUR_DKK".to_string(),
        "EUR_GBP".to_string(),
        "EUR_HKD".to_string(),
        "EUR_HUF".to_string(),
        "EUR_JPY".to_string(),
        "EUR_NOK".to_string(),
        "EUR_NZD".to_string(),
        "EUR_PLN".to_string(),
        "EUR_SEK".to_string(),
        "EUR_SGD".to_string(),
        "EUR_TRY".to_string(),
        "EUR_USD".to_string(),
        "EUR_ZAR".to_string(),
        "GBP_AUD".to_string(),
        "GBP_CAD".to_string(),
        "GBP_CHF".to_string(),
        "GBP_HKD".to_string(),
        "GBP_JPY".to_string(),
        "GBP_NZD".to_string(),
        "GBP_PLN".to_string(),
        "GBP_SGD".to_string(),
        "GBP_USD".to_string(),
        "GBP_ZAR".to_string(),
        "HKD_JPY".to_string(),
        "NZD_CAD".to_string(),
        "NZD_CHF".to_string(),
        "NZD_HKD".to_string(),
        "NZD_JPY".to_string(),
        "NZD_SGD".to_string(),
        "NZD_USD".to_string(),
        "SGD_CHF".to_string(),
        "SGD_JPY".to_string(),
        "TRY_JPY".to_string(),
        "USD_CAD".to_string(),
        "USD_CHF".to_string(),
        "USD_CNH".to_string(),
        "USD_CZK".to_string(),
        "USD_DKK".to_string(),
        "USD_HKD".to_string(),
        "USD_HUF".to_string(),
        "USD_JPY".to_string(),
        "USD_MXN".to_string(),
        "USD_NOK".to_string(),
        "USD_PLN".to_string(),
        "USD_SEK".to_string(),
        "USD_SGD".to_string(),
        "USD_THB".to_string(),
        "USD_TRY".to_string(),
        "USD_ZAR".to_string(),
        "ZAR_JPY".to_string()
    ]
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Handle SIGINT
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    // Configure logger
    logging::configure_logger("logs/data-collection.log")?;

    // Ensure output directory exists
    let output_dir = "data/";
    log::info!("Validating output directory...");
    validate_output_directory(output_dir)?;

    // Read settings
    let settings = quantlib::util::read_settings().unwrap_or_else(|err| {
        log::error!("Failed to read settings: {}", err);
        std::process::exit(1);
    });

    let instruments = get_instruments();
    log::info!("Starting logging price stream for {} instruments...", instruments.len());
    let mut logging_price_stream = quantlib::oanda::LoggingPriceStream::new(
        instruments,
        output_dir,
        10_000, // 10 second timeout, we expect a heartbeat every 5 seconds
        &settings.oanda,
    )
    .await?;

    while let Some(item) = logging_price_stream.next() {
        match item {
            Ok(quantlib::oanda::StreamItem::Price(price)) => {
                // It appears that the logging macros are not oppressively slow
                log::info!(
                    "[{}] Bid: {:.5} Ask: {:.5}",
                    price.instrument, price.bid, price.ask
                );

                // println!(
                //     "[{}] Bid: {:.5} Ask: {:.5}",
                //     price.instrument, price.bid, price.ask
                // );
            }
            Ok(quantlib::oanda::StreamItem::Heartbeat(_)) => {
                log::debug!("Heartbeat received.");
            }
            Err(e) => {
                handle_error(e);
            }
        }

        // Handle SIGINT elegantly
        if running.load(Ordering::SeqCst) == false {
            log::info!("Received SIGINT, flushing buffers and exiting...");
            logging_price_stream.flush()?;
            break;
        }
    }

    Ok(())
}
