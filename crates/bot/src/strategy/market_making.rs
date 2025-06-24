use rust_decimal::{prelude::ToPrimitive, Decimal};
use rust_decimal_macros::dec;

use crate::{
    config::MarketMakingConfig,
    engine::{MarketCondition, MarketMakingRange, PoolSymbol},
};

/// A market making simulator calculates optimal bid/ask price ranges for
/// providing liquidity on a decentralized exchange (DEX). It uses the
/// centralized exchange (CEX) prices as a fair value references and dynamically
/// adjusts based on real-time market conditions.
///
/// # BPS: Basis Points
///
/// Basis points is a unit for measuring percentages. 1 Basis Point is equal to
/// 0.01%. We use this unit to represent spreads in a more precise manner.
///
/// # Fields
/// - `symbol`: The trading pair symbol for which the market making strategy is
///   applied (e.g., "ETH-USDC").
/// - `base_spread_bps`: The base spread in basis points used to calculate the
///   initial bid/ask prices. This is the minimum spread applied to the CEX
///   price.
/// - `max_spread_bps`: The maximum spread in basis points allowed for the
///   bid/ask prices.
/// - `min_spread_bps`: The minimum spread in basis points that can be applied
///   to the CEX price.
/// - `arbitrage_threshold_bps`: The threshold in basis points for triggering
///   arbitrage opportunities. If the price difference between CEX and DEX
/// - `gas_price`: The current gas price in the network,
#[derive(Debug, Clone)]
pub struct MarketMakingSimulator {
    pub symbol: PoolSymbol,
    pub base_spread_bps: u32,
    pub max_spread_bps: u32,
    pub min_spread_bps: u32,
    pub arbitrage_threshold_bps: u32,
    pub arbitrage_tighten_factor: Decimal,
    pub arbitrage_widen_factor: Decimal,
    pub gas_price: Decimal,
}

impl MarketMakingSimulator {
    /// Creates a new `MarketMakingSimulator` with the specified symbol and
    /// default parameters.
    pub fn new_with_default(symbol: PoolSymbol) -> Self {
        Self {
            symbol,
            base_spread_bps: 50,
            max_spread_bps: 200,
            min_spread_bps: 25,
            arbitrage_threshold_bps: 100,
            gas_price: dec!(0.5),
            arbitrage_tighten_factor: dec!(0.7),
            arbitrage_widen_factor: dec!(1.3),
        }
    }

    /// Creates a new `MarketMakingSimulator` with the specified parameters.
    pub fn new(symbol: PoolSymbol, config: MarketMakingConfig) -> Self {
        Self {
            symbol,
            base_spread_bps: config.base_spread_bps,
            max_spread_bps: config.max_spread_bps,
            min_spread_bps: config.min_spread_bps,
            arbitrage_threshold_bps: config.arbitrage_threshold_bps,
            gas_price: config.gas_price,
            arbitrage_tighten_factor: config.arbitrage_tighten_factor,
            arbitrage_widen_factor: config.arbitrage_widen_factor,
        }
    }

    /// Calculate the optimal market making ranges based on current market
    /// conditions
    pub fn calculate_ranges(
        &self,
        cex_price: Decimal,
        dex_price: Option<Decimal>,
    ) -> MarketMakingRange {
        // Assess market conditions
        let market_condition = self.assess_market_conditions(cex_price, dex_price);

        // Calculate optimal spreads
        let (bid_spread_bps, ask_spread_bps) =
            self.calculate_spreads(&market_condition, cex_price, dex_price);

        // Convert basis points to decimal for price calculations
        let bid_spread_decimal = Decimal::new(bid_spread_bps as i64, 4); // bps to decimal
        let ask_spread_decimal = Decimal::new(ask_spread_bps as i64, 4);

        // Calculate actual bid/ask prices
        let bid_price = cex_price * (Decimal::ONE - bid_spread_decimal);
        let ask_price = cex_price * (Decimal::ONE + ask_spread_decimal);
        let total_range_width = ask_price - bid_price;

        // Generate strategy reasoning
        let reasoning = self.explain_strategy(
            &market_condition,
            cex_price,
            dex_price,
            bid_spread_bps,
            ask_spread_bps,
        );

        MarketMakingRange {
            symbol: self.symbol.clone(),
            fair_value: cex_price,
            bid_price,
            ask_price,
            bid_spread_bps,
            ask_spread_bps,
            total_range_width,
            reasoning,
            market_condition,
        }
    }

