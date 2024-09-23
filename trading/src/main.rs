use quantlib::models::{AlphaModel, PortfolioBuilder};
use quantlib::oanda::{FastPriceStream, PriceStream};
use quantlib::util::{read_settings, TradingConfig};
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <config>", args[0]);
        std::process::exit(1);
    }

    let settings = read_settings()?;
    let config = TradingConfig::load(args.remove(1))?;
    let instruments = config.instruments;
    let price_stream: FastPriceStream = FastPriceStream::new(instruments, &settings.oanda, 1000);

    let mut portfolio_builder = PortfolioBuilder::new(&settings);
    portfolio_builder.update_positions().await?; // TODO: this should be done automatically by the portfolio builder

    // Strategies must be "registered" here in order to be used
    let mut strategy = match config.model.as_str() {
        "random" => quantlib::models::RandomStrategy::from(
            quantlib::models::RandomStrategyConfig::from(config.model_config),
        ),
        _ => panic!("Unknown model: {}", config.model),
    };

    for item in price_stream {
        // Match on the item to see what kind of stream item it is, if it's a price, print it out, otherwise ignore it
        match item {
            Ok(quantlib::oanda::objects::StreamItem::Price(price)) => {
                println!(
                    "[{}][PRICE] Bid: {:.5} Ask: {:.5}",
                    price.instrument, price.bid, price.ask
                );
                let signal = strategy.tick(&price)?;
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
