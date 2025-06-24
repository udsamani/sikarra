mod logging;
pub use logging::LoggingBotStrategy;

use crate::engine::InternalEvent;

pub trait BotStrategy: Send + Sync {
    fn handle_internal_event(&mut self, event: InternalEvent);
}
