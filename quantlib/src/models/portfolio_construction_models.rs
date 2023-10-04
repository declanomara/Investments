use crate::oanda;
use crate::oanda::objects::{Settings, Position};
use crate::models::TradingSignal;


// The portfolio construction model takes in a collection of trading signals, determines desired position sizes,
// and returns a collection of trades to be executed by the execution model.
// Currently, trades are simple enough that the portfolio construction model can just place them directly.

pub struct PortfolioBuilder<'a> {
    settings: &'a Settings,
    positions: Vec<Position>
}

impl<'a> PortfolioBuilder<'a> {
    pub fn new(settings: &'a Settings) -> Self {
        PortfolioBuilder {
            settings,
            positions: Vec::new()
        }
        // TODO: initialize positions
    }

    // Update the positions held by the portfolio builder to reflect the current state of the account
    pub async fn update_positions(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.positions = oanda::get_positions(&self.settings.oanda).await?;
        Ok(())
    }

    // Given a trading signal, determine the desired position size and either buy or sell to reach that position
    // TODO: in the future, this should produce a trade to be executed by the execution model
    // TODO: in the future, this should account for confidence in the signal
    pub async fn handle_signal(&mut self, signal: TradingSignal) -> Result<(), Box<dyn std::error::Error>> {
        let current_position = self.positions.iter().find(|p| p.instrument == signal.instrument);
        if let Some(position) = current_position {
            // Determine the desired position size
            let desired_position = if signal.forecast > 0.0 {
                self.settings.units
            } else if signal.forecast < 0.0 {
                -self.settings.units
            } else {
                0.0
            };

            println!("Desired position: {}", desired_position);
            println!("Current position: {}", position.units());

            // Determine the required changes to the current position to reach the desired position
            let required_units = desired_position - position.units();
            println!("Required units: {}", required_units);

            // If no changes are required, do nothing
            // This automatically handles the case where the desired position is 0.0,
            // as well as preventing repeated signals from increasing the position size
            if required_units == 0.0 {
                return Ok(());
            }
            else {
                oanda::place_market_order(&signal.instrument, required_units, &self.settings.oanda).await?;
            }

        } else {
            // If no position exists, open a new position
            if signal.forecast > 0.0 {
                oanda::place_market_order(&signal.instrument, self.settings.units, &self.settings.oanda).await?;
            } else {
                oanda::place_market_order(&signal.instrument, -self.settings.units, &self.settings.oanda).await?;
            }
        }

        // Update the positions held by the portfolio builder to reflect the current state of the account
        self.update_positions().await?;
        Ok(())
    }

    // Given a collection of trading signals, determine the desired position sizes and either buy or sell to reach those positions
    pub async fn handle_signals(&mut self, signals: Vec<TradingSignal>) -> Result<(), Box<dyn std::error::Error>> {
        for signal in signals {
            self.handle_signal(signal).await?;
        }
        Ok(())
    }
}
