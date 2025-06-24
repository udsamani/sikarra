use std::{pin::Pin, time::Duration};

use rust_decimal::Decimal;
use sikkara_adapters::UniswapV4StateViewManager;
use sikkara_core::AppResult;
use tokio_stream::StreamExt;

use crate::engine::{Pool, PoolPriceUpdate, PoolSymbol};

/// A stream that yields updates for trading pools, providing real-time data
pub type PoolUpdateStream<'a> =
    Pin<Box<dyn tokio_stream::Stream<Item = PoolPriceUpdate> + Send + 'a>>;

/// Trait for subscribing to and managing updates for trading pools.
#[async_trait::async_trait]
pub trait PoolFeed {
    /// Subscribe to pool updates for a specific trading pair.
    ///    /// Creates a stream that yields updates for the specified trading
    /// pair.
    ///
    /// # Parameters
    ///
    /// * `pool_symbol` - The trading pair symbol to subscribe to (e.g.,
    ///   ETH-USDC)
    async fn subscribe_pool_updates(&mut self, pool: Pool) -> AppResult<PoolUpdateStream<'_>>;

    /// Unsubscribe from pool updates for a specific trading pair.
    ///
    /// Stops receiving updates for the specified trading pair.
    ///
    /// # Parameters
    ///
    /// * `pool_symbol` - The trading pair symbol to unsubscribe from
    async fn unsubscribe_pool_updates(&mut self, pool_symbol: PoolSymbol) -> AppResult<()>;
}

#[async_trait::async_trait]
impl<P> PoolFeed for UniswapV4StateViewManager<P>
where
    P: alloy::providers::Provider + Send + Sync + 'static,
{
    async fn subscribe_pool_updates(&mut self, pool: Pool) -> AppResult<PoolUpdateStream<'_>> {
        let pool_id = pool.compute_pool_id();
        let symbol = pool.symbol.clone();
        let stream = self.watch_pool(
            pool_id,
            Duration::from_secs(5), // Polling interval
            pool.token_0.address < pool.token_1.address,
        );

        let stream = stream.filter_map(move |pool_slot_data| {
            let price = pool_slot_data.spot_price.to_fixed(pool.scaling, None);
            let price = match Decimal::from_str_exact(&price) {
                Ok(p) => p,
                Err(e) => {
                    tracing::error!("Failed to parse price: {}", e);
                    return None;
                },
            };
            let pool_price_update = PoolPriceUpdate { symbol: symbol.clone(), price };
            Some(pool_price_update)
        });
        Ok(Box::pin(stream))
    }

    async fn unsubscribe_pool_updates(&mut self, _pool_symbol: PoolSymbol) -> AppResult<()> {
        // Unsubscription logic if needed, currently a no-op
        Ok(())
    }
}
