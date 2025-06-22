//! Arbitrage trading engine for processing market events and generating trading
//! actions.
//!
//! This module contains the core arbitrage engine that processes incoming
//! market data events and uses configured strategies to identify arbitrage
//! opportunities and generate appropriate trading actions.

use sikkara_core::{AppResult, Engine};
use tracing::info;

use crate::{action::InternalAction, event::InternalEvent, strategy::ArbitrageStrategy};

/// Core arbitrage trading engine that processes market events and executes
/// strategies. Note this engine is designed to be per pool/pair, meaning it
/// operates on a specific trading pair.
///
/// The `ArbitrageEngine` is responsible for:
/// - Processing incoming market data events (ticker updates)
/// - Applying arbitrage strategies to identify trading opportunities
/// - Generating trading actions based on strategy decisions
///
/// # Type Parameters
///
/// * `S` - The arbitrage strategy implementation that defines trading logic
pub struct ArbitrageEngine<S>
where
    S: ArbitrageStrategy,
{
    /// Unique identifier for this engine instance
    name: String,
    /// The arbitrage strategy used to evaluate trading opportunities
    strategy: S,
    /// The trading pool/pair this engine is monitoring (e.g., "ETH-USDT")
    pool: String,
}

impl<S> ArbitrageEngine<S>
where
    S: ArbitrageStrategy,
{
    /// Creates a new arbitrage engine with the specified strategy and trading
    /// pool.
    ///
    /// # Parameters
    ///
    /// * `strategy` - The arbitrage strategy implementation to use for trading
    ///   decisions
    /// * `pool` - The trading pair identifier (e.g., "ETH-USDT", "BTC-USDC")
    ///
    /// # Returns
    ///
    /// Returns a new `ArbitrageEngine` instance configured with the given
    /// strategy and pool.
    pub fn new(strategy: S, pool: String) -> Self {
        let name = format!("arbitrage_engine_{}", pool);
        Self { strategy, pool, name }
    }

    /// Gets the trading pool this engine is monitoring.
    ///
    /// # Returns
    ///
    /// Returns a reference to the trading pool identifier string.
    pub fn pool(&self) -> &str {
        &self.pool
    }

    /// Gets a reference to the arbitrage strategy.
    ///
    /// # Returns
    ///
    /// Returns a reference to the configured arbitrage strategy.
    pub fn strategy(&self) -> &S {
        &self.strategy
    }

    /// Gets a mutable reference to the arbitrage strategy.
    ///
    /// This allows for runtime strategy configuration updates.
    ///
    /// # Returns
    ///
    /// Returns a mutable reference to the configured arbitrage strategy.
    pub fn strategy_mut(&mut self) -> &mut S {
        &mut self.strategy
    }
}

#[async_trait::async_trait]
impl<S> Engine<InternalEvent, InternalAction> for ArbitrageEngine<S>
where
    S: ArbitrageStrategy + Send + Sync,
{
    /// Returns the unique identifier for this engine instance.
    ///
    /// # Returns
    ///
    /// Returns "arbitrage_engine" as the engine identifier.
    fn id(&self) -> &str {
        &self.name
    }

    /// Processes incoming market events and generates trading actions.
    ///
    /// This method handles different types of market events and applies the
    /// configured arbitrage strategy to determine if any trading actions
    /// should be taken.
    ///
    /// # Parameters
    ///
    /// * `event` - The market event to process (ticker updates, order book
    ///   changes, etc.)
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(action))` if a trading action should be taken,
    /// `Ok(None)` if no action is needed, or an error if processing fails.
    async fn process_event(&mut self, event: InternalEvent) -> AppResult<Option<InternalAction>> {
        match event {
            InternalEvent::TickerUpdate(ticker) => {
                info!(
                    "Processing ticker update for {}: price=${} at {}",
                    ticker.symbol, ticker.price, ticker.timestamp
                );

                // TODO: Apply arbitrage strategy to evaluate opportunity
                // let opportunity = self.strategy.evaluate_opportunity(&ticker).await?;
                //
                // if let Some(trade) = opportunity {
                //     return Ok(Some(InternalAction::PlaceOrder(trade.into())));
                // }

                Ok(None)
            },
        }
    }
}
