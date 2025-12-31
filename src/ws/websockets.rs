use anyhow::anyhow;
use anyhow::Result;
use futures::{SinkExt, StreamExt};
use serde_json::json;
use std::sync::Arc;
use tokio::time::Instant;
use tokio::{
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

// async fn run_internal(
//     wss_domain: &str,
//     interval: &str,
//     symbol: &str,
//     handler: Option<Arc<dyn MessageHandler>>,
//     callback: Option<MessageCallback>,
// ) -> Result<()> {
//     info!("初始化 【OKX】 WebSocket...");

//     let mut retry_count = 0;
//     let mut retry_delay = RETRY_DELAY;

//     loop {
//         match connect_websocket(wss_domain).await {
//             Ok((ws_stream, _tx, _rx)) => {
//                 let (write_half, mut read_half) = ws_stream.split();
//                 let write = Arc::new(Mutex::new(write_half));

//                 // 1️⃣ 初始订阅
//                 {
//                     let mut w = write.lock().await;
//                     subscribe_channel(&mut *w, interval, symbol).await?;
//                 }

//                 retry_count = 0;
//                 retry_delay = RETRY_DELAY;

//                 // 2️⃣ 低频订阅刷新（避免 OKX 清状态）
//                 let mut resub_timer = tokio::time::interval(Duration::from_secs(60));

//                 // 3️⃣ 主循环
//                 loop {
//                     tokio::select! {
//                         _ = resub_timer.tick() => {
//                             let mut w = write.lock().await;
//                             if let Err(e) = subscribe_channel(&mut *w, interval, symbol).await {
//                                 info!("【OKX】订阅刷新失败: {:?}", e);
//                             }
//                         }

//                         msg = read_half.next() => {
//                             match msg {
//                                 Some(Ok(Message::Text(text))) => {
//                                     // ✅ OKX 心跳处理
//                                     if text == "ping" {
//                                         let mut w = write.lock().await;
//                                         if let Err(e) = w.send(Message::Text("pong".into())).await {
//                                             info!("【OKX】pong 发送失败: {:?}", e);
//                                             break;
//                                         }
//                                         continue;
//                                     }

//                                     if let Some(ref h) = handler {
//                                         h.handle(&text).await;
//                                     }

//                                     if let Some(ref cb) = callback {
//                                         cb(&text).await;
//                                     }
//                                 }

//                                 Some(Ok(Message::Close(frame))) => {
//                                     info!("【OKX】服务端关闭连接: {:?}", frame);
//                                     break;
//                                 }

//                                 Some(Err(e)) => {
//                                     info!("【OKX】读取消息失败: {:?}", e);
//                                     break;
//                                 }

//                                 None => {
//                                     info!("【OKX】WebSocket 流结束");
//                                     break;
//                                 }

//                                 _ => {}
//                             }
//                         }
//                     }
//                 }
//             }

//             Err(e) => {
//                 info!("【OKX】连接失败: {:?}", e);
//             }
//         }

//         retry_count += 1;
//         if retry_count >= MAX_RETRY_ATTEMPTS {
//             info!("【OKX】达到最大重试次数，退出");
//             break;
//         }

//         info!("【OKX】{} 秒后重连...", retry_delay);
//         sleep(Duration::from_secs(retry_delay)).await;
//         retry_delay = (retry_delay * 2).min(MAX_RETRY_DELAY);
//     }

//     Ok(())
// }

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
        info!("【OKX】connecting...");
        match connect_async(wss_domain).await {
            Ok((ws, _)) => {
                let (mut write_half, mut read_half) = ws.split();
                let (write_tx, mut write_rx) = mpsc::channel::<Message>(512);

                let subscribe_msg = Message::Text(
                    json!({
                        "op": "subscribe",
                        "args": [{
                            "instId": symbol,
                            "channel": interval,
                            "instType": "SWAP"
                        }]
                    })
                    .to_string(),
                );

                // writer
                let writer = tokio::spawn(async move {
                    while let Some(msg) = write_rx.recv().await {
                        write_half.send(msg).await?;
                    }
                    Ok::<_, anyhow::Error>(())
                });

                write_tx.send(subscribe_msg.clone()).await?;

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

                // resub
                let rs_tx = write_tx.clone();
                tokio::spawn(async move {
                    let mut t = tokio::time::interval(Duration::from_secs(60));
                    loop {
                        t.tick().await;
                        let _ = rs_tx.send(subscribe_msg.clone()).await;
                    }
                });

                let mut last_msg = tokio::time::Instant::now();

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
