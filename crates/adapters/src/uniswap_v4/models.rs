//! Uniswap V4 Pool Data Models
//!
//! This module defines the core data structures for representing Uniswap V4
//! pool state. It provides types for pool slot data including price
//! calculations and conversions between different price representations (tick,
//! sqrtPriceX96, decimal price).

use alloy::primitives::{
    aliases::{I24, U24},
    U160,
};
use rust_decimal::Decimal;

/// Represents the complete state data for a Uniswap V4 pool at a specific point
/// in time.
///
/// This struct contains all the essential information about a pool's current
/// state, including price information in multiple formats and fee
/// configurations. The data corresponds to the pool's `slot0` storage slot plus
/// additional calculated fields.
///
/// # Price Representations
///
/// The pool state includes price in three different formats:
/// - `sqrt_price_x96`: The raw square root price scaled by 2^96 (from contract)
/// - `tick`: The current tick (logarithmic price representation)
/// - `spot_price`: Human-readable decimal price (calculated)
#[derive(Debug, Clone, PartialEq)]
pub struct PoolSlotData {
    /// The square root of the price scaled by 2^96.
    pub sqrt_price_x96: U160,

    /// The current tick of the pool.
    ///
    /// Ticks represent price in logarithmic scale where each tick represents
    /// a 0.01% (1 basis point) price change. The relationship between tick and
    /// price is: `price = 1.0001^tick`
    pub tick: i32,

    /// Protocol fee charged on swaps, expressed in basis points.
    ///
    /// This fee goes to the protocol treasury and is separate from LP fees.
    /// The fee is a percentage of the LP fee (not a percentage of the swap
    /// amount).
    pub protocol_fee: U24,

    /// Liquidity provider fee charged on swaps, expressed in basis points.
    ///
    /// This is the total fee charged to swappers, which is split between
    /// liquidity providers and the protocol based on the protocol fee.
    pub lp_fee: U24,

    /// The spot price in human-readable decimal format.
    ///
    /// This represents the price of token1 in terms of token0, adjusted for
    /// decimal differences between the tokens. For WETH/USDC pools:
    /// - Represents: "How many USDC to buy 1 WETH"
    /// - Example: 3289.52 means 1 WETH costs 3,289.52 USDC
    pub spot_price: Decimal,
}

impl PoolSlotData {
    /// Creates a new [`PoolSlotData`] instance with calculated spot price.
    ///
    /// This constructor takes the raw contract data and performs the necessary
    /// calculations to derive the human-readable spot price. The spot price
    /// calculation includes proper decimal adjustments for the token pair.
    pub fn new(sqrt_price_x96: U160, tick: I24, protocol_fee: U24, lp_fee: U24) -> Self {
        let spot_price = Self::compute_spot_price(sqrt_price_x96, tick);
        Self { sqrt_price_x96, tick: tick.as_i32(), protocol_fee, lp_fee, spot_price }
    }

    /// Computes the spot price from raw Uniswap V4 price data.
    ///
    /// This method converts from Uniswap's internal price representation to a
    /// human-readable decimal price suitable for display and calculations.
    /// It handles the mathematical conversion and decimal adjustments needed
    /// for different token pairs.
    pub fn compute_spot_price(sqrt_price_x96: U160, tick: I24) -> Decimal {
        // TODO: Implement price calculation
        // This is a placeholder that should be replaced with actual calculation logic
        Decimal::ZERO
    }

    /// Returns the spot price rounded to a specific number of decimal places.
    pub fn spot_price_rounded(&self, decimal_places: u32) -> Decimal {
        self.spot_price.round_dp(decimal_places)
    }

    /// Formats the spot price for display purposes.
    ///
    /// Returns a string representation of the price with dollar sign and
    /// two decimal places, suitable for user interfaces.
    pub fn format_price(&self) -> String {
        format!("${:.2}", self.spot_price)
    }
}
