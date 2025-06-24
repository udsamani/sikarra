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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PoolSymbol {
    ETHUSDC,
    ETHUSDT,
    BTCUSDC,
    BTCUSDT,
}

impl std::fmt::Display for PoolSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PoolSymbol::ETHUSDC => write!(f, "ETH-USDC"),
            PoolSymbol::ETHUSDT => write!(f, "ETH-USDT"),
            PoolSymbol::BTCUSDC => write!(f, "BTC-USDC"),
            PoolSymbol::BTCUSDT => write!(f, "BTC-USDT"),
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
            "ETH-USDC" => Ok(PoolSymbol::ETHUSDC),
            "ETH-USDT" => Ok(PoolSymbol::ETHUSDT),
            "BTC-USDC" => Ok(PoolSymbol::BTCUSDC),
            "BTC-USDT" => Ok(PoolSymbol::BTCUSDT),
            _ => Err(serde::de::Error::custom(format!("Unknown PoolSymbol: {}", s))),
        }
    }
}

/// Convert from Coinbase symbol to internal pool symbol.
impl From<CoinbaseSymbol> for PoolSymbol {
    fn from(symbol: CoinbaseSymbol) -> Self {
        match symbol {
            CoinbaseSymbol::EthUsd => PoolSymbol::ETHUSDC,
            CoinbaseSymbol::BtcUsd => PoolSymbol::BTCUSDC,
            CoinbaseSymbol::EthUsdt => PoolSymbol::ETHUSDT,
            CoinbaseSymbol::BtcUsdt => PoolSymbol::BTCUSDT,
        }
    }
}

/// Convert from internal pool symbol to Coinbase symbol.
impl From<PoolSymbol> for CoinbaseSymbol {
    fn from(symbol: PoolSymbol) -> Self {
        match symbol {
            PoolSymbol::ETHUSDC => CoinbaseSymbol::EthUsd,
            PoolSymbol::ETHUSDT => CoinbaseSymbol::EthUsdt,
            PoolSymbol::BTCUSDC => CoinbaseSymbol::BtcUsd,
            PoolSymbol::BTCUSDT => CoinbaseSymbol::BtcUsdt,
        }
    }
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
