use anyhow::anyhow;
use anyhow::Result;
use futures::{SinkExt, StreamExt};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::Instant;
use tokio::{
    sync::mpsc,
    time::{sleep, Duration},
};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::info;

use super::types::{MessageCallback, MessageHandler};

const RETRY_DELAY: u64 = 5;
const MAX_RETRY_ATTEMPTS: u32 = 10;
const MAX_RETRY_DELAY: u64 = 60;

/// 支持每个 symbol 对应不同 interval 的 handler 版本
pub async fn run_with_handler(
    wss_domain: &str,
    subscriptions: HashMap<&str, Vec<&str>>,
    handler: Arc<dyn MessageHandler>,
) -> Result<()> {
    run_internal(wss_domain, subscriptions, Some(handler), None).await
}

/// 支持每个 symbol 对应不同 interval 的 callback 版本
pub async fn run_with_callback(
    wss_domain: &str,
    subscriptions: HashMap<&str, Vec<&str>>,
    callback: MessageCallback,
) -> Result<()> {
    run_internal(wss_domain, subscriptions, None, Some(callback)).await
}

async fn run_internal(
    wss_domain: &str,
    subscriptions: HashMap<&str, Vec<&str>>,
    handler: Option<Arc<dyn MessageHandler>>,
    callback: Option<MessageCallback>,
) -> Result<()> {
    let mut retry = 0;
    let mut delay = RETRY_DELAY;

    // 构建所有订阅消息
    let mut subscribe_msgs = vec![];
    for (symbol, intervals) in &subscriptions {
        for interval in intervals {
            subscribe_msgs.push(Message::Text(
                json!({
                    "op": "subscribe",
                    "args": [{
                        "instId": symbol,
                        "channel": interval,
                        "instType": "SWAP"
                    }]
                })
                .to_string(),
            ));
        }
    }

    loop {
        info!("【OKX】connecting...");
        match connect_async(wss_domain).await {
            Ok((ws, _)) => {
                let (mut write_half, mut read_half) = ws.split();
                let (write_tx, mut write_rx) = mpsc::channel::<Message>(512);

                // writer
                let writer = tokio::spawn(async move {
                    while let Some(msg) = write_rx.recv().await {
                        if let Err(e) = write_half.send(msg).await {
                            return Err(anyhow!("write error: {:?}", e));
                        }
                    }
                    Ok::<_, anyhow::Error>(())
                });

                // 初始订阅
                for msg in &subscribe_msgs {
                    write_tx.send(msg.clone()).await?;
                }

                // heartbeat
                let hb_tx = write_tx.clone();
                tokio::spawn(async move {
                    let mut t = tokio::time::interval(Duration::from_secs(20));
                    loop {
                        t.tick().await;
                        if hb_tx.send(Message::Ping(vec![])).await.is_err() {
                            break;
                        }
                    }
                });

                // resubscribe
                let rs_tx = write_tx.clone();
                let resub_msgs = subscribe_msgs.clone();
                tokio::spawn(async move {
                    let mut t = tokio::time::interval(Duration::from_secs(60));
                    loop {
                        t.tick().await;
                        for msg in &resub_msgs {
                            let _ = rs_tx.send(msg.clone()).await;
                        }
                    }
                });

                let mut last_msg = Instant::now();

                while let Some(msg) = read_half.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                            last_msg = Instant::now();
                            if text == "ping" {
                                let _ = write_tx.send(Message::Text("pong".into())).await;
                                continue;
                            }
                            if let Some(h) = &handler {
                                h.handle(&text).await;
                            }
                            if let Some(cb) = &callback {
                                cb(&text).await;
                            }
                        }
                        Ok(Message::Pong(_)) => last_msg = Instant::now(),
                        Ok(Message::Close(_)) => break,
                        Err(e) => return Err(anyhow!("read error: {:?}", e)),
                        _ => {}
                    }

                    if last_msg.elapsed() > Duration::from_secs(90) {
                        return Err(anyhow!("heartbeat timeout"));
                    }
                }

                let _ = writer.abort();
            }
            Err(e) => {
                info!("connect error: {:?}", e);
            }
        }

        retry += 1;
        if retry >= MAX_RETRY_ATTEMPTS {
            break;
        }

        sleep(Duration::from_secs(delay)).await;
        delay = (delay * 2).min(MAX_RETRY_DELAY);
    }

    Ok(())
}
