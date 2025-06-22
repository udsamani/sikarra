use sikkara_core::AppResult;
use tokio_tungstenite::tungstenite::Message;

/// A trait for handling WebSocket connection lifecycle events and messages.
///
/// This trait provides a callback interface for WebSocket clients to respond to
/// various connection states and incoming messages. Implementations should
/// handle each event appropriately based on the application's requirements.
#[async_trait::async_trait]
pub trait WsCallback {
    /// Called when a WebSocket connection is successfully established.
    ///
    /// This method is invoked immediately after the WebSocket handshake
    /// completes and the connection is ready to send and receive messages.
    ///
    /// # Parameters
    ///
    /// * `timestamp` - The exact time when the connection was established
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the connection handling succeeds, or an error if
    /// initialization fails. Returning an error may cause the connection to be
    /// closed.
    async fn on_connect(&mut self, timestamp: jiff::Timestamp) -> AppResult<()>;

    /// Called when a message is received from the WebSocket connection.
    ///
    /// This method handles all incoming WebSocket messages, including text,
    /// binary, ping, pong, and close frames. The implementation should
    /// process the message according to the application protocol.
    ///
    /// # Parameters
    ///
    /// * `message` - The WebSocket message received from the remote peer
    /// * `receive_at` - The timestamp when the message was received
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the message is processed successfully, or an error
    /// if message handling fails. Errors may trigger connection closure
    /// depending on the client implementation.
    async fn on_message(&mut self, message: Message, receive_at: jiff::Timestamp) -> AppResult<()>;

    /// Called when the WebSocket connection is closed or lost.
    ///
    /// This method is invoked when the connection terminates, either gracefully
    /// (via a close frame) or unexpectedly (due to network issues, timeouts,
    /// etc.). Use this callback to perform cleanup operations and handle
    /// reconnection logic.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if cleanup succeeds, or an error if disconnection
    /// handling fails. Errors in this method are typically logged but don't
    /// affect the connection state since the connection is already closed.
    fn on_disconnect(&mut self) -> AppResult<()>;

    /// Called periodically to maintain connection health.
    ///
    /// This method is invoked at regular intervals to perform heartbeat
    /// operations, such as sending ping frames or application-level
    /// keep-alive messages. The frequency depends on the WebSocket client
    /// configuration.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the heartbeat succeeds, or an error if heartbeat
    /// operations fail. Heartbeat failures may trigger connection closure
    /// or reconnection attempts.
    fn on_heartbeat(&mut self) -> AppResult<()>;
}
