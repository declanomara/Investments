pub mod backtesting;
pub mod optimization;

use anyhow::Result;
use quantlib::oanda::objects::Price;
use rand::prelude::*;

const DATA_PATHS: [&str; 6] = [
    "historical-data/weekly/EUR_USD/week-20.bin",
    "historical-data/weekly/EUR_USD/week-21.bin",
    "historical-data/weekly/EUR_USD/week-22.bin",
    "historical-data/weekly/EUR_USD/week-23.bin",
    "historical-data/weekly/EUR_USD/week-24.bin",
    "historical-data/weekly/EUR_USD/week-25.bin",
];

fn main() -> Result<()> {
    // temp
    thread_rng().gen_range(0..1);

    // Load the data
    let mut data_sets: Vec<Vec<Price>> = Vec::new();
    for path in DATA_PATHS.iter() {
        let data = backtesting::load_data(path)?;
        data_sets.push(data);
    }

    // Randomize the order of the data sets
    data_sets.shuffle(&mut thread_rng());

    // Pick the first 3 data sets
    data_sets.truncate(3);

    // Run the optimization
    optimization::optimize_ema(&data_sets)?;

    Ok(())
}
