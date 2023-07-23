use std::error::Error;
use quantlib;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let settings = quantlib::util::read_settings()?;
    let price_stream = quantlib::oanda::PriceStream::new(&["EUR_USD".to_string()], &settings.oanda).await;

    for item in price_stream {
        // Match on the item to see what kind of stream item it is, if it's a price, print it out, otherwise ignore it
        match item {
            Ok(quantlib::oanda::StreamItem::Price(price)) => println!("{}: {}", price.instrument, price.closeout_bid),
            _ => {}
        }
    }

    Ok(())
}