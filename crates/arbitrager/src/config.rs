use serde::Deserialize;

use crate::models::PoolSymbol;

#[derive(Debug, Clone, Deserialize)]
pub struct ArbitrageConfig {
    pub pools: Vec<Pool>,
    pub cex: CexConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "dex", rename = "lowercase")]
pub enum Pool {
    #[serde(rename = "uniswapv4")]
    UniswapV4 { address: String, symbol: PoolSymbol },
}

impl Pool {
    pub fn address(&self) -> &str {
        match self {
            Pool::UniswapV4 { address, .. } => address,
        }
    }

    pub fn symbol(&self) -> &PoolSymbol {
        match self {
            Pool::UniswapV4 { symbol, .. } => symbol,
        }
    }

    pub fn symbol_owned(&self) -> PoolSymbol {
        match self {
            Pool::UniswapV4 { symbol, .. } => symbol.clone(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "exchange", rename = "lowercase")]
pub enum CexConfig {
    #[serde(rename = "coinbase")]
    Coinbase { ws_url: String },
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
