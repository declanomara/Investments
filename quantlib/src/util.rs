pub fn generate_timestamp() -> String {
    let now = chrono::Utc::now();
    now.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn generate_timestamp_filename() -> String {
    let now = chrono::Utc::now();
    now.format("%Y-%m-%d_%H-%M-%S").to_string()
}

use crate::objects::OrderBook;
use std::io::{stdout, Write};
use crossterm::{
    cursor::{Hide, Show},
    terminal::{Clear, ClearType},
    ExecutableCommand,
};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;

pub struct OrderBookDisplay {
    stdout: std::io::Stdout,
}

impl OrderBookDisplay {
    pub fn new() -> Self {
        Self {
            stdout: stdout(),
        }
    }

    pub fn show(&mut self, book: &OrderBook) -> std::io::Result<()> {
        // Clear the screen and hide cursor
        self.stdout.execute(Clear(ClearType::All))?;
        self.stdout.execute(Hide)?;

        // Get the top 5 bids and asks
        let top_bids: Vec<_> = book.bids.iter().take(5).collect();
        let top_asks: Vec<_> = book.asks.iter().take(5).collect();

        // Calculate max quantity for scaling
        let max_qty = top_bids.iter()
            .chain(top_asks.iter())
            .map(|(_, qty)| **qty)
            .fold(Decimal::ZERO, std::cmp::max);

        // Print header
        println!("Order Book: {}", book.symbol);
        let header = format!("\n{:─^93}", "");
        println!("{}", header);
        println!("│ {:^10} │ {:^40} │ {:^15} │ {:^15} │", "Price", "Volume", "Quantity", "Total Value");
        let separator = format!("{:─^93}", "");
        println!("{}", separator);

        // Print asks (highest to lowest)
        for (price, qty) in top_asks.iter().rev() {
            let bar_width = ((**qty / max_qty) * Decimal::from(40)).round().to_u32().unwrap_or(0);
            let bar = "█".repeat(bar_width as usize);
            let padded_bar = format!("{:<40}", bar);
            println!("│ {:>10.2} │ {} │ {:>15.8} │ {:>15.2} │", 
                price, padded_bar, qty, *price * *qty);
        }

        // Print separator between asks and bids
        println!("{}", separator);

        // Print bids (highest to lowest)
        for (price, qty) in top_bids.iter().rev() {
            let bar_width = ((**qty / max_qty) * Decimal::from(40)).round().to_u32().unwrap_or(0);
            let bar = "█".repeat(bar_width as usize);
            let padded_bar = format!("{:<40}", bar);
            println!("│ {:>10.2} │ {} │ {:>15.8} │ {:>15.2} │", 
                price, padded_bar, qty, *price * *qty);
        }

        println!("{}", separator);

        // Flush stdout to ensure immediate display
        self.stdout.flush()?;
        Ok(())
    }

    pub fn cleanup(&mut self) -> std::io::Result<()> {
        // Show cursor and clear screen
        self.stdout.execute(Show)?;
        self.stdout.execute(Clear(ClearType::All))?;
        self.stdout.flush()?;
        Ok(())
    }
}