    /// Assess the current market conditions
    fn assess_market_conditions(
        &self,
        cex_price: Decimal,
        dex_price: Option<Decimal>,
    ) -> MarketCondition {
        if let Some(dex) = dex_price {
            // Calculate price difference in basis points
            let price_diff_bps = ((dex - cex_price).abs() / cex_price) * Decimal::new(10000, 0);

            if price_diff_bps > Decimal::new(self.arbitrage_threshold_bps as i64, 0) {
                return MarketCondition::Arbitrage;
            }
        }
        MarketCondition::Normal
    }

    /// Calculate the optimal bid/ask spreads based on market conditions
    fn calculate_spreads(
        &self,
        condition: &MarketCondition,
        cex_price: Decimal,
        dex_price: Option<Decimal>,
    ) -> (u32, u32) {
        let mut bid_spread = self.base_spread_bps;
        let mut ask_spread = self.base_spread_bps;

        match condition {
            MarketCondition::Normal => {
                // Use base spreads - no adjustment needed
            },
            MarketCondition::Arbitrage => {
                if let Some(dex) = dex_price {
                    if dex > cex_price {
                        // DEX price higher - expect selling pressure
                        // Tighten ask (more aggressive selling), widen bid (cautious buying)
                        ask_spread = self
                            .apply_arbitrage_adjustment(ask_spread, self.arbitrage_tighten_factor);
                        bid_spread = self
                            .apply_arbitrage_adjustment(bid_spread, self.arbitrage_widen_factor);
                    } else {
                        // DEX price lower - expect buying pressure
                        // Tighten bid (more aggressive buying), widen ask (cautious selling)
                        bid_spread = self
                            .apply_arbitrage_adjustment(bid_spread, self.arbitrage_tighten_factor);
                        ask_spread = self
                            .apply_arbitrage_adjustment(ask_spread, self.arbitrage_widen_factor);
                    }
                }
            },
            MarketCondition::Volatile => {},
        }

        // Apply bounds
        bid_spread = bid_spread.clamp(self.min_spread_bps, self.max_spread_bps);
        ask_spread = ask_spread.clamp(self.min_spread_bps, self.max_spread_bps);

        (bid_spread, ask_spread)
    }

    /// Apply arbitrage adjustment (simplified without intensity)
    fn apply_arbitrage_adjustment(&self, base_spread: u32, adjustment_factor: Decimal) -> u32 {
        // Convert base_spread to Decimal for precise calculation
        let base_spread_decimal = Decimal::from(base_spread);

        // Apply adjustment factor
        let adjusted_spread_decimal = base_spread_decimal * adjustment_factor;

        // Convert back to u32, with fallback to original value if conversion fails
        let adjusted_spread = adjusted_spread_decimal.to_u32().unwrap_or(base_spread);

        // Ensure reasonable bounds (don't go below 5 bps or above 500 bps)
        adjusted_spread.clamp(5, 500)
    }

    /// Explain the strategy reasoning
    fn explain_strategy(
        &self,
        condition: &MarketCondition,
        cex_price: Decimal,
        dex_price: Option<Decimal>,
        bid_spread_bps: u32,
        ask_spread_bps: u32,
    ) -> String {
        let mut reasoning = format!("Fair value: ${:.2} (CEX reference). ", cex_price);

        match condition {
            MarketCondition::Normal => {
                reasoning.push_str(&format!(
                    "Normal market conditions: symmetric {}bps spread.",
                    (bid_spread_bps + ask_spread_bps) / 2
                ));
            },
            MarketCondition::Arbitrage => {
                if let Some(dex) = dex_price {
                    let direction = if dex > cex_price { "above" } else { "below" };
                    let diff_pct = ((dex - cex_price).abs() / cex_price) * Decimal::new(100, 0);
                    reasoning.push_str(&format!(
                        "DEX price ${:.2} is {:.2}% {} fair value: asymmetric spreads {}bps/{}bps to capture mean reversion.",
                        dex, diff_pct, direction, bid_spread_bps, ask_spread_bps
                    ));
                }
            },
            MarketCondition::Volatile => {
                reasoning.push_str(&format!(
                    "Volatile market conditions: widened spreads to {}bps/{}bps for protection.",
                    bid_spread_bps, ask_spread_bps
                ));
            },
        }

        reasoning
    }
}
