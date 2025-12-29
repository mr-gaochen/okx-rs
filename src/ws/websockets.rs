use anyhow::anyhow;
use anyhow::Result;
use futures::{SinkExt, StreamExt};
use serde_json::json;
use std::sync::Arc;
use tokio::{
    select,
    sync::{mpsc, Mutex},
    time::{sleep, Duration},
};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::info;

use super::types::{MessageCallback, MessageHandler};

const RETRY_DELAY: u64 = 5;
const MAX_RETRY_ATTEMPTS: u32 = 10;
const MAX_RETRY_DELAY: u64 = 60;

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

async fn connect_websocket(
    ws_url: &str,
) -> Result<(WsStream, mpsc::Sender<Message>, mpsc::Receiver<Message>)> {
    info!("websocket connecting to {}", ws_url);
    let (ws_stream, _) = connect_async(ws_url).await?;
    let (tx, rx) = mpsc::channel(100);
    Ok((ws_stream, tx, rx))
}

async fn subscribe_channel<S>(write: &mut S, interval: &str, symbol: &str) -> Result<()>
where
    S: SinkExt<Message> + Unpin,
    S::Error: std::fmt::Debug, // 加上这句约束
{
    let subscribe_msg = json!({
        "op": "subscribe",
        "args": [{
            "instId":symbol,
            "channel": interval,
            "instType": "SWAP"
        }
        ]
    })
    .to_string();
    info!("订阅消息:{:?}", subscribe_msg);
    write
        .send(Message::Text(subscribe_msg))
        .await
        .map_err(|e| anyhow!("【OKX】订阅消息发送失败: {:?}", e))
}

pub async fn run_with_handler(
    wss_domain: &str,
    interval: &str,
    symbol: &str,
    handler: Arc<dyn MessageHandler>,
) -> Result<()> {
    run_internal(wss_domain, interval, symbol, Some(handler), None).await
}

pub async fn run_with_callback(
    wss_domain: &str,
    interval: &str,
    symbol: &str,
    callback: MessageCallback,
) -> Result<()> {
    run_internal(wss_domain, interval, symbol, None, Some(callback)).await
}

async fn run_internal(
    wss_domain: &str,
    interval: &str,
    symbol: &str,
    handler: Option<Arc<dyn MessageHandler>>,
    callback: Option<MessageCallback>,
) -> Result<()> {
    let mut retry_delay = RETRY_DELAY;
    let mut retry_count = 0;

    loop {
        info!("【OKX】connecting {}", wss_domain);

        match connect_async(wss_domain).await {
            Ok((ws_stream, _)) => {
                info!("【OKX】TCP + TLS connected");

                let (mut write, mut read) = ws_stream.split();

                // ✅ 等待服务端首条消息（非常关键）
                match read.next().await {
                    Some(Ok(Message::Text(text))) => {
                        info!("【OKX】server hello: {}", text);
                    }
                    other => {
                        info!("【OKX】no hello msg: {:?}", other);
                        continue;
                    }
                }

                // ✅ 只订阅一次
                subscribe_channel(&mut write, interval, symbol).await?;

                retry_delay = RETRY_DELAY;
                retry_count = 0;

                loop {
                    match read.next().await {
                        Some(Ok(Message::Text(text))) => {
                            if text == "ping" {
                                write.send(Message::Text("pong".into())).await?;
                                continue;
                            }

                            if let Some(ref h) = handler {
                                h.handle(&text).await;
                            }

                            if let Some(ref cb) = callback {
                                cb(&text).await;
                            }
                        }

                        Some(Ok(Message::Close(frame))) => {
                            info!("【OKX】server close: {:?}", frame);
                            break;
                        }

                        Some(Err(e)) => {
                            info!("【OKX】read error: {:?}", e);
                            break;
                        }

                        None => break,
                        _ => {}
                    }
                }
            }

            Err(e) => {
                info!("【OKX】connect failed: {:?}", e);
            }
        }

        retry_count += 1;
        if retry_count >= MAX_RETRY_ATTEMPTS {
            return Err(anyhow!("max retry reached"));
        }

        info!("【OKX】reconnect in {}s", retry_delay);
        sleep(Duration::from_secs(retry_delay)).await;
        retry_delay = (retry_delay * 2).min(MAX_RETRY_DELAY);
    }
}
