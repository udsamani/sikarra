//! Uniswap V4 Pool Data Models
//!
//! This module defines the core data structures for representing Uniswap V4
//! pool state. It provides types for pool slot data including price
//! calculations and conversions between different price representations (tick,
//! sqrtPriceX96, decimal price).

use core::num;

use alloy::{
    dyn_abi::abi::token,
    primitives::{
        aliases::{I24, U24},
        U160, U256,
    },
    signers::k256::elliptic_curve::consts::U25,
};
use fastnum::{
    decimal::{Context, RoundingMode},
    i512, D512, I512, U512,
};
use rust_decimal::Decimal;

pub const Q192: I512 = I512::from_bits(U512::from_digits([0, 0, 0, 1, 0, 0, 0, 0]));

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
#[derive(Debug, Clone)]
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

    /// The calculated spot price
    pub spot_price: SpotPrice,
}

impl PoolSlotData {
    /// Creates a new [`PoolSlotData`] instance with calculated spot price.
    ///
    /// This constructor takes the raw contract data and performs the necessary
    /// calculations to derive the human-readable spot price. The spot price
    /// calculation includes proper decimal adjustments for the token pair.
    pub fn new(
        sqrt_price_x96: U160,
        tick: I24,
        protocol_fee: U24,
        lp_fee: U24,
        token_0_decimals: u8,
        token_1_decimals: u8,
        invert: bool,
    ) -> Self {
        let spot_price = SpotPrice::new_from_sqrt_ratio_x96(
            sqrt_price_x96,
            token_0_decimals,
            token_1_decimals,
            invert,
        );
        Self { sqrt_price_x96, tick: tick.as_i32(), protocol_fee, lp_fee, spot_price }
    }
}

/// High precision spot price representation of Uniswap V4 pools.
///
/// This structure provides exact decimal arithmetic for price calculations
/// avoiding floating-point precision errors that could be exploited in
/// financial applications. It handles the conversion from Uniswap's internal
/// `sqrt_price_x96` format to human readable prices with proper token decimal
/// adjustments
///
/// # Fields
///
/// - `numerator`: The numerator of the price ratio.
/// - `denominator`: The denominator of the price ratio.
/// - `scale`: A decimal scale factor to adjust the price based on token
///   decimals.
#[derive(Debug, Clone)]
pub struct SpotPrice {
    numerator: I512,
    denominator: I512,
    scale: D512,
}

impl SpotPrice {
    /// Creates a new `SpotPrice` instance from the square root price ratio.
    ///
    /// This method performs core price calculation that converts from Uniswap's
    /// internal format to a human readable price with proper token decimal
    /// adjustments
    ///
    /// # Arguments
    ///
    /// - `sqrt_ratio_x96`: The square root price ratio in x96 format.
    /// - `token_0_decimals`: The number of decimals for token 0.
    /// - `token_1_decimals`: The number of decimals for token 1.
    /// - `invert`: Whether to invert the price ratio aka compute token0/token1
    ///   or token1/token0.
    pub fn new_from_sqrt_ratio_x96(
        sqrt_ratio_x96: U160,
        token_0_decimals: u8,
        token_1_decimals: u8,
        invert: bool,
    ) -> Self {
        // Calculate the numerator and denominator based on the square root price
        let x96_i512 = I512::from_le_slice(&sqrt_ratio_x96.as_le_bytes()).unwrap();
        let ratio_x192 = x96_i512.pow(2);

        let scale_0 = i512!(10).pow(token_0_decimals as u32);
        let scale_1 = i512!(10).pow(token_1_decimals as u32);
        let scale_0_decimal = D512::from_parts(
            scale_0.to_bits(),
            0,
            match scale_0.is_negative() {
                true => fastnum::decimal::Sign::Minus,
                false => fastnum::decimal::Sign::Plus,
            },
            Context::default(),
        );
        let scale_1_decimal = D512::from_parts(
            scale_1.to_bits(),
            0,
            match scale_1.is_negative() {
                true => fastnum::decimal::Sign::Minus,
                false => fastnum::decimal::Sign::Plus,
            },
            Context::default(),
        );
        let scale = scale_0_decimal / scale_1_decimal;

        let (numerator, denominator) = if invert { (ratio_x192, Q192) } else { (Q192, ratio_x192) };
        Self { numerator, denominator, scale }
    }

    /// Returns the adjusted price in decimal format, taking into account the
    /// token decimal scales.
    pub fn adjsusted_to_decimal(&self) -> D512 {
        self.to_decimal() * self.scale
    }

    fn to_decimal(&self) -> D512 {
        let numerator_decimal = D512::from_parts(
            self.numerator.to_bits(),
            0,
            match self.numerator.is_negative() {
                true => fastnum::decimal::Sign::Minus,
                false => fastnum::decimal::Sign::Plus,
            },
            Context::default(),
        );

        let denominator_decimal = D512::from_parts(
            self.denominator.to_bits(),
            0,
            match self.denominator.is_negative() {
                true => fastnum::decimal::Sign::Minus,
                false => fastnum::decimal::Sign::Plus,
            },
            Context::default(),
        );

        numerator_decimal / denominator_decimal
    }

    /// Formats the price to fixed decimal representation with specified
    /// rounding controls
    pub fn to_fixed(&self, rounding_decimals: u8, rounding_mode: Option<RoundingMode>) -> String {
        let rounding = rounding_mode.unwrap_or(RoundingMode::HalfUp);
        self.adjsusted_to_decimal()
            .with_rounding_mode(rounding)
            .round(rounding_decimals as i16)
            .to_string()
    }
}
