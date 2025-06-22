//! Configuration structures for arbitrage trading setup.
//!
//! This module defines the configuration format for setting up arbitrage
//! trading between centralized exchanges (CEX) and decentralized exchanges
//! (DEX).

use serde::Deserialize;

use crate::models::PoolSymbol;

/// Main configuration for arbitrage trading operations.
///
/// Contains configuration for both DEX pools and CEX connections that will be
/// monitored for arbitrage opportunities.
#[derive(Debug, Clone, Deserialize)]
pub struct ArbitrageConfig {
    /// List of DEX pools to monitor for arbitrage opportunities
    pub pools: Vec<Pool>,
    /// Centralized exchange configuration for price feeds
    pub cex: CexConfig,
}

/// Configuration for a decentralized exchange pool.
///
/// Represents a trading pool on a DEX that can be monitored for arbitrage
/// opportunities against centralized exchange prices.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "dex", rename = "lowercase")]
pub enum Pool {
    /// Uniswap V4 pool configuration
    #[serde(rename = "uniswapv4")]
    UniswapV4 {
        /// Contract address of the Uniswap V4 pool
        address: String,
        /// Trading pair symbol for this pool
        symbol: PoolSymbol,
    },
}

impl Pool {
    /// Returns the contract address of the pool.
    pub fn address(&self) -> &str {
        match self {
            Pool::UniswapV4 { address, .. } => address,
        }
    }

    /// Returns a reference to the trading pair symbol.
    pub fn symbol(&self) -> &PoolSymbol {
        match self {
            Pool::UniswapV4 { symbol, .. } => symbol,
        }
    }

    /// Returns an owned copy of the trading pair symbol.
    pub fn symbol_owned(&self) -> PoolSymbol {
        match self {
            Pool::UniswapV4 { symbol, .. } => symbol.clone(),
        }
    }
}

/// Configuration for centralized exchange connections.
///
/// Defines which centralized exchange to connect to and how to establish
/// the connection for receiving real-time price feeds.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "exchange", rename = "lowercase")]
pub enum CexConfig {
    /// Coinbase Pro WebSocket configuration
    #[serde(rename = "coinbase")]
    Coinbase {
        /// WebSocket URL for Coinbase Pro price feeds
        ws_url: String,
    },
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn config_deserialization() {
        let json_data = json!({
            "pools": [
                {
                    "dex": "uniswapv4",
                    "address": "0x1234567890abcdef1234567890abcdef12345678",
                    "symbol": "ETH-USDC"
                }
            ],
            "cex": {
                "exchange": "coinbase",
                "ws_url": "wss://ws-feed.pro.coinbase.com"
            }
        });

        let config: ArbitrageConfig = serde_json::from_value(json_data).unwrap();
        assert_eq!(config.pools.len(), 1);
        let Pool::UniswapV4 { address, symbol } = &config.pools[0];
        assert_eq!(address, "0x1234567890abcdef1234567890abcdef12345678");
        assert_eq!(*symbol, PoolSymbol::ETHUSDC);
        let CexConfig::Coinbase { ws_url } = &config.cex;
        assert_eq!(ws_url, "wss://ws-feed.pro.coinbase.com");
    }
}
