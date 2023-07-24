use std::error::Error;
use quantlib;
use quantlib::models::AlphaModel;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let settings = quantlib::util::read_settings()?;
    let price_stream = quantlib::oanda::PriceStream::new(&["EUR_USD".to_string()], &settings.oanda).await;

    let ema = quantlib::models::ExponentialMovingAverage::new("EUR_USD".to_string(), 0.05, 0.1);
    let sma = quantlib::models::SimpleMovingAverage::new("EUR_USD".to_string(), 10);
    let mut consensus = quantlib::models::WeightedConsensus::new()
        .add_model(Box::new(ema), 0.5)
        .add_model(Box::new(sma), 0.5);

    for item in price_stream {
        // Match on the item to see what kind of stream item it is, if it's a price, print it out, otherwise ignore it
        match item {
            Ok(quantlib::oanda::StreamItem::Price(price)) => {
                println!("{}: {}", price.instrument, price.closeout_bid);
                let signal = consensus.tick(&price)?;
                // if magnitude of signal is greater than 0.5, then print it out
                if let Some(signal) = signal {
                    if signal.forecast.abs() > 0.5 {
                        println!("{:?}", signal);
                    }
                }

            },
            _ => {}
        }
    }

    Ok(())
}