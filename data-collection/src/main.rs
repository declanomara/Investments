use quantlib;
use quantlib::logging;

// Ensure output directory exists
fn validate_output_directory(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create the directory if it doesn't exist
    if !std::path::Path::new(path).exists() {
        logging::info(&format!("Creating output directory at {}...", path));
        std::fs::create_dir_all(path)?;
    }

    // Create the raw.log file within the output directory
    let raw_log_path = format!("{}raw.log", path);
    if !std::path::Path::new(&raw_log_path).exists() {
        logging::info(&format!("Creating raw.log file at {}...", raw_log_path));
        std::fs::File::create(&raw_log_path)?;
    }
    logging::info(format!("Saving raw data to {}...", raw_log_path).as_str());

    // Create bin/ directory within the output directory
    let bin_path = format!("{}bin/", path);
    if !std::path::Path::new(&bin_path).exists() {
        logging::info(&format!("Creating bin/ directory at {}...", bin_path));
        std::fs::create_dir_all(&bin_path)?;
    }
    logging::info(format!("Saving binary files to {}...", bin_path).as_str());
    Ok(())
}

fn handle_error(e: Box<dyn std::error::Error>) {
    match e.downcast_ref::<tokio::time::error::Elapsed>() {
        Some(_elapsed_error) => {
            // Handle the elapsed error here
            logging::error("Connection timed out.");
        },
        None => {
            // Handle other errors here
            logging::error(e.to_string().as_str());
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log_path = "data/";
    logging::info("Validating output directory...");

    validate_output_directory(log_path)?;

    let settings = quantlib::util::read_settings().unwrap_or_else(|err| {
        logging::error(&format!("Failed to read settings: {}", err));
        std::process::exit(1);
    });

    let logging_price_stream = quantlib::oanda::LoggingPriceStream::new(
        &["EUR_USD".to_string()],
        log_path,
        10_000, // 10 second timeout, we expect a heartbeat every 5 seconds
        &settings.oanda
    ).await;

    for item in logging_price_stream {
        match item {
            Ok(quantlib::oanda::StreamItem::Price(price)) => {
                logging::info(&format!("[{}] Bid: {:.5} Ask: {:.5}", price.instrument, price.bid, price.ask));
            },
            Ok(quantlib::oanda::StreamItem::Heartbeat(_)) => {
                // logging::debug("Heartbeat received.");
            },
            Err(e) => {
                handle_error(e);
            }
        }
    }

    Ok(())
}
