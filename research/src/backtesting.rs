use anyhow::Result;
use quantlib::oanda::objects::Price;
use std::fs::File;
use std::io::{prelude::*, BufReader};

// Helper function to interpret a slice of bytes as a Price object
pub fn bytes_to_price(bytes: &[u8]) -> Result<Price> {
    let timestamp_bytes: [u8; 8] = bytes[0..8].try_into()?;
    let timestamp = u64::from_le_bytes(timestamp_bytes);

    let bid_bytes: [u8; 4] = bytes[8..12].try_into()?;
    let bid = f32::from_le_bytes(bid_bytes);

    let ask_bytes: [u8; 4] = bytes[12..16].try_into()?;
    let ask = f32::from_le_bytes(ask_bytes);

    let price = Price {
        bid,
        ask,
        time: timestamp,
        instrument: "".to_string(),
    };

    Ok(price)
}

// TODO: Maybe this should be part of quantlib?
pub fn load_data(data_path: &str) -> Result<Vec<Price>> {
    // Open the data file, parse each line, make a Price object, and return a Vec<Price>
    let mut prices = vec![];
    let file = File::open(data_path)?;
    let mut reader = BufReader::new(file);

    // Get the size of the file and calculate the number of data points
    let file_size = reader.seek(std::io::SeekFrom::End(0))? as usize;
    const PRICE_SIZE: usize = 16;
    let num_data_points = file_size / PRICE_SIZE;
    reader.seek(std::io::SeekFrom::Start(0))?;

    println!("Num data points: {}", num_data_points);

    // Read 16 bytes at a time, and parse them into a Price object
    // The first 8 bytes are the timestamp, the next 4 bytes are the bid, and the last 4 bytes are the ask
    // TODO: Consider reading the instrument name from the file, and using it to create the Price object
    let mut buffer: [u8; 16] = [0; 16];
    for _ in 0..num_data_points {
        match reader.read_exact(&mut buffer) {
            Ok(_) => {
                let price = bytes_to_price(&buffer)?;
                prices.push(price);
            }

            // TODO: If the file is not a multiple of 16, the last read will fail, but we should still return the prices we have
            Err(_) => return Err(anyhow::anyhow!("Error reading file")),
        }
    }

    Ok(prices)
}

pub struct RandomStrategy {}

impl RandomStrategy {
    pub fn new() -> Self {
        Self {}
    }

    pub fn signal(&self, _price: &Price) -> f32 {
        // 10% chance of buying, 10% chance of selling, 80% chance of doing nothing
        let random_number = rand::random::<f32>();
        if random_number < 0.1 {
            1.0
        } else if random_number < 0.2 {
            -1.0
        } else {
            0.0
        }
    }
}

pub struct EMAStrategy {
    fast_ema: f32,
    slow_ema: f32,

    fast_ema_weight: f32,
    slow_ema_weight: f32,
}

fn calculate_ema(current_ema: f32, data_point: f32, weight: f32) -> f32 {
    (1.0 - weight) * current_ema + weight * data_point
}

impl EMAStrategy {
    pub fn new(
        fast_ema_initial_value: f32,
        slow_ema_initial_value: f32,
        fast_ema_weight: f32,
        slow_ema_weight: f32,
    ) -> Self {
        Self {
            fast_ema: fast_ema_initial_value,
            slow_ema: slow_ema_initial_value,
            fast_ema_weight,
            slow_ema_weight,
        }
    }

    pub fn signal(&mut self, price: &Price) -> f32 {
        let new_fast_ema = calculate_ema(self.fast_ema, price.bid, self.fast_ema_weight);
        let new_slow_ema = calculate_ema(self.slow_ema, price.bid, self.slow_ema_weight);

        // If the fast EMA crosses above the slow EMA, buy
        // If the fast EMA crosses below the slow EMA, sell
        // Otherwise, do nothing
        if new_fast_ema > new_slow_ema && self.fast_ema < self.slow_ema {
            self.fast_ema = new_fast_ema;
            self.slow_ema = new_slow_ema;
            1.0
        } else if new_fast_ema < new_slow_ema && self.fast_ema > self.slow_ema {
            self.fast_ema = new_fast_ema;
            self.slow_ema = new_slow_ema;
            -1.0
        } else {
            self.fast_ema = new_fast_ema;
            self.slow_ema = new_slow_ema;
            0.0
        }
    }
}

pub struct BacktestReport {
    pub final_balance: f32,
    pub final_position: f32,
    pub final_value: f32,
    pub max_value: f32,
    pub min_value: f32,
    pub num_trades: u32,

    pub rows: Vec<Vec<String>>,
}

impl BacktestReport {
    pub fn save_to_csv(&self, path: &str) -> Result<()> {
        let mut wtr = csv::Writer::from_writer(std::io::BufWriter::new(File::create(path)?));
        for row in &self.rows {
            wtr.write_record(row)?;
        }
        wtr.flush()?;
        Ok(())
    }
}

// Takes in a balance, position, signal, and price, and returns the new balance and position
// A positive signal represents fraction of the balance to spend
// A negative signal represents fraction of the position to sell
// A signal of 0 represents doing nothing
pub fn handle_signal(balance: f32, position: f32, signal: f32, price: &Price) -> (f32, f32) {
    if signal > 0.0 {
        // Buy
        let units = (signal * balance) / price.ask;
        let new_balance = (1.0 - signal) * balance;
        let new_position = position + units;
        (new_balance, new_position)
    } else if signal < 0.0 {
        // Sell
        let value = signal.abs() * position * price.bid;
        let new_balance = balance + value;
        let new_position = (1.0 - signal.abs()) * position;
        (new_balance, new_position)
    } else {
        // Do nothing
        (balance, position)
    }
}

// Helper function to calculate the value of the portfolio
pub fn calculate_value(balance: f32, position: f32, price: &Price) -> f32 {
    balance + (position * price.bid)
}

pub fn backtest(data_set: &Vec<Price>, strategy: &mut EMAStrategy) -> Result<BacktestReport> {
    let mut balance = 1000.0; // Doesn't really matter where we start, its all percentagewise anyway
    let mut position = 0.0;

    // Metrics to track
    let mut num_trades = 0;
    let mut max_value = balance;
    let mut min_value = balance;

    // Verbose logging
    let mut rows = vec![vec![
        "Time".to_string(),
        "Balance".to_string(),
        "Position".to_string(),
        "Value".to_string(),
        "Signal".to_string(),
    ]];

    // Run the strategy on the data set
    for price in data_set {
        let signal = strategy.signal(price);

        // If the signal is 1, buy, if it is -1, sell
        (balance, position) = handle_signal(balance, position, signal, price);

        // Record metrics
        if signal != 0.0 {
            num_trades += 1;
        }
        let value = calculate_value(balance, position, price);
        if value > max_value {
            max_value = value;
        }
        if value < min_value {
            min_value = value;
        }

        // Verbose logging
        let row = vec![
            price.time.to_string(),
            balance.to_string(),
            position.to_string(),
            value.to_string(),
            signal.to_string(),
        ];
        rows.push(row);
    }

    let final_value = calculate_value(balance, position, &data_set[data_set.len() - 1]);
    Ok(BacktestReport {
        final_balance: balance,
        final_position: position,
        final_value,
        max_value,
        min_value,
        num_trades,
        rows,
    })
}
