use anyhow::{anyhow, Result};
use futures::{SinkExt, StreamExt};
use serde_json::json;
use std::sync::Arc;
use tokio::{
    sync::{mpsc, Mutex},
    time::{sleep, Duration},
};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{info, warn};

use super::types::{MessageCallback, MessageHandler};

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

const RETRY_BASE_DELAY: u64 = 5;
const MAX_RETRY_DELAY: u64 = 60;
const MAX_RETRY_TIMES: u32 = 10;

/// OKX WebSocket Client
pub struct OkxWsClient {
    ws_url: String,
    interval: String,
    symbol: String,
    handler: Option<Arc<dyn MessageHandler>>,
    callback: Option<MessageCallback>,
}

impl OkxWsClient {
    pub fn new(
        ws_url: impl Into<String>,
        interval: impl Into<String>,
        symbol: impl Into<String>,
    ) -> Self {
        Self {
            ws_url: ws_url.into(),
            interval: interval.into(),
            symbol: symbol.into(),
            handler: None,
            callback: None,
        }
    }

    pub fn with_handler(mut self, handler: Arc<dyn MessageHandler>) -> Self {
        self.handler = Some(handler);
        self
    }

    pub fn with_callback(mut self, cb: MessageCallback) -> Self {
        self.callback = Some(cb);
        self
    }

    pub async fn run(self) -> Result<()> {
        let mut retry_count = 0;
        let mut delay = RETRY_BASE_DELAY;

        loop {
            match self.connect_and_run().await {
                Ok(_) => {
                    retry_count = 0;
                    delay = RETRY_BASE_DELAY;
                }
                Err(e) => {
                    warn!("【OKX】连接异常: {:?}", e);
                    retry_count += 1;
                    if retry_count >= MAX_RETRY_TIMES {
                        return Err(anyhow!("【OKX】超过最大重连次数"));
                    }
                }
            }

            info!("【OKX】{} 秒后尝试重连...", delay);
            sleep(Duration::from_secs(delay)).await;
            delay = (delay * 2).min(MAX_RETRY_DELAY);
        }
    }

    async fn connect_and_run(&self) -> Result<()> {
        info!("【OKX】连接 WebSocket: {}", self.ws_url);

        let (ws_stream, _) = connect_async(&self.ws_url).await?;
        let (write, mut read) = ws_stream.split();

        let write = Arc::new(Mutex::new(write));

        // 初始订阅
        self.subscribe(&write).await?;

        // 订阅刷新（低频，防止服务端清状态）
        let mut resub_timer = tokio::time::interval(Duration::from_secs(60));

        loop {
            tokio::select! {
                _ = resub_timer.tick() => {
                    self.subscribe(&write).await?;
                }

                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            // OKX 心跳
                            if text == "ping" {
                                let mut w = write.lock().await;
                                w.send(Message::Text("pong".into())).await?;
                                continue;
                            }

                            if let Some(h) = &self.handler {
                                h.handle(&text).await;
                            }

                            if let Some(cb) = &self.callback {
                                cb(&text).await;
                            }
                        }

                        Some(Ok(Message::Close(frame))) => {
                            warn!("【OKX】服务端关闭连接: {:?}", frame);
                            return Err(anyhow!("server closed"));
                        }

                        Some(Err(e)) => {
                            return Err(anyhow!("ws read error: {:?}", e));
                        }

                        None => {
                            return Err(anyhow!("ws stream ended"));
                        }

                        _ => {}
                    }
                }
            }
        }
    }

    async fn subscribe(
        &self,
        write: &Arc<Mutex<futures::stream::SplitSink<WsStream, Message>>>,
    ) -> Result<()> {
        let msg = json!({
            "op": "subscribe",
            "args": [{
                "instType": "SWAP",
                "instId": self.symbol,
                "channel": self.interval
            }]
        })
        .to_string();

        info!("【OKX】发送订阅: {}", msg);

        let mut w = write.lock().await;
        w.send(Message::Text(msg))
            .await
            .map_err(|e| anyhow!("subscribe failed: {:?}", e))
    }
}
