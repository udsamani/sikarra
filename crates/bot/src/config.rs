//! Configuration structures for arbitrage trading setup.
//!
//! This module defines the configuration format for setting up arbitrage
//! trading between centralized exchanges (CEX) and decentralized exchanges
//! (DEX).

use serde::Deserialize;

use crate::engine::PoolSymbol;

/// Main configuration for  trading operations.
///
/// Contains configuration for both DEX pools and CEX connections that will be
/// monitored for arbitrage and market making simulation opportunities.
#[derive(Debug, Clone, Deserialize)]
pub struct BotConfig {
    /// List of DEX pools to monitor for arbitrage opportunities
    pub pools: Vec<PoolConfig>,
    /// Centralized exchange configuration for price feeds
    pub cex: CexConfig,
}

/// Configuration for a decentralized exchange pool.
///
/// Represents a trading pool on a DEX that can be monitored for arbitrage
/// opportunities against centralized exchange prices.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "dex", rename = "lowercase")]
pub enum PoolConfig {
    /// Uniswap V4 pool configuration
    #[serde(rename = "uniswapv4")]
    UniswapV4 {
        /// Contract address of the Uniswap V4 pool
        address: String,
        /// Trading pair symbol for this pool
        symbol: PoolSymbol,
        /// Token 0, meaning the first token decimal places
        token_0: TokenConfig,
        /// Token 1, meaning the second token decimal places
        token_1: TokenConfig,
        /// Fee tier for the pool, e.g., 0.01% means 100, 0.05% means 500, 3%
        /// means 30000
        fee_tier: u32,
        /// Node Url
        node_url: String,
        /// Tick spacing for the pool, used for price granularity
        tick_spacing: i32,
        /// Hook address for the pool, if applicable
        #[serde(skip_serializing_if = "Option::is_none")]
        hook_address: Option<String>,
        /// Scaling factor for the pool, used for price calculations
        scaling: u8,
    },
}

/// Represnts Token configuration in a trading pool.
#[derive(Debug, Clone, Deserialize)]
pub struct TokenConfig {
    /// Address of the token contract
    pub address: String,
    /// Number of decimal places for the token
    pub decimals: u8,
}

impl PoolConfig {
    /// Returns the contract address of the pool.
    pub fn address(&self) -> &str {
        match self {
            PoolConfig::UniswapV4 { address, .. } => address,
        }
    }

    /// Returns a reference to the trading pair symbol.
    pub fn symbol(&self) -> &PoolSymbol {
        match self {
            PoolConfig::UniswapV4 { symbol, .. } => symbol,
        }
    }

    /// Returns an owned copy of the trading pair symbol.
    pub fn symbol_owned(&self) -> PoolSymbol {
        match self {
            PoolConfig::UniswapV4 { symbol, .. } => symbol.clone(),
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
    use crate::engine::PoolSymbol;

    #[test]
    fn config_deserialization() {
        let json_data = json!({
            "pools": [
                {
                    "dex": "uniswapv4",
                    "address": "0x1234567890abcdef1234567890abcdef12345678",
                    "symbol": "ETH-USDC",
                    "token_0": {
                        "address": "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd",
                        "decimals": 18
                    },
                    "token_1": {
                        "address": "0x1234567890abcdef1234567890abcdef12345678",
                        "decimals": 6
                    },
                    "tick_spacing": 10,
                    "fee_tier": 500,
                    "scaling": 2,
                    "node_url": "https://mainnet.infura.io/v3/YOUR_INFURA_PROJECT_ID",
                }
            ],
            "cex": {
                "exchange": "coinbase",
                "ws_url": "wss://ws-feed.pro.coinbase.com"
            }
        });

        let config: BotConfig = serde_json::from_value(json_data).unwrap();
        assert_eq!(config.pools.len(), 1);
        let PoolConfig::UniswapV4 {
            address,
            symbol,
            token_0,
            token_1,
            fee_tier,
            node_url,
            hook_address,
            tick_spacing,
            scaling,
        } = &config.pools[0];
        assert_eq!(address, "0x1234567890abcdef1234567890abcdef12345678");
        assert_eq!(*symbol, PoolSymbol::ETHUSDC);
        assert_eq!(token_0.decimals, 18);
        assert_eq!(token_1.decimals, 6);
        assert_eq!(*fee_tier, 3000);
        assert_eq!(*tick_spacing, 10);
        assert_eq!(node_url, "https://mainnet.infura.io/v3/YOUR_INFURA_PROJECT_ID");
        assert_eq!(*scaling, 2);
        let CexConfig::Coinbase { ws_url } = &config.cex;
        assert_eq!(ws_url, "wss://ws-feed.pro.coinbase.com");
    }
}
