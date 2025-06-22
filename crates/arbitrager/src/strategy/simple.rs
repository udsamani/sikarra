use tracing::info;

use crate::strategy::ArbitrageStrategy;

pub struct SimpleArbitrageStrategy;

impl ArbitrageStrategy for SimpleArbitrageStrategy {
    fn determine_arbitrage_opportunity(&self, event: &crate::event::InternalEvent) -> bool {
        info!("Simple arbitrage strategy: determining opportunity for event: {:?}", event);
        true
    }
}
