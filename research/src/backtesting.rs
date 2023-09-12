use std::fs::File;
use std::io::{BufReader, Read};
use std::error::Error;
use csv;


// Parses a 16 byte chunk of binary price data.
// The first 8 bytes are the timestamp, in milliseconds since the epoch.
// The next 4 bytes are the bid price, as a f32 (little endian).
// The last 4 bytes are the ask price, as a f32 (little endian).
pub fn parse_chunk(chunk: &[u8], is_little_endian: bool) -> Result<(u64, f32, f32), Box<dyn Error>> {
    let timestamp_bytes: [u8; 8] = chunk[0..8].try_into()?;    
    let bid_bytes: [u8; 4] = chunk[8..12].try_into()?;
    let ask_bytes: [u8; 4] = chunk[12..16].try_into()?;

    let timestamp: u64;
    let bid: f32;
    let ask: f32;
    
    if is_little_endian{
        timestamp = u64::from_le_bytes(timestamp_bytes);
        bid = f32::from_le_bytes(bid_bytes);
        ask = f32::from_le_bytes(ask_bytes);
    }
    else{
        timestamp = u64::from_be_bytes(timestamp_bytes);
        bid = f32::from_be_bytes(bid_bytes);
        ask = f32::from_be_bytes(ask_bytes);
    }
    
    Ok((timestamp, bid, ask))
}
#[derive(Debug)]
pub struct HistoricalPrice {
    pub time: u64,
    pub bid: f32,
    pub ask: f32,
}

pub struct HistoricalPriceStream {
    file: BufReader<File>,
    buffer: [u8; 16],
    is_little_endian: bool,
}

impl HistoricalPriceStream {
    pub fn new(path: &str, is_little_endian: bool) -> Result<HistoricalPriceStream, Box<dyn Error>> {
        let file = File::open(path)?;
        let file = BufReader::new(file);
        let buffer: [u8; 16] = [0; 16];
        Ok(HistoricalPriceStream { file, buffer, is_little_endian })
    }
}

// Implement Iterator trait for HistoricalPriceStream
impl Iterator for HistoricalPriceStream {
    type Item = HistoricalPrice;
    fn next(&mut self) -> Option<Self::Item> {
        match self.file.read_exact(&mut self.buffer) {
            Ok(_) => {
                match parse_chunk(&self.buffer, self.is_little_endian) {
                    Ok((timestamp, bid, ask)) => {
                        let price = HistoricalPrice {
                            time: timestamp,
                            bid: bid.into(),
                            ask: ask.into(),
                        };
                        Some(price)
                    },
                    Err(e) => {
                        println!("Error: {}", e);
                        None
                    },
                }
            },
            Err(_) => None,
        }
    }
}

pub struct Signal {
    pub forecast: f64,
}
pub trait AlphaModel {
    fn tick(&mut self, price: &HistoricalPrice) -> Result<Option<Signal>, Box<dyn Error>>;

    // This row should only contain 'special' values, such as moving averages, etc.
    // timestamp, bid, ask, and signal are automatically added
    fn generate_row(&self, price: &HistoricalPrice) -> Result<Vec<BacktestMetric>, Box<dyn Error>>;
    fn generate_headers(&self) -> Result<Vec<String>, Box<dyn Error>>;
}

pub struct ExponentialMovingAverage {
    slow_ma_weight: f64,
    fast_ma_weight: f64,

    slow_ma: f64,
    fast_ma: f64,

    direction: i32
}

impl ExponentialMovingAverage {
    pub fn new(slow_ma_weight: f64, fast_ma_weight: f64) -> Self {
        ExponentialMovingAverage {
            slow_ma_weight,
            fast_ma_weight,
            slow_ma: -1.0,
            fast_ma: -1.0,
            direction: 0,
        }
    }
}

