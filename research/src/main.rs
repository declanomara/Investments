// main.rs

use std::error::Error;
use rand::Rng;

mod backtesting;
mod data_cleaning;

const GENERATE_REPORTS: bool = true;
const ADJUSTMENT_PCNT: f32 = 0.9;
const INITIAL_BALANCE: f64 = 100_000.0;
const DATA_SET: &str = "data/weekly/EUR_USD/week-20.bin";
const DATA_SETS: [&str; 6] = [
    "data/weekly/EUR_USD/week-20.bin",
    "data/weekly/EUR_USD/week-21.bin",
    "data/weekly/EUR_USD/week-22.bin",
    "data/weekly/EUR_USD/week-23.bin",
    "data/weekly/EUR_USD/week-24.bin",
    "data/weekly/EUR_USD/week-25.bin",
];

fn adjust_weights(slow_ma_weight: f32, fast_ma_weight: f32) -> (f32, f32) {
    // Increase or decrease slow_ma_weight and fast_ma_weight by a random amount up to ADJUSTMENT_PCNT
    // If the new weights are out of bounds, try again

    let mut rng = rand::thread_rng();

    loop {
        let slow_ma_adjustment_factor = rng.gen_range(-ADJUSTMENT_PCNT..=ADJUSTMENT_PCNT);
        let fast_ma_adjustment_factor = rng.gen_range(-ADJUSTMENT_PCNT..=ADJUSTMENT_PCNT);

        let new_slow_ma_weight = slow_ma_weight + slow_ma_weight * slow_ma_adjustment_factor;
        let new_fast_ma_weight = fast_ma_weight + fast_ma_weight * fast_ma_adjustment_factor;

        if 0.0 < new_slow_ma_weight && new_slow_ma_weight < new_fast_ma_weight && new_fast_ma_weight < 1.0 {
            return (new_slow_ma_weight, new_fast_ma_weight);
        }
    }

}

fn create_backtest(slow_ma_weight: f32, fast_ma_weight: f32) -> backtesting::Backtest {
    let alpha_model = Box::new(
        backtesting::ExponentialMovingAverage::new(slow_ma_weight as f64, fast_ma_weight as f64)
    );

    backtesting::Backtest::new(alpha_model, INITIAL_BALANCE)
}

// Print the profit and max balance for a backtest
fn print_results(backtest: &backtesting::Backtest) {
    println!("Profit: {}", backtest.profit);
    println!("Max Balance: {}", backtest.max_balance);
    println!("Max Drawdown: {}", backtest.max_drawdown);
    println!("Trade Count: {}", backtest.trade_count);
}

fn run_experiment(slow_ma_weight: f32, fast_ma_weight: f32) -> backtesting::Backtest {
        let mut backtest = create_backtest(slow_ma_weight, fast_ma_weight);
        let price_stream = backtesting::HistoricalPriceStream::new(DATA_SET).unwrap();
        for price in price_stream {
            backtest.tick(&price).unwrap();
        }

        // for data_set in DATA_SETS.iter() {
        //     let price_stream = backtesting::HistoricalPriceStream::new(data_set).unwrap();

        //     let mut prev_price = backtesting::HistoricalPrice {
        //         time: 0,
        //         bid: 0.0,
        //         ask: 0.0,
        //     };

        //     for price in price_stream {
        //         backtest.tick(&price).unwrap();
        //         prev_price = price;
        //     }

        //     // Sell at the end of the week
        //     backtest.sell(1.0, &prev_price);
        // }
        
        backtest
}

fn main() -> Result<(), Box<dyn Error>> {
    let (mut slow_ma, mut fast_ma) = (0.00003469167, 0.00005774755);
    
    let price_stream = backtesting::HistoricalPriceStream::new(DATA_SET)?;
    let mut backtest = create_backtest(slow_ma, fast_ma);

    print!("Running backtest with slow_ma_weight: {}, fast_ma_weight: {} on data set {}... ", slow_ma, fast_ma, DATA_SET);
    backtest.run(price_stream)?;
    println!("Done!");

    print_results(&backtest);

    print!("Saving results to file... ");
    backtest.save_report("results.csv")?;
    println!("Done!");

    Ok(())
}