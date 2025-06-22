use crate::models::Ticker;

#[derive(Debug, Clone)]
pub enum InternalEvent {
    TickerUpdate(Ticker),
}
