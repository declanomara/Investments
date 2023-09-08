use quantlib;

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
