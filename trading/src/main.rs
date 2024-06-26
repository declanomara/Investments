use quantlib::oanda::{PriceStream, FastPriceStream};
use quantlib::models::{AlphaModel, PortfolioBuilder};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let settings = quantlib::util::read_settings()?;
    let instruments = vec!["EUR_USD".to_string()].to_vec();
    let price_stream: FastPriceStream  = FastPriceStream::new(
        instruments,
        &settings.oanda,
        1000,
    );
    
    let mut portfolio_builder = PortfolioBuilder::new(&settings);
    portfolio_builder.update_positions().await?; // TODO: this should be done automatically by the portfolio builder
    let mut ema = quantlib::models::ExponentialMovingAverage::new("EUR_USD".to_string(), 0.1, 0.2);

    for item in price_stream {
        // Match on the item to see what kind of stream item it is, if it's a price, print it out, otherwise ignore it
        match item {
            Ok(quantlib::oanda::objects::StreamItem::Price(price)) => {
                println!(
                    "[{}][PRICE] Bid: {:.5} Ask: {:.5}",
                    price.instrument, price.bid, price.ask
                );
                let signal = ema.tick(&price)?;
                match signal {
                    Some(signal) => {
                        println!(
                            "[{}][SIGNAL] Forecast: {}",
                            signal.instrument, signal.forecast
                        );
                        portfolio_builder.handle_signal(signal).await?;
                    }
                    None => {}
                }
            }
            _ => {}
        }
    }

    Ok(())
}
