use quantlib;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log_path = "data/";
    let settings = quantlib::util::read_settings()?;
    let logging_price_stream = quantlib::oanda::LoggingPriceStream::new(
        &["EUR_USD".to_string()],
        log_path,
        &settings.oanda
    ).await;

    for item in logging_price_stream {
        // Match on the item to see what kind of stream item it is, if it's a price, print it out, otherwise ignore it
        match item {
            Ok(quantlib::oanda::StreamItem::Price(price)) => {
                println!("[{}][PRICE] Bid: {:.5} Ask: {:.5}", price.instrument, price.bid, price.ask);
            },
            _ => {}
        }
    }

    Ok(())
}
