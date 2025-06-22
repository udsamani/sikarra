#[allow(unused)]
mod models;
#[allow(unused)]
pub use models::{
    CoinbaseChannelMessage, CoinbaseHeartbeatMessage, CoinbaseMessage, CoinbaseRequest,
    CoinbaseRequestType, CoinbaseResponse, CoinbaseTickerMessage,
};

mod callback;
