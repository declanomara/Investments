// main.rs
use std::error::Error;
// use quantlib::models::{AlphaModel, ExponentialMovingAverage};
use backtesting::{AlphaModel, ExponentialMovingAverage};

mod backtesting;
mod data_cleaning;

const INITIAL_BALANCE: f64 = 100_000.0;
const DATA_SET: &str = "data/bin/EUR_USD.bin";
const DATA_SETS: [&str; 6] = [
    "historical-data/weekly/EUR_USD/week-21.bin",
    "historical-data/weekly/EUR_USD/week-20.bin",
    "historical-data/weekly/EUR_USD/week-22.bin",
    "historical-data/weekly/EUR_USD/week-23.bin",
    "historical-data/weekly/EUR_USD/week-24.bin",
    "historical-data/weekly/EUR_USD/week-25.bin",
];

fn main() -> Result<(), Box<dyn Error>> {
    let i = 0;
    let price_stream = backtesting::HistoricalPriceStream::new(DATA_SETS[i], true)?;
    let strategy: Box<dyn AlphaModel> = Box::new(ExponentialMovingAverage::new(0.1, 0.2));
    let mut backtest = backtesting::Backtest::new(strategy, INITIAL_BALANCE);
    backtest.run(price_stream)?;
    
    // Print the last row in the backtest's results
    let last_row = backtest.results.last().unwrap();
    println!("{last_row}");
    Ok(())
}
