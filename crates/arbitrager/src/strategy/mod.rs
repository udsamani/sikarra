mod simple;
pub use simple::SimpleArbitrageStrategy;

pub trait ArbitrageStrategy: Send + Sync {
    fn determine_arbitrage_opportunity(&self, event: &crate::event::InternalEvent) -> bool;
}
