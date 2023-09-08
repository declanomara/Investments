use quantlib;

// Ensure output directory exists
fn validate_output_directory(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create the directory if it doesn't exist
    if !std::path::Path::new(path).exists() {
        println!("[WARN] Output directory does not exist. Creating {}...", path);
        std::fs::create_dir_all(path)?;
    }

    // Create the raw.log file within the output directory
    let raw_log_path = format!("{}/raw.log", path);
    if !std::path::Path::new(&raw_log_path).exists() {
        println!("[INFO] Creating raw.log file at {}...", raw_log_path);
        std::fs::File::create(&raw_log_path)?;
    }

    // Create bin/ directory within the output directory
    let bin_path = format!("{}/bin", path);
    if !std::path::Path::new(&bin_path).exists() {
        println!("[INFO] Creating bin/ directory at {}...", bin_path);
        std::fs::create_dir_all(&bin_path)?;
    }
    Ok(())
}

fn handle_error(e: Box<dyn std::error::Error>) {
    match e.downcast_ref::<tokio::time::error::Elapsed>() {
        Some(_elapsed_error) => {
            // Handle the elapsed error here
            println!("[ERROR] Connection timed out.");
        },
        None => {
            // Handle other errors here
            println!("[ERROR] {}", e);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log_path = "data/";
    println!("[INFO] Validating output directory...");
    validate_output_directory(log_path)?;

    let settings = quantlib::util::read_settings()?;
    let logging_price_stream = quantlib::oanda::LoggingPriceStream::new(
        &["EUR_USD".to_string()],
        log_path,
        5000,
        &settings.oanda
    ).await;

    for item in logging_price_stream {
        match item {
            Ok(quantlib::oanda::StreamItem::Price(price)) => {
                println!("[{}][PRICE] Bid: {:.5} Ask: {:.5}", price.instrument, price.bid, price.ask);
            },
            Ok(quantlib::oanda::StreamItem::Heartbeat(_)) => {
                // println!("[HEARTBEAT]");
            },
            Err(e) => {
                handle_error(e);
            }
        }
    }

    Ok(())
}
