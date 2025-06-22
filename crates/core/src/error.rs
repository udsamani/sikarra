/// A type alias for a result type used throughout the application.
pub type AppResult<T> = anyhow::Result<T>;

/// An enumeration representing various errors that can occur in the
/// application.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// An error that occurs when a piece of code is not implemented
    #[error("Not immplemented: {0}")]
    NotImplemented(String),

    #[error("WebSocket error: {0}")]
    WebSocketError(String),
}
