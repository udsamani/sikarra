//! Uniswap V4 State Management
//!
//! This module provides functionality to watch and stream Uniswap V4 pool state
//! changes in real-time. It uses polling-based approach to fetch pool data at
//! configurable intervals.

use std::{pin::Pin, sync::Arc, time::Duration};

use alloy::{
    primitives::{Address, B256},
    sol,
};
use futures::{stream, Stream};
use tokio::time::interval;
use tracing::error;

use crate::uniswap_v4::models::PoolSlotData;

// Generate contract bindings from ABI
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    UniswapV4,
    "abis/StateView.json"
);

/// A stream that emits [`PoolSlotData`] whenever pool state is fetched.
///
/// The stream will emit data at regular intervals based on the polling
/// frequency configured when creating the stream. Each emission contains the
/// current state of the pool including price, tick, and fee information.
pub type PoolSlotDataStream = Pin<Box<dyn Stream<Item = PoolSlotData> + Send>>;

/// Manager for watching Uniswap V4 pool state changes.
///
/// This struct provides methods to create streams that monitor pool state
/// changes by polling the Uniswap V4 contract at configurable intervals. It
/// uses an RPC provider to fetch data from the blockchain.
///
/// # Type Parameters
///
/// * `P` - The RPC provider type that implements [`alloy::providers::Provider`]
pub struct UniswapV4StateViewManager<P>
where
    P: alloy::providers::Provider + Send + Sync,
{
    /// The RPC provider for blockchain interactions
    provider: Arc<P>,
    /// The address of the Uniswap V4 contract
    address: Address,
}

impl<P> UniswapV4StateViewManager<P>
where
    P: alloy::providers::Provider + Send + Sync + 'static,
{
    /// Creates a new state view manager.
    ///
    /// # Arguments
    ///
    /// * `provider` - Arc-wrapped RPC provider for blockchain communication
    /// * `address` - The contract address of the Uniswap V4 pool manager
    ///
    /// # Returns
    ///
    /// A new [`UniswapV4StateViewManager`] instance
    pub fn new(provider: Arc<P>, address: Address) -> Self {
        Self { provider, address }
    }

    /// Creates a stream that watches a specific pool's state changes.
    ///
    /// This method creates an infinite stream that polls the specified pool at
    /// regular intervals and emits [`PoolSlotData`] containing the current
    /// pool state. The stream will continue indefinitely until dropped.
    ///
    /// # Arguments
    ///
    /// * `pool_id` - The unique identifier (hash) of the pool to watch
    /// * `poll_interval` - How frequently to poll for state changes
    ///
    /// # Recommended Intervals
    ///
    /// - **Arbitrage**: 500ms - 1s (catch every price movement)
    /// - **Price monitoring**: 2-5s (balance updates vs efficiency)
    /// - **Analytics**: 10-30s (reduce RPC load)
    pub fn watch_pool(
        &self,
        pool_id: B256,
        poll_interval: Duration,
        invert: bool,
    ) -> PoolSlotDataStream {
        let provider = self.provider.clone();
        let address = self.address;

        let stream = stream::unfold(
            (provider, address, pool_id, interval(poll_interval)),
            move |(provider, address, pool_id, mut timer)| async move {
                // Wait for the next polling interval
                timer.tick().await;

                // Attempt to fetch current pool state
                match UniswapV4::new(address, &provider)
                    .getSlot0(pool_id)
                    .call()
                    .await
                {
                    Ok(slot) => {
                        // Successfully fetched slot0 data, create PoolSlotData
                        let data = PoolSlotData::new(
                            slot.sqrtPriceX96,
                            slot.tick,
                            slot.protocolFee,
                            slot.lpFee,
                            18,
                            6,
                            invert,
                        );
                        // Return data and continue the stream
                        println!("Fetched pool state: {:?}", slot);
                        Some((data, (provider, address, pool_id, timer)))
                    },
                    Err(e) => {
                        // Log error and skip this iteration
                        error!(
                            pool_id = %pool_id,
                            error = %e,
                            "Failed to fetch pool state from contract"
                        );
                        // Return None to skip emission but continue stream
                        None
                    },
                }
            },
        );

        Box::pin(stream)
    }
}
