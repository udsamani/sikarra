use rust_decimal::Decimal;
use tracing::{info, warn};

use crate::{
    config::MarketMakingConfig,
    engine::{InternalEvent, MarketCondition, PoolSymbol},
    strategy::{market_making::MarketMakingSimulator, BotStrategy},
};

/// A simple logging arbitrage strategy that logs if an arbitrage opportunity
/// exists and simulates market making ranges
pub struct LoggingBotStrategy {
    symbol: PoolSymbol,
    last_cex_price: Option<Decimal>,
    last_dex_price: Option<Decimal>,
    simulator: MarketMakingSimulator,
}

impl LoggingBotStrategy {
    pub fn new(symbol: PoolSymbol, config: MarketMakingConfig) -> Self {
        let simulator = MarketMakingSimulator::new(symbol.clone(), config);
        Self { symbol, last_cex_price: None, last_dex_price: None, simulator }
    }

    /// Check for arbitrage opportunities and run market making simulation
    fn check_arbitrage_and_simulate_mm(&self) {
        if let (Some(cex_price), Some(dex_price)) = (self.last_cex_price, self.last_dex_price) {
            // 1. Check for simple arbitrage opportunities
            self.log_arbitrage_opportunity(cex_price, dex_price);

            // 2. Run market making simulation
            self.run_market_making_simulation(cex_price, dex_price);
        }
    }

    /// Log arbitrage opportunities
    #[allow(clippy::comparison_chain)]
    fn log_arbitrage_opportunity(&self, cex_price: Decimal, dex_price: Decimal) {
        let diff = (cex_price - dex_price).abs();
        let profit_pct = (diff / cex_price) * Decimal::new(100, 0);

        // Only log if there's a meaningful difference (e.g., > 0.1%)
        if profit_pct > Decimal::new(10, 2) {
            // 0.1%
            if cex_price > dex_price {
                info!(
                    "🚀 ARBITRAGE OPPORTUNITY: Buy DEX ${:.2} → Sell CEX ${:.2} | Profit: ${:.2} ({:.2}%) | Symbol: {}",
                    dex_price, cex_price, diff, profit_pct, self.symbol
                );
            } else if dex_price > cex_price {
                info!(
                    "🚀 ARBITRAGE OPPORTUNITY: Buy CEX ${:.2} → Sell DEX ${:.2} | Profit: ${:.2} ({:.2}%) | Symbol: {}",
                    cex_price, dex_price, diff, profit_pct, self.symbol
                );
            }
        }
    }

    /// Run market making simulation and log results
    fn run_market_making_simulation(&self, cex_price: Decimal, dex_price: Decimal) {
        // Calculate optimal market making ranges
        let mm_range = self.simulator.calculate_ranges(cex_price, Some(dex_price));

        // Log the simulation results
        info!("🎯 MARKET MAKING SIMULATION");
        info!(
            "Symbol: {} | Fair Value: ${:.2} (CEX) | Current DEX: ${:.2}",
            self.symbol, mm_range.fair_value, dex_price
        );

        info!(
            "Optimal Range: BID ${:.2} ({} bps) ← → ASK ${:.2} ({} bps)",
            mm_range.bid_price,
            mm_range.bid_spread_bps,
            mm_range.ask_price,
            mm_range.ask_spread_bps
        );

        info!(
            "Range Width: ${:.2} ({:.2}% of fair value)",
            mm_range.total_range_width,
            (mm_range.total_range_width / mm_range.fair_value) * Decimal::new(100, 0)
        );

        info!("Strategy Logic: {}", mm_range.reasoning);

        // Log market condition assessment
        match mm_range.market_condition {
            MarketCondition::Normal => {
                info!("📊 Market Condition: NORMAL - Symmetric spreads around fair value");
            },
            MarketCondition::Arbitrage => {
                info!("⚡ Market Condition: ARBITRAGE - Asymmetric spreads for mean reversion capture");
            },
            MarketCondition::Volatile => {
                info!("🌪️  Market Condition: VOLATILE - Wide spreads for protection");
            },
        }

        // Calculate and log potential profits for different trade sizes
        self.log_potential_profits(&mm_range);
    }

    /// Log potential profits for different trade sizes
    fn log_potential_profits(&self, mm_range: &crate::engine::MarketMakingRange) {
        let trade_sizes = [
            (Decimal::new(1, 0), format!("1 {}", self.symbol.get_base_asset())),
            (Decimal::new(5, 0), format!("5 {}", self.symbol.get_base_asset())),
            (Decimal::new(10, 0), format!("10 {}", self.symbol.get_base_asset())),
            (Decimal::new(50, 0), format!("50 {}", self.symbol.get_base_asset())),
        ];

        let mut profit_info = String::from("💰 Potential MM Profits: ");

        for (size, label) in trade_sizes.iter() {
            let profit = self.calculate_mm_profit(*size, mm_range);
            profit_info.push_str(&format!("{} = ${:.2}, ", label, profit));
        }

        // Remove trailing comma and space
        profit_info.truncate(profit_info.len() - 2);

        info!("{}", profit_info);
    }

    /// Calculate potential market making profit for a given trade size
    fn calculate_mm_profit(
        &self,
        trade_size_eth: Decimal,
        mm_range: &crate::engine::MarketMakingRange,
    ) -> Decimal {
        // Simplified calculation: average spread * trade size
        // In reality, this would depend on actual fills and market conditions
        let avg_spread = (mm_range.ask_price - mm_range.bid_price) / Decimal::new(2, 0);
        avg_spread * trade_size_eth
    }
}

impl BotStrategy for LoggingBotStrategy {
    fn handle_internal_event(&mut self, event: InternalEvent) {
        match event {
            InternalEvent::TickerUpdate(ticker) if ticker.symbol == self.symbol => {
                self.last_cex_price = Some(ticker.price);
                self.check_arbitrage_and_simulate_mm();
            },
            InternalEvent::PoolPriceUpdate(update) if update.symbol == self.symbol => {
                self.last_dex_price = Some(update.price);
                self.check_arbitrage_and_simulate_mm();
            },
            _ => {
                unreachable!("Unexpected event for LoggingBotStrategy: {:?}", event);
            },
        }
    }
}
