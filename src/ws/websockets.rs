use anyhow::{anyhow, Result};
use futures::{SinkExt, StreamExt};
use serde_json::json;
use std::sync::Arc;
use tokio::{
    select,
    sync::mpsc,
    time::{sleep, Duration, Instant},
};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{info, warn};

use super::types::{MessageCallback, MessageHandler};

const RETRY_DELAY: u64 = 5;
const MAX_RETRY_ATTEMPTS: u32 = 10;
const MAX_RETRY_DELAY: u64 = 60;

const PING_INTERVAL: u64 = 20;
const READ_TIMEOUT: u64 = 60;

async fn connect(ws_url: &str) -> Result<tokio_tungstenite::WebSocketStream<_>> {
    info!("【OKX】连接 {}", ws_url);
    let (ws, _) = connect_async(ws_url).await?;
    Ok(ws)
}

fn subscribe_msg(interval: &str, symbol: &str) -> Message {
    Message::Text(
        json!({
            "op": "subscribe",
            "args": [{
                "instType": "SWAP",
                "instId": symbol,
                "channel": interval
            }]
        })
        .to_string(),
    )
}

async fn write_task<W>(mut write: W, mut rx: mpsc::Receiver<Message>) -> Result<()>
where
    W: SinkExt<Message> + Unpin,
    W::Error: std::fmt::Debug,
{
    while let Some(msg) = rx.recv().await {
        write
            .send(msg)
            .await
            .map_err(|e| anyhow!("【OKX】写入失败: {:?}", e))?;
    }
    Ok(())
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
    let mut retry = 0;
    let mut delay = RETRY_DELAY;

    loop {
        match connect(wss_domain).await {
            Ok(ws) => {
                let (write, mut read) = ws.split();
                let (tx, rx) = mpsc::channel::<Message>(128);

                // 启动写任务
                let writer = tokio::spawn(write_task(write, rx));

                // 初始订阅
                tx.send(subscribe_msg(interval, symbol)).await.ok();

                let mut last_msg = Instant::now();
                let mut ping_timer = tokio::time::interval(Duration::from_secs(PING_INTERVAL));

                loop {
                    select! {
                        _ = ping_timer.tick() => {
                            if last_msg.elapsed() > Duration::from_secs(READ_TIMEOUT) {
                                warn!("【OKX】读超时，触发重连");
                                break;
                            }
                            tx.send(Message::Text("ping".into())).await.ok();
                        }

                        msg = read.next() => {
                            match msg {
                                Some(Ok(Message::Text(text))) => {
                                    last_msg = Instant::now();

                                    if text == "pong" {
                                        continue;
                                    }

                                    if let Some(ref h) = handler {
                                        h.handle(&text).await;
                                    }
                                    if let Some(ref cb) = callback {
                                        cb(&text).await;
                                    }
                                }

                                Some(Ok(Message::Close(f))) => {
                                    info!("【OKX】服务端关闭: {:?}", f);
                                    break;
                                }

                                Some(Err(e)) => {
                                    warn!("【OKX】读取错误: {:?}", e);
                                    break;
                                }

                                None => {
                                    warn!("【OKX】连接断开");
                                    break;
                                }

                                _ => {}
                            }
                        }
                    }
                }

                writer.abort();
                retry = 0;
                delay = RETRY_DELAY;
            }

            Err(e) => {
                warn!("【OKX】连接失败: {:?}", e);
            }
        }

        retry += 1;
        if retry >= MAX_RETRY_ATTEMPTS {
            warn!("【OKX】超过最大重试次数");
            break;
        }

        info!("【OKX】{} 秒后重连", delay);
        sleep(Duration::from_secs(delay)).await;
        delay = (delay * 2).min(MAX_RETRY_DELAY);
    }

    Ok(())
}
