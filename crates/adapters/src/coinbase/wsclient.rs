use sikkara_core::{AppError, AppResult};
use sikkara_wsclient::WsCallback;
use tokio::sync::{broadcast, mpsc};
use tokio_tungstenite::tungstenite::{Message, Utf8Bytes};
use tracing::{debug, error, info, warn};

use crate::coinbase::{
    models::CoinbaseSymbol, CoinbaseMessage, CoinbaseRequest, CoinbaseRequestType,
};

#[derive(Debug, Clone)]
pub struct CoinbaseWsClient {
    ws_url: String,
    sender: mpsc::Sender<Message>,
    message_broadcaster: broadcast::Sender<CoinbaseMessage>,
}

impl CoinbaseWsClient {
    pub fn new(
        ws_url: String,
        sender: mpsc::Sender<Message>,
        message_broadcaster: broadcast::Sender<CoinbaseMessage>,
    ) -> Self {
        CoinbaseWsClient { ws_url, sender, message_broadcaster }
    }

    pub fn subscribe(
        &self,
        product_ids: Vec<CoinbaseSymbol>,
        channels: Vec<String>,
    ) -> AppResult<broadcast::Receiver<CoinbaseMessage>> {
        let request =
            CoinbaseRequest { request_type: CoinbaseRequestType::Subscribe, product_ids, channels };
        let message = serde_json::to_string(&request)?;

        self.write(Message::Text(Utf8Bytes::from(&message)));
        Ok(self.message_broadcaster.subscribe())
    }
    pub fn ws_url(&self) -> &str {
        &self.ws_url
    }

    pub fn write(&self, message: Message) -> AppResult<()> {
        match self.sender.try_send(message) {
            Ok(_) => Ok(()),
            Err(e) => {
                Err(AppError::WebSocketError(format!("failed to send message to websocket: {}", e))
                    .into())
            },
        }
    }

    pub fn close(&self) -> AppResult<()> {
        self.write(Message::Close(None))
    }
}

#[async_trait::async_trait]
impl WsCallback for CoinbaseWsClient {
    async fn on_connect(&mut self, timestamp: jiff::Timestamp) -> AppResult<()> {
        info!("Connected to Coinbase WebSocket at {}", self.ws_url);
        Ok(())
    }

    async fn on_message(&mut self, message: Message, receive_at: jiff::Timestamp) -> AppResult<()> {
        match message {
            Message::Text(text) => {
                let coinbase_message: CoinbaseMessage = match serde_json::from_str(&text) {
                    Ok(msg) => msg,
                    Err(e) => {
                        error!("Failed to parse Coinbase message: {}", e);
                        return Err(AppError::WebSocketError(format!(
                            "Failed to parse Coinbase message: {}",
                            e
                        ))
                        .into());
                    },
                };
                self.message_broadcaster
                    .send(coinbase_message.clone())
                    .map_err(|e| {
                        AppError::WebSocketError(format!("Failed to broadcast message: {}", e))
                    })?;
            },
            Message::Close(_) => {
                info!("WebSocket connection closed by remote peer");
                return self.on_disconnect();
            },

            Message::Ping(ping) => {
                self.write(Message::Pong(ping))?;
            },

            _ => {
                warn!("Received unsupported message type: {:?}", message);
            },
        };
        Ok(())
    }

    fn on_disconnect(&mut self) -> AppResult<()> {
        info!("WebSocket connection closed or lost");
        Ok(())
    }

    fn on_heartbeat(&mut self) -> AppResult<()> {
        info!("Heartbeat check for Coinbase WebSocket connection");
        Ok(())
    }
}
