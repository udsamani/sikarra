use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinbaseRequest {
    #[serde(rename = "type")]
    pub request_type: CoinbaseRequestType,
    pub product_ids: Vec<String>,
    pub channels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CoinbaseRequestType {
    Subscribe,
    Unsubscribe,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum CoinbaseMessage {
    ChannelMessage(CoinbaseChannelMessage),
    Response(CoinbaseResponse),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum CoinbaseChannelMessage {
    Ticker(CoinbaseTickerMessage),
    Heartbeat(CoinbaseHeartbeatMessage),
}

#[derive(Debug, Clone, Deserialize)]
pub struct CoinbaseTickerMessage {
    pub sequence: u64,
    pub product_id: String,
    pub price: Decimal,
    pub open_24h: Decimal,
    pub volume_24h: Decimal,
    pub low_24h: Decimal,
    pub high_24h: Decimal,
    pub volume_30d: Decimal,
    pub best_bid: Decimal,
    pub best_bid_size: Decimal,
    pub best_ask: Decimal,
    pub best_ask_size: Decimal,
    pub side: Side,
    #[serde(with = "sikkara_core::timestamp_with_tz_serializer")]
    pub time: jiff::Timestamp,
    pub trade_id: u64,
    pub last_size: Decimal,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CoinbaseHeartbeatMessage {
    pub last_trade_id: u64,
    pub product_id: String,
    pub sequence: u64,
    #[serde(with = "sikkara_core::timestamp_with_tz_serializer")]
    pub time: jiff::Timestamp,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum CoinbaseResponse {
    Subscriptions(CoinbaseSubscriptionsResponse),
}

#[derive(Debug, Clone, Deserialize)]
pub struct CoinbaseSubscriptionsResponse {
    pub channels: Vec<CoinbaseSubscription>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CoinbaseSubscription {
    pub name: String,
    pub product_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Buy,
    Sell,
}

#[cfg(test)]
mod tests {

    use rust_decimal_macros::dec;

    use super::*;

    #[test]
    fn test_coinbase_subscribe_request_serialize() {
        let request = CoinbaseRequest {
            request_type: CoinbaseRequestType::Subscribe,
            product_ids: vec!["BTC-USD".to_string()],
            channels: vec!["tickers".to_string()],
        };

        let serialized = serde_json::to_value(&request).unwrap();
        assert_eq!(
            serialized,
            serde_json::json!({
                "type": "subscribe",
                "product_ids": ["BTC-USD"],
                "channels": ["tickers"]
            })
        );
    }

    #[test]
    fn test_coinbase_subscribe_request_deserialize() {
        let json = serde_json::json!({
            "type": "subscribe",
            "product_ids": ["BTC-USD"],
            "channels": ["tickers"]
        });

        let request: CoinbaseRequest = serde_json::from_value(json).unwrap();
        assert!(matches!(request.request_type, CoinbaseRequestType::Subscribe));
        assert_eq!(request.product_ids, vec!["BTC-USD"]);
        assert_eq!(request.channels, vec!["tickers"]);
    }

    #[test]
    fn test_coinbase_ticker_channel_message_deserialize() {
        let json = serde_json::json!({
            "type": "ticker",
            "sequence": 75193216603_u64,
            "product_id": "ETH-USD",
            "price": "2687.37",
            "open_24h": "2621.85",
            "volume_24h": "132964.98967648",
            "low_24h": "2548",
            "high_24h": "2695.87",
            "volume_30d": "5204346.20541330",
            "best_bid": "2686.83",
            "best_bid_size": "2.01571863",
            "best_ask": "2687.37",
            "best_ask_size": "0.03375599",
            "side": "buy",
            "time": "2025-02-12T21:12:33.778451Z",
            "trade_id": 609139973_u64,
            "last_size": "0.0007456"
        });

        let message: CoinbaseChannelMessage = serde_json::from_value(json.clone()).unwrap();
        match message {
            CoinbaseChannelMessage::Ticker(ticker) => {
                assert_eq!(ticker.product_id, "ETH-USD");
                assert_eq!(ticker.price, dec!(2687.37));
                assert_eq!(ticker.time.to_string(), "2025-02-12T21:12:33.778451Z");
                assert_eq!(ticker.trade_id, 609139973_u64);
                assert!(matches!(ticker.side, Side::Buy));
                assert_eq!(ticker.open_24h, dec!(2621.85));
                assert_eq!(ticker.volume_24h, dec!(132964.98967648));
                assert_eq!(ticker.low_24h, dec!(2548));
                assert_eq!(ticker.high_24h, dec!(2695.87));
                assert_eq!(ticker.volume_30d, dec!(5204346.20541330));
                assert_eq!(ticker.best_bid, dec!(2686.83));
                assert_eq!(ticker.best_bid_size, dec!(2.01571863));
                assert_eq!(ticker.best_ask, dec!(2687.37));
                assert_eq!(ticker.best_ask_size, dec!(0.03375599));
                assert_eq!(ticker.last_size, dec!(0.0007456));
            },
            _ => panic!("Expected Ticker"),
        }

        let message: CoinbaseMessage = serde_json::from_value(json).unwrap();
        match message {
            CoinbaseMessage::ChannelMessage(CoinbaseChannelMessage::Ticker(ticker)) => {
                assert_eq!(ticker.product_id, "ETH-USD");
                assert_eq!(ticker.price, dec!(2687.37));
                assert_eq!(ticker.time.to_string(), "2025-02-12T21:12:33.778451Z");
                assert_eq!(ticker.trade_id, 609139973_u64);
                assert!(matches!(ticker.side, Side::Buy));
                assert_eq!(ticker.open_24h, dec!(2621.85));
                assert_eq!(ticker.volume_24h, dec!(132964.98967648));
                assert_eq!(ticker.low_24h, dec!(2548));
                assert_eq!(ticker.high_24h, dec!(2695.87));
                assert_eq!(ticker.volume_30d, dec!(5204346.20541330));
                assert_eq!(ticker.best_bid, dec!(2686.83));
                assert_eq!(ticker.best_bid_size, dec!(2.01571863));
                assert_eq!(ticker.best_ask, dec!(2687.37));
                assert_eq!(ticker.best_ask_size, dec!(0.03375599));
                assert_eq!(ticker.last_size, dec!(0.0007456));
            },
            _ => panic!("Expected CoinbaseMessage::ChannelMessage with Ticker"),
        }
    }

    #[test]
    fn test_coinbase_subscriptions_response_deserialize() {
        let json = serde_json::json!(
            {
                "type": "subscriptions",
                "channels": [
                    {
                        "name": "ticker",
                        "product_ids": ["BTC-USD"]
                    }
                ]
            }
        );

        let response: CoinbaseResponse = serde_json::from_value(json.clone()).unwrap();
        match response {
            CoinbaseResponse::Subscriptions(subscriptions) => {
                assert_eq!(subscriptions.channels.len(), 1);
                assert_eq!(subscriptions.channels[0].name, "ticker");
                assert_eq!(subscriptions.channels[0].product_ids, vec!["BTC-USD"]);
            },
        }

        let response: CoinbaseMessage = serde_json::from_value(json).unwrap();
        match response {
            CoinbaseMessage::Response(CoinbaseResponse::Subscriptions(subscriptions)) => {
                assert_eq!(subscriptions.channels.len(), 1);
                assert_eq!(subscriptions.channels[0].name, "ticker");
                assert_eq!(subscriptions.channels[0].product_ids, vec!["BTC-USD"]);
            },
            _ => panic!("Expected CoinbaseMessage::Response with Subscriptions"),
        }
    }

    #[test]
    fn test_coinbase_heartbeat_channel_message_deserialize() {
        let json = serde_json::json!({
            "type":"heartbeat",
            "last_trade_id":610049064_u64,
            "product_id":"ETH-USD",
            "sequence":75305048571_u64,
            "time":"2025-02-14T19:51:40.843016Z"
        });

        let message: CoinbaseChannelMessage = serde_json::from_value(json).unwrap();
        match message {
            CoinbaseChannelMessage::Heartbeat(heartbeat) => {
                assert_eq!(heartbeat.last_trade_id, 610049064_u64);
                assert_eq!(heartbeat.product_id, "ETH-USD");
                assert_eq!(heartbeat.sequence, 75305048571_u64);
                assert_eq!(heartbeat.time.to_string(), "2025-02-14T19:51:40.843016Z");
            },
            _ => panic!("Expected Heartbeat"),
        }
    }
}
