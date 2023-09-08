use quantlib;

use tokio::time::timeout;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log_path = "data/";
    let settings = quantlib::util::read_settings()?;
    let mut logging_price_stream = quantlib::oanda::LoggingPriceStream::new(
        &["EUR_USD".to_string()],
        log_path,
        &settings.oanda
    ).await;

    loop {
        // Attempt to get new prices, but timeout after 5 seconds
        let prices = timeout(
            std::time::Duration::from_secs(5),
            logging_price_stream.next_items()
        ).await??;
        println!("{:?}", prices);
    }

    // Ok(())
}
