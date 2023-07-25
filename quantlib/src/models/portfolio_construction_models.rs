use crate::oanda;
use crate::models::TradingSignal;


// The portfolio construction model takes in a collection of trading signals, determines desired position sizes,
// and returns a collection of trades to be executed by the execution model.
// Currently, trades are simple enough that the portfolio construction model can just place them directly.

pub struct PortfolioBuilder<'a> {
    settings: &'a oanda::Settings,
    positions: Vec<oanda::Position>
}

impl<'a> PortfolioBuilder<'a> {
    pub fn new(settings: &'a oanda::Settings) -> Self {
        PortfolioBuilder {
            settings,
            positions: Vec::new()
        }
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
            // If signal is positive, close short position if it exists and open long position
            // If signal is negative, close long position if it exists and open short position
            if signal.forecast > 0.0 {
                if position.short.units > 0.0 {
                    oanda::place_market_order(&signal.instrument, position.short.units, &self.settings.oanda).await?;
                }
                oanda::place_market_order(&signal.instrument, self.settings.units, &self.settings.oanda).await?;
            } else if signal.forecast < 0.0 {
                if position.long.units > 0.0 {
                    oanda::place_market_order(&signal.instrument, position.long.units, &self.settings.oanda).await?;
                }
                oanda::place_market_order(&signal.instrument, -self.settings.units, &self.settings.oanda).await?;
            }
        } else {
            // If no position exists, open a new position
            if signal.forecast > 0.0 {
                oanda::place_market_order(&signal.instrument, self.settings.units, &self.settings.oanda).await?;
            } else {
                oanda::place_market_order(&signal.instrument, -self.settings.units, &self.settings.oanda).await?;
            }
        }
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
