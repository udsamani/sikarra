mod logging;
pub use logging::LogginArbitrageStrategy;

use crate::engine::InternalEvent;

pub trait ArbitrageStrategy: Send + Sync {
    fn determine_arbitrage_opportunity(&mut self, event: InternalEvent);
}
