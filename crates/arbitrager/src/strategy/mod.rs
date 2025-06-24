mod simple;
pub use simple::SimpleArbitrageStrategy;

use crate::engine::InternalEvent;

pub trait ArbitrageStrategy: Send + Sync {
    fn determine_arbitrage_opportunity(&self, event: InternalEvent) -> bool;
}
