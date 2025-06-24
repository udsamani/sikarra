use rust_decimal::Decimal;
use tracing::{info, warn};

use crate::{
    engine::{InternalEvent, PoolSymbol},
    strategy::ArbitrageStrategy,
};

/// A simple logging arbitrage strategy that logs if an arbitrage opportunity
/// exists
pub struct LogginArbitrageStrategy {
    symbol: PoolSymbol,
    last_cex_price: Option<Decimal>,
    last_dex_price: Option<Decimal>,
}

impl LogginArbitrageStrategy {
    pub fn new(symbol: PoolSymbol) -> Self {
        Self { symbol, last_cex_price: None, last_dex_price: None }
    }

    #[allow(clippy::comparison_chain)]
    fn check_arbitrage(&self) {
        if let (Some(cex_price), Some(dex_price)) = (self.last_cex_price, self.last_dex_price) {
            let diff = (cex_price - dex_price).abs();
            let profit_pct = (diff / cex_price) * Decimal::new(100, 0);

            if cex_price > dex_price {
                info!(
                    "🚀 ARBITRAGE: Buy DEX ${} → Sell CEX ${} | Profit: ${} ({:.2}%) | Symbol: {}",
                    dex_price, cex_price, diff, profit_pct, self.symbol
                );
            } else if dex_price > cex_price {
                info!(
                    "🚀 ARBITRAGE: Buy CEX ${} → Sell DEX ${} | Profit: ${} ({:.2}%) | Symbol: {}",
                    cex_price, dex_price, diff, profit_pct, self.symbol
                );
            }
        }
    }
}

impl ArbitrageStrategy for LogginArbitrageStrategy {
    fn determine_arbitrage_opportunity(&mut self, event: InternalEvent) {
        match event {
            InternalEvent::TickerUpdate(ticker) if ticker.symbol == self.symbol => {
                self.last_cex_price = Some(ticker.price);
                self.check_arbitrage();
            },
            InternalEvent::PoolPriceUpdate(update) if update.symbol == self.symbol => {
                self.last_dex_price = Some(update.price);
                self.check_arbitrage();
            },
            _ => {
                warn!("Received unexpected event for symbol: {}", self.symbol);
            },
        }
    }
}
