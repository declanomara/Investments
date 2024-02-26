pub mod backtesting;

use anyhow::Result;

const DATA_PATH: &str = "historical-data/weekly/EUR_USD/week-20.bin";
const RESULT_PATH: &str = "result.csv";

fn main() -> Result<()> {
    // Load the data
    println!("Loading data from {}", DATA_PATH);
    let data_set = backtesting::load_data(DATA_PATH)?;

    // Load the strategy
    // let strategy = backtesting::RandomStrategy::new();

    let first_bid = data_set[0].bid;
    let slow_weight = 0.01; // Most recent data point has 10% weight
    let fast_weight = 0.013; // Most recent data point has 90% weight
    let mut strategy =
        backtesting::EMAStrategy::new(first_bid, first_bid, fast_weight, slow_weight);

    // Run the backtest
    let result = backtesting::backtest(&data_set, &mut strategy)?;

    // Print the result
    println!("Final balance: {}", result.final_balance);
    println!("Final position: {}", result.final_position);
    println!("Final value: {}", result.final_value);
    println!("Max value: {}", result.max_value);
    println!("Min value: {}", result.min_value);
    println!("Number of trades: {}", result.num_trades);

    // Save the result to a CSV file
    result.save_to_csv(RESULT_PATH)?;

    Ok(())
}
