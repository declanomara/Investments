use quantlib::models::{create_model_from_config, read_config, AlphaModel};
use quantlib::oanda::{FastPriceStream, PriceStream};
use quantlib::util::read_settings;
use std::env;
use std::error::Error;

// TODO: THIS NO LONGER EXECUTES TRADES, IT JUST PRINTS OUT THE SIGNALS

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <config>", args[0]);
        std::process::exit(1);
    }

    let settings = read_settings()?;
    let config = read_config(&args[1]);
    let instruments = &config.instruments;
    let price_stream: FastPriceStream =
        FastPriceStream::new(instruments.clone(), &settings.oanda, 1000);

    let mut strategy = create_model_from_config(&config);

    for item in price_stream {
        // Match on the item to see what kind of stream item it is, if it's a price, print it out, otherwise ignore it
        match item {
            Ok(quantlib::oanda::objects::StreamItem::Price(price)) => {
                println!(
                    "[{}][PRICE] Bid: {:.5} Ask: {:.5}",
                    price.instrument, price.bid, price.ask
                );
                let signal = strategy.tick(&price);
                match signal {
                    Some(signal) => {
                        println!("[{}][SIGNAL] Forecast: {}", price.instrument, signal.amount);
                    }
                    None => {}
                }
            }
            _ => {}
        }
    }

    Ok(())
}
