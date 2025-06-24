//! Price feed module for handling real-time market data streams.
//!
//! This module provides a unified interface for subscribing to price feeds from
//! various cryptocurrency exchanges. It handles message processing, filtering,
//! and conversion to standardized ticker formats.
use std::pin::Pin;

use sikkara_adapters::{CoinbaseChannelMessage, CoinbaseMessage, CoinbaseWsClient};
use sikkara_core::AppResult;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use tracing::{debug, error, info, warn};

use crate::engine::{Exchange, PoolSymbol, Ticker};

/// A pinned stream that yields ticker data for price feeds.
///
/// This type represents an asynchronous stream of ticker updates that can be
/// consumed to receive real-time price information from exchanges.
pub type PriceFeedSubscription<'a> = Pin<Box<dyn tokio_stream::Stream<Item = Ticker> + Send + 'a>>;

/// Trait for subscribing to and managing price feeds from cryptocurrency
/// exchanges.
///
/// This trait provides a unified interface for connecting to various exchange
/// WebSocket APIs and receiving standardized ticker data. Implementations
/// handle the exchange-specific message formats and convert them to our
/// internal ticker representation.
#[async_trait::async_trait]
pub trait PriceFeed {
    /// Subscribe to price feed for a specific trading pair.
    ///
    /// Creates a WebSocket subscription to receive real-time price updates
    /// for the specified trading pair. The returned stream will yield
    /// `Ticker` objects containing price and metadata.
    ///
    /// # Parameters
    ///
    /// * `pool_symbol` - The trading pair symbol to subscribe to (e.g.,
    ///   BTC-USD)
    async fn subscribe_price_feed(
        &mut self,
        pool_symbol: PoolSymbol,
    ) -> AppResult<PriceFeedSubscription<'_>>;

    /// Unsubscribe from price feed for a specific trading pair.
    ///
    /// Stops receiving price updates for the specified trading pair and
    /// cleans up the associated WebSocket subscription resources.
    ///
    /// # Parameters
    ///
    /// * `pool_symbol` - The trading pair symbol to unsubscribe from
    async fn unsubscribe_price_feed(&mut self, pool_symbol: PoolSymbol) -> AppResult<()>;
}

/// Helper struct to process Coinbase WebSocket messages.
///
/// This struct contains utility methods for parsing and filtering Coinbase
/// WebSocket messages, converting them to standardized ticker format, and
/// handling various message types and error conditions.
///
/// The processor handles:
/// - Ticker updates with price and volume data
/// - Heartbeat messages for connection health
/// - Subscription responses and confirmations
/// - Stream errors and connection issues
struct CoinbaseMessageProcessor;

impl CoinbaseMessageProcessor {
    /// Creates a filtered stream that converts Coinbase messages to Ticker
    /// objects.
    fn create_ticker_stream(
        receiver: tokio::sync::broadcast::Receiver<CoinbaseMessage>,
    ) -> impl tokio_stream::Stream<Item = Ticker> {
        BroadcastStream::new(receiver).filter_map(|result| match result {
            Ok(message) => Self::process_coinbase_message(message),
            Err(e) => {
                Self::handle_stream_error(e);
                None
            },
        })
    }

    /// Processes a Coinbase WebSocket message and extracts ticker data.
    ///
    /// Handles the top-level message types from Coinbase WebSocket API:
    /// - Channel messages (ticker, heartbeat)
    /// - Response messages (subscription confirmations)
    fn process_coinbase_message(message: CoinbaseMessage) -> Option<Ticker> {
        match message {
            CoinbaseMessage::ChannelMessage(channel_msg) => {
                Self::process_channel_message(channel_msg)
            },
            CoinbaseMessage::Response(response) => {
                debug!("Received subscription response: {:?}", response);
                None
            },
        }
    }

    /// Processes channel-specific messages (ticker, heartbeat, etc.).
    ///
    /// Handles different types of channel messages from Coinbase:
    /// - Ticker messages containing price and volume data
    /// - Heartbeat messages for connection health monitoring
    fn process_channel_message(message: CoinbaseChannelMessage) -> Option<Ticker> {
        match message {
            CoinbaseChannelMessage::Ticker(ticker_data) => {
                Some(Self::convert_to_ticker(ticker_data))
            },
            CoinbaseChannelMessage::Heartbeat(heartbeat) => {
                debug!("Received heartbeat for product: {}", heartbeat.product_id);
                None
            },
        }
    }

    /// Converts Coinbase ticker data to our internal Ticker model.
    ///
    /// Maps Coinbase-specific ticker fields to our standardized ticker format,
    /// extracting the essential price and timing information needed for
    /// arbitrage analysis.
    fn convert_to_ticker(coinbase_ticker: sikkara_adapters::CoinbaseTickerMessage) -> Ticker {
        Ticker {
            symbol: coinbase_ticker.product_id.into(),
            price: coinbase_ticker.price,
            exchage: Exchange::Coinbase,
            timestamp: coinbase_ticker.time,
        }
    }

    /// Handles stream errors with appropriate logging.
    ///
    /// Processes different types of broadcast stream errors and logs them
    /// appropriately. Currently handles lagged message warnings when the
    /// consumer falls behind the producer.
    fn handle_stream_error(error: tokio_stream::wrappers::errors::BroadcastStreamRecvError) {
        match error {
            tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(n) => {
                warn!("Stream lagged by {} messages, some data may be lost", n);
            },
        }
    }
}

#[async_trait::async_trait]
impl PriceFeed for CoinbaseWsClient {
    async fn subscribe_price_feed(
        &mut self,
        pool_symbol: PoolSymbol,
    ) -> AppResult<PriceFeedSubscription<'_>> {
        let product_ids = vec![pool_symbol.into()];
        let channels = vec!["ticker".to_string()];

        let receiver = self.subscribe(product_ids, channels)?;
        let stream = CoinbaseMessageProcessor::create_ticker_stream(receiver);

        Ok(Box::pin(stream))
    }

    async fn unsubscribe_price_feed(&mut self, pool_symbol: PoolSymbol) -> AppResult<()> {
        let product_ids = vec![pool_symbol.into()];
        let channels = vec!["ticker".to_string()];
        self.unsubscribe(product_ids, channels)
    }
}