impl AlphaModel for ExponentialMovingAverage {
    fn tick(&mut self, price: &HistoricalPrice) -> Result<Option<Signal>, Box<dyn Error>> {
        let mut signal = None;

        // If we don't have a slow or fast moving average yet, set them to the current price
        if self.slow_ma == -1.0 {
            self.slow_ma = price.ask as f64;
            self.fast_ma = price.ask as f64;
        }

        // Calculate the new moving averages
        let new_slow_ma = self.slow_ma_weight * price.ask as f64 + (1.0 - self.slow_ma_weight) * self.slow_ma;
        let new_fast_ma = self.fast_ma_weight * price.ask as f64 + (1.0 - self.fast_ma_weight) * self.fast_ma;

        // Direction is 1 if the fast moving average is above the slow moving average, -1 if the fast moving average is below the slow moving average
        let new_direction = if new_fast_ma > new_slow_ma { 1 } else { -1 };

        // If the direction has changed, generate a signal
        if new_direction != self.direction {
            signal = Some(Signal { forecast: new_direction as f64 });
        }

        self.slow_ma = new_slow_ma;
        self.fast_ma = new_fast_ma;
        self.direction = new_direction;
        Ok(signal)
    }

    // Report functions
    fn generate_row(&self, price: &HistoricalPrice) -> Result<Vec<BacktestMetric>, Box<dyn Error>> {
        Ok(vec![
            BacktestMetric::Float(self.slow_ma),
            BacktestMetric::Float(self.fast_ma),
        ])
    }

    fn generate_headers(&self) -> Result<Vec<String>, Box<dyn Error>> {
        Ok(vec![
            "slow_ma".to_string(),
            "fast_ma".to_string(),
        ])
    }

}

pub enum BacktestMetric {
    Float(f64),
    Int(i64),
    String(String),
    Bool(bool)
}
pub struct Row {
    pub data: Vec<BacktestMetric>
}
pub struct Backtest {
    // Input parameters
    pub alpha_model: Box<dyn AlphaModel>,
    pub initial_balance: f64,

    // Primary variables
    pub balance: f64,
    pub position_size: f64,

    // Secondary variables (metrics)
    pub max_balance: f64,
    pub max_drawdown: f64,
    pub trade_count: u64,
    pub profit: f64,

    // Results are stored as rows of data corresponding to each tick
    // All models include the following columns:
    // - timestamp
    // - bid
    // - ask
    // - value
    // - cash
    // - position
    // - signal
    // The rest of the columns are model-specific,
    // and should be generated by the generate_row() function and defined in the generate_header() function
    pub results: Vec<Row>,
}

impl Backtest {
    pub fn new(alpha_model: Box<dyn AlphaModel>, balance: f64) -> Self {
        Backtest {
            alpha_model,
            balance,
            initial_balance: balance,
            max_balance: balance,
            position_size: 0.0,
            max_drawdown: 0.0,
            trade_count: 0,
            profit: 0.0,
            results: Vec::new(),
        }
    }

    // Primary backtesting entry points

    pub fn tick(&mut self, price: &HistoricalPrice) -> Result<(), Box<dyn Error>> {
        // Pass the price to the alpha model
        let signal = self.alpha_model.tick(price)?;

        // Handle the signal
        self.handle_signal(&signal, price);

        // Update the backtest metrics
        self.update(price);

        // Generate a row of data for the current tick, and add it to the results
        let row: Row = self.generate_row(&signal, price)?;
        self.results.push(row);

        Ok(())
    }

    pub fn run(&mut self, price_stream: HistoricalPriceStream) -> Result<(), Box<dyn Error>> {
        for price in price_stream {
            self.tick(&price)?;
        }
        Ok(())
    }
    // Trading simulation functions

    fn handle_signal(&mut self, signal: &Option<Signal>, price: &HistoricalPrice) {
        // If there is no signal, do nothing
        let signal = match signal {
            Some(signal) => signal,
            None => return,
        };

        // If the signal is 0, do nothing
        if signal.forecast == 0.0 {
            return;
        }

        // If the signal is positive, buy according to the signal confidence
        if signal.forecast > 0.0 {
            if self.position_size == 0.0 {
                self.buy(signal.forecast, price);
            }
        }
        
        // If the signal is negative, sell according to the signal confidence
        else {
            if self.position_size > 0.0 {
                self.sell(-signal.forecast, price);
            }
        }
    }

