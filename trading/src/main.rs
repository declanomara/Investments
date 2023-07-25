use std::error::Error;
use quantlib;
use quantlib::models::AlphaModel;
use quantlib::models::PortfolioBuilder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let settings = quantlib::util::read_settings()?;
    let price_stream = quantlib::oanda::PriceStream::new(&["EUR_USD".to_string()], &settings.oanda).await;
    let mut portfolio_builder = PortfolioBuilder::new(&settings);

    let mut sma = quantlib::models::SimpleMovingAverage::new("EUR_USD".to_string(), 10);

    for item in price_stream {
        // Match on the item to see what kind of stream item it is, if it's a price, print it out, otherwise ignore it
        match item {
            Ok(quantlib::oanda::StreamItem::Price(price)) => {
                println!("{}: {}", price.instrument, price.closeout_bid);
                let signal = sma.tick(&price)?;
                match signal {
                    Some(signal) => {
                        println!("{}: {}", signal.instrument, signal.forecast);
                        portfolio_builder.handle_signal(signal).await?;
                    },
                    None => {}
                }
            },
            _ => {}
        }
    }

    Ok(())
}