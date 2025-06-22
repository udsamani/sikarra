use futures_util::{SinkExt, StreamExt};
use sikkara_core::{AppError, AppResult, ExponentialBackoff};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

use crate::callback::WsCallback;

pub struct WsConsumer<C>
where
    C: WsCallback + Clone,
{
    pub ws_url: String,
    pub callback: C,
    pub heartbeat_millis: u64,
    pub backoff: ExponentialBackoff,
    pub receiver: mpsc::Receiver<Message>,
}

impl<C> WsConsumer<C>
where
    C: WsCallback + Clone,
{
    pub async fn run(&mut self, shutdown: CancellationToken) -> AppResult<()> {
        loop {
            match self.backoff.next() {
                Some(backoff_secs) => {
                    if backoff_secs > 0 {
                        tokio::time::sleep(tokio::time::Duration::from_secs(backoff_secs as u64))
                            .await;
                    }
                },
                None => {
                    return Err(AppError::WebSocketError(format!(
                        "failed to connect to {} after {} retries",
                        self.ws_url,
                        self.backoff.get_iteration_count()
                    ))
                    .into());
                },
            }

            info!("connecting to websocket at {}", self.ws_url);
            let ws_stream = match connect_async(&self.ws_url).await {
                Ok((ws_stream, _)) => {
                    info!("connected to websocket at {}", self.ws_url);
                    self.backoff.reset();
                    ws_stream
                },
                Err(e) => {
                    error!("failed to connect to websocket at {}: {} will retry", self.ws_url, e);
                    continue;
                },
            };

            let stream_result = self.stream(ws_stream, shutdown.child_token()).await;
            self.callback.on_disconnect()?;

            match stream_result {
                Ok(_) => {
                    info!("websocket connection closed gracefully");
                    return Ok(());
                },
                Err(e) => {
                    error!("websocket connection lost: {} will retry", e);
                    continue;
                },
            }
        }
    }

    async fn stream<S>(
        &mut self,
        mut ws_stream: WebSocketStream<S>,
        shutdown: CancellationToken,
    ) -> AppResult<()>
    where
        S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
    {
        self.callback.on_connect(jiff::Timestamp::now()).await?;
        let mut num_message_since_last_heartbeat = 0;
        let mut heartbeat =
            tokio::time::interval(tokio::time::Duration::from_millis(self.heartbeat_millis));

        loop {
            tokio::select! {
                _ = shutdown.cancelled() => {
                    // Shutdown requested, send a close frame to the WebSocket
                    if let Err(e) = ws_stream.send(Message::Close(None)).await {
                        return Err(AppError::WebSocketError(format!("failed to send close frame: {}", e)).into());
                    }
                    return Ok(());
                }

                result = ws_stream.next() => {
                    // A message was received from the Websocket handle it

                    match result {
                        Some(result) => {
                            match result {
                                Ok(message) => {
                                    let recieved_time = jiff::Timestamp::now();
                                    num_message_since_last_heartbeat += 1;
                                    self.callback.on_message(message, recieved_time).await?;
                                },
                                Err(e) => {
                                    return Err(AppError::WebSocketError(format!("websocket streaming error: {}", e)).into());
                                }
                            }
                        },
                        None => {
                            return Err(AppError::WebSocketError("websocket stream closed unexpectedly".to_string()).into());
                        }
                    }
                }

                result = self.receiver.recv() => {
                    // Handle the incoming message from the receiver aka sending a request to the websocket
                    match result {
                        Some(message) => {
                            if let Err(e) = ws_stream.send(message).await {
                                return Err(AppError::WebSocketError(format!("failed to send message: {}", e)).into());
                            }
                        },
                        None => {
                            return Err(AppError::WebSocketError("receiver channel closed unexpectedly".to_string()).into());
                        }
                    }
                }


                _ = heartbeat.tick() => {
                    // Heartbeat tick
                    let _ = self.callback.on_heartbeat();
                    if num_message_since_last_heartbeat > 0 {
                        info!("number of messages received since last heartbeat: {}", num_message_since_last_heartbeat);
                        num_message_since_last_heartbeat = 0;
                    }
                }
            }
        }

        todo!()
    }
}