    pub fn buy(&mut self, portion: f64, price: &HistoricalPrice) {
        self.position_size = Backtest::usd_to_units(self.balance * portion, price);
        self.balance = self.balance * (1.0 - portion);
        self.trade_count += 1;
    }

    pub fn sell(&mut self, portion: f64, price: &HistoricalPrice) {
        self.balance = self.balance + Backtest::units_to_usd(self.position_size * portion, price);
        self.position_size = self.position_size * (1.0 - portion);
        self.trade_count += 1;
    }

    // Metrics functions

    pub fn cash_value(&self) -> f64 {
        self.balance
    }

    pub fn position_value(&self, price: &HistoricalPrice) -> f64 {
        Backtest::units_to_usd(self.position_size, price)
    }

    pub fn total_value(&self, price: &HistoricalPrice) -> f64 {
        self.balance + Backtest::units_to_usd(self.position_size, price)
    }

    pub fn update_profit(&mut self, price: &HistoricalPrice) -> f64 {
        let profit = self.total_value(price) - self.initial_balance;
        self.profit = profit;
        profit
    }

    // Helper functions

    fn generate_row(&self, signal: &Option<Signal>, price: &HistoricalPrice) -> Result<Row, Box<dyn Error>> {
        let mut row = Row {
            data: Vec::new(),
        };

        // Convert the signal to a float with a default value of 0.0
        let signal_value = match signal {
            Some(signal) => signal.forecast,
            None => 0.0,
        };

        // Add the default columns
        row.data.push(BacktestMetric::Float(price.time as f64));
        row.data.push(BacktestMetric::Float(price.bid as f64));
        row.data.push(BacktestMetric::Float(price.ask as f64));
        row.data.push(BacktestMetric::Float(self.total_value(price)));
        row.data.push(BacktestMetric::Float(self.cash_value()));
        row.data.push(BacktestMetric::Float(self.position_value(price)));
        row.data.push(BacktestMetric::Float(signal_value));

        // Add the model-specific columns
        let model_row = self.alpha_model.generate_row(price)?;
        row.data.extend(model_row);

        Ok(row)
    }

    fn generate_headers(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let mut headers = vec![
            "timestamp".to_string(),
            "bid".to_string(),
            "ask".to_string(),
            "value".to_string(),
            "cash".to_string(),
            "position".to_string(),
            "signal".to_string()
            ];

        headers.extend(self.alpha_model.generate_headers()?);
        Ok(headers)
    }
    fn usd_to_units(quantity: f64, price: &HistoricalPrice) -> f64 {
        quantity / price.ask as f64
    }

    fn units_to_usd(quantity: f64, price: &HistoricalPrice) -> f64 {
        quantity * price.bid as f64
    }

    fn update(&mut self, price: &HistoricalPrice) {
        // Calculate total value
        let total_value = self.total_value(price);

        // Update max balance
        if total_value > self.max_balance {
            self.max_balance = total_value;
        }

        // Update max drawdown
        let drawdown = 1.0 - total_value / self.max_balance;
        if drawdown > self.max_drawdown {
            self.max_drawdown = drawdown;
        }

        // Update profit
        self.update_profit(price);
    }

    pub fn save_report(&mut self, output: &str) -> Result<(), Box<dyn Error>> {
        let file = File::create(output)?;
        let mut writer = csv::Writer::from_writer(file);

        // Write the headers to the top of the file
        let headers = self.generate_headers()?;
        writer.write_record(headers)?;

        for row in self.results.iter() {
            let mut record = Vec::new();
            for metric in row.data.iter() {
                match metric {
                    BacktestMetric::Float(value) => record.push(value.to_string()),
                    BacktestMetric::Int(value) => record.push(value.to_string()),
                    BacktestMetric::String(value) => record.push(value.to_string()),
                    BacktestMetric::Bool(value) => record.push(value.to_string()),
                }
            }
            writer.write_record(record)?;
        }

        writer.flush()?;
        Ok(())
    }
}
