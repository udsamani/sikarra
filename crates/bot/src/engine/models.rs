//! Core data models for arbitrage trading operations.

use alloy::primitives::{keccak256, Address, B256};
use rust_decimal::Decimal;
use serde::Deserialize;
use sikkara_adapters::CoinbaseSymbol;

use crate::config::TokenConfig;

/// Real-time price data from an exchange.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ticker {
    pub exchage: Exchange,
    pub symbol: PoolSymbol,
    pub price: Decimal,
    pub timestamp: jiff::Timestamp,
}

/// Price update from a pool
#[derive(Debug, Clone)]
pub struct PoolPriceUpdate {
    pub symbol: PoolSymbol,
    pub price: Decimal,
}

/// Supported cryptocurrency exchanges.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Exchange {
    Coinbase,
}

/// Represents a trading pool configuration for arbitrage opportunities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pool {
    pub symbol: PoolSymbol,
    pub token_0: Token,
    pub token_1: Token,
    pub fee_tier: u32,
    pub tick_spacing: i32,
    pub hook: Address,
    pub scaling: u8,
}

impl Pool {
    pub fn compute_pool_id(&self) -> B256 {
        let (currency0, currency1) = (self.token_0.address, self.token_1.address);

        // Ensure currency0 < currency1
        let (currency0, currency1) =
            if currency0 < currency1 { (currency0, currency1) } else { (currency1, currency0) };

        let encoded = alloy::sol_types::SolValue::abi_encode(&(
            currency0,
            currency1,
            self.fee_tier,
            self.tick_spacing,
            self.hook,
        ));

        keccak256(encoded)
    }
}

/// Corresponds to a token in a trading pool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub address: Address,
    pub decimals: u8,
}

impl From<&TokenConfig> for Token {
    fn from(config: &TokenConfig) -> Self {
        Self {
            address: Address::parse_checksummed(&config.address, None)
                .expect("Invalid token address"),
            decimals: config.decimals,
        }
    }
}

/// Trading pair symbols for arbitrage opportunities.
#[allow(clippy::upper_case_acronyms)]
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum PoolSymbol {
    #[default]
    EthUsdc,
    EthUsdt,
    UsdcCbbtc,
}

impl PoolSymbol {
    pub fn get_base_asset(&self) -> String {
        match self {
            PoolSymbol::EthUsdc => "ETH".to_string(),
            PoolSymbol::EthUsdt => "ETH".to_string(),
            PoolSymbol::UsdcCbbtc => "BTC".to_string(),
        }
    }
}

impl std::fmt::Display for PoolSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PoolSymbol::EthUsdc => write!(f, "ETH-USDC"),
            PoolSymbol::EthUsdt => write!(f, "ETH-USDT"),
            PoolSymbol::UsdcCbbtc => write!(f, "BTC-USDC"),
        }
    }
}

impl<'de> Deserialize<'de> for PoolSymbol {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        match s.as_str() {
            "ETH-USDC" => Ok(PoolSymbol::EthUsdc),
            "ETH-USDT" => Ok(PoolSymbol::EthUsdt),
            "USDC-cbBTC" => Ok(PoolSymbol::UsdcCbbtc),
            _ => Err(serde::de::Error::custom(format!("Unknown PoolSymbol: {}", s))),
        }
    }
}

/// Convert from Coinbase symbol to internal pool symbol.
impl From<CoinbaseSymbol> for PoolSymbol {
    fn from(symbol: CoinbaseSymbol) -> Self {
        match symbol {
            CoinbaseSymbol::EthUsd => PoolSymbol::EthUsdc,
            CoinbaseSymbol::BtcUsd => PoolSymbol::UsdcCbbtc,
            CoinbaseSymbol::EthUsdt => PoolSymbol::EthUsdt,
        }
    }
}

/// Convert from internal pool symbol to Coinbase symbol.
impl From<PoolSymbol> for CoinbaseSymbol {
    fn from(symbol: PoolSymbol) -> Self {
        match symbol {
            PoolSymbol::EthUsdc => CoinbaseSymbol::EthUsd,
            PoolSymbol::EthUsdt => CoinbaseSymbol::EthUsdt,
            PoolSymbol::UsdcCbbtc => CoinbaseSymbol::BtcUsd,
        }
    }
}

/// Represents a market-making range for a trading pair.
#[derive(Debug, Clone)]
pub struct MarketMakingRange {
    pub symbol: PoolSymbol,
    pub fair_value: Decimal,
    pub bid_price: Decimal,
    pub ask_price: Decimal,
    pub bid_spread_bps: u32,
    pub ask_spread_bps: u32,
    pub total_range_width: Decimal,
    pub reasoning: String,
    pub market_condition: MarketCondition,
}

/// Represents the current market condition for trading strategies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarketCondition {
    Normal,
    Volatile,
    Arbitrage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InternalAction {
    Opportunity,
}

#[derive(Debug, Clone)]
pub enum InternalEvent {
    TickerUpdate(Ticker),
    PoolPriceUpdate(PoolPriceUpdate),
}
