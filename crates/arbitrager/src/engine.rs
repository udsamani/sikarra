use sikkara_core::{AppResult, Engine};
use tracing::info;

use crate::{action::InternalAction, event::InternalEvent, strategy::ArbitrageStrategy};

pub struct ArbitrageEngine<S>
where
    S: ArbitrageStrategy,
{
    strategy: S,
    pool: String,
}

impl<S> ArbitrageEngine<S>
where
    S: ArbitrageStrategy,
{
    pub fn new(strategy: S, pool: String) -> Self {
        Self { strategy, pool }
    }
}

#[async_trait::async_trait]
impl<S> Engine<InternalEvent, InternalAction> for ArbitrageEngine<S>
where
    S: ArbitrageStrategy + Send + Sync,
{
    fn id(&self) -> &str {
        "arbitrage_engine"
    }

    async fn process_event(&mut self, event: InternalEvent) -> AppResult<Option<InternalAction>> {
        match event {
            InternalEvent::TickerUpdate(ticker) => {
                info!("Processing ticker update: {:?}", ticker);
            },
        }
        Ok(None)
    }
}
