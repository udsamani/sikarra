use tracing::info;

use crate::{engine::InternalEvent, strategy::ArbitrageStrategy};

pub struct SimpleArbitrageStrategy;

impl ArbitrageStrategy for SimpleArbitrageStrategy {
    fn determine_arbitrage_opportunity(&self, event: InternalEvent) -> bool {
        info!("Simple arbitrage strategy: determining opportunity for event: {:?}", event);
        true
    }
}
