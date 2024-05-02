#[derive(Debug)]
pub struct TradingSignal {
    pub instrument: String,
    pub forecast: f64, // 1.0 for 100% confidence in a price increase, -1.0 for 100% confidence in a price decrease
}
