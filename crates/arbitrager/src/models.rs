//! Core data models for arbitrage trading operations.

use rust_decimal::Decimal;
use serde::Deserialize;
use sikkara_adapters::CoinbaseSymbol;

/// Real-time price data from an exchange.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ticker {
    pub exchage: Exchange,
    pub symbol: PoolSymbol,
    pub price: Decimal,
    pub timestamp: jiff::Timestamp,
}

/// Supported cryptocurrency exchanges.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Exchange {
    Coinbase,
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
