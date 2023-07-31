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

fn generate_report(slow_ma_weight: f32, fast_ma_weight: f32) -> Result<(), Box<dyn Error>> {
    let mut backtest = create_backtest(slow_ma_weight, fast_ma_weight);
    let price_stream = backtesting::HistoricalPriceStream::new(DATA_SET)?;

    backtest.generate_report(price_stream, "report.csv".to_string());

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let (mut slow_ma, mut fast_ma) = (0.00003469167, 0.00005774755);
    let mut best_profit = run_experiment(slow_ma, fast_ma).profit;

    // generate_report(slow_ma, fast_ma);

    let mut count = 0;

    loop {
        let (new_slow_ma, new_fast_ma) = adjust_weights(slow_ma, fast_ma);
        let backtest = run_experiment(new_slow_ma, new_fast_ma);
        if backtest.profit > best_profit {
            println!("New best parameters: ({}, {})", new_slow_ma, new_fast_ma);
            print_results(&backtest);

            if GENERATE_REPORTS {
                match generate_report(new_slow_ma, new_fast_ma) {
                    Ok(_) => println!("Report generated"),
                    Err(e) => println!("Error generating report: {}", e),
                }
            }

            slow_ma = new_slow_ma;
            fast_ma = new_fast_ma;
            best_profit = backtest.profit;
        }

        count += 1;
        if count % 100 == 0 {
            println!("{} iterations", count);
            println!("Best profit: {}", best_profit);
        }
    }

    Ok(())
}