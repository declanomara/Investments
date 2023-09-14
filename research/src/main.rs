// main.rs

use rand::Rng;
use std::error::Error;

mod backtesting;
mod data_cleaning;

const INITIAL_BALANCE: f64 = 100_000.0;
const DATA_SET: &str = "data/bin/EUR_USD.bin";
const DATA_SETS: [&str; 6] = [
    "data/weekly/EUR_USD/week-20.bin",
    "data/weekly/EUR_USD/week-21.bin",
    "data/weekly/EUR_USD/week-22.bin",
    "data/weekly/EUR_USD/week-23.bin",
    "data/weekly/EUR_USD/week-24.bin",
    "data/weekly/EUR_USD/week-25.bin",
];

fn create_backtest(slow_ma_weight: f32, fast_ma_weight: f32) -> backtesting::Backtest {
    let alpha_model = Box::new(backtesting::ExponentialMovingAverage::new(
        slow_ma_weight as f64,
        fast_ma_weight as f64,
    ));

    backtesting::Backtest::new(alpha_model, INITIAL_BALANCE)
}

// Print the profit and max balance for a backtest
fn print_results(backtest: &backtesting::Backtest) {
    println!("Profit: {}", backtest.profit);
    println!("Max Balance: {}", backtest.max_balance);
    println!("Max Drawdown: {}", backtest.max_drawdown);
    println!("Trade Count: {}", backtest.trade_count);
}

fn main() -> Result<(), Box<dyn Error>> {
    // let (mut slow_ma, mut fast_ma) = (0.00003469167, 0.00005774755);

    // let price_stream = backtesting::HistoricalPriceStream::new(DATA_SET)?;
    // let mut backtest = create_backtest(slow_ma, fast_ma);

    // print!("Running backtest with slow_ma_weight: {}, fast_ma_weight: {} on data set {}... ", slow_ma, fast_ma, DATA_SET);
    // backtest.run(price_stream)?;
    // println!("Done!");

    // print_results(&backtest);

    // print!("Saving results to file... ");
    // backtest.save_report("results.csv")?;
    // println!("Done!");

    let price_stream = backtesting::HistoricalPriceStream::new(DATA_SET, false)?;
    for price in price_stream {
        println!("{:?}", price);
    }

    Ok(())
}
