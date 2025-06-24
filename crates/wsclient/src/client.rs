use sikkara_core::{AppError, AppResult, ExponentialBackoff};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

use crate::{callback::WsCallback, consumer::WsConsumer};

#[derive(Debug)]
pub struct WsClient {
    ws_url: String,
    producer: mpsc::Sender<Message>,
    receiver: Option<mpsc::Receiver<Message>>,
    heartbeat_millis: u64,
}

impl WsClient {
    pub fn new(ws_url: String, heartbeat_millis: u64, channel_size: usize) -> Self {
        let (producer, receiver) = mpsc::channel(channel_size);
        WsClient { ws_url, producer, heartbeat_millis, receiver: Some(receiver) }
    }

    pub fn ws_url(&self) -> &str { &self.ws_url }

    pub fn heartbeat_millis(&self) -> u64 { self.heartbeat_millis }

    pub fn write(&self, message: Message) -> AppResult<()> {
        match self.producer.try_send(message) {
            Ok(_) => Ok(()),
            Err(e) => {
                Err(AppError::WebSocketError(format!("failed to send message to websocket: {}", e))
                    .into())
            },
        }
    }

    pub fn close(&self) -> AppResult<()> { self.write(Message::Close(None)) }

    pub fn consumer<C>(&mut self, callback: C) -> AppResult<WsConsumer<C>>
    where
        C: WsCallback + Clone,
    {
        if self.receiver.is_none() {
            return Err(AppError::WebSocketError(
                "WebSocket client receiver is not initialized".to_string(),
            )
            .into());
        }

        let receiver = self.receiver.take().unwrap();

        Ok(WsConsumer {
            ws_url: self.ws_url.clone(),
            callback,
            heartbeat_millis: self.heartbeat_millis,
            backoff: ExponentialBackoff::default(),
            receiver,
        })
    }
}
