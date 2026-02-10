//! Discord Gateway WebSocket: real-time message updates.

use crate::state::Message;
use dioxus::prelude::spawn;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};

const GATEWAY_URL: &str = "wss://gateway.discord.gg/?v=10&encoding=json";

#[derive(Debug, Deserialize)]
struct GatewayFrame {
    op: u8,
    #[serde(default)]
    d: Option<serde_json::Value>,
    #[serde(default)]
    s: Option<u64>,
    #[serde(default)]
    t: Option<String>,
}

#[derive(Debug, Deserialize)]
struct HelloData {
    heartbeat_interval: u64,
}

#[derive(Serialize)]
struct IdentifyPayload {
    token: String,
    intents: u64,
    properties: IdentifyProperties,
}

#[derive(Serialize)]
struct IdentifyProperties {
    os: String,
    browser: String,
    device: String,
}

#[derive(Debug, Deserialize)]
struct GatewayMessage {
    id: String,
    channel_id: String,
    content: String,
    #[serde(default)]
    timestamp: Option<String>,
    #[serde(default)]
    author: Option<GatewayAuthor>,
}

#[derive(Debug, Deserialize)]
struct GatewayAuthor {
    id: String,
    username: String,
    #[serde(default)]
    global_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TypingStartData {
    channel_id: String,
    user_id: String,
    #[serde(default)]
    timestamp: Option<i64>,
}

/// Spawn Gateway task. Sends new messages and typing events.
/// Uses Dioxus spawn so it runs on the same runtime as the receiver.
pub fn spawn_gateway(
    token: String,
    tx: mpsc::UnboundedSender<Message>,
    tx_typing: Option<mpsc::UnboundedSender<(String, String)>>,
) {
    spawn(async move {
        if let Err(e) = run_gateway_loop(token, tx, tx_typing).await {
            eprintln!("Gateway error: {}", e);
        }
    });
}

async fn run_gateway_loop(
    token: String,
    tx: mpsc::UnboundedSender<Message>,
    tx_typing: Option<mpsc::UnboundedSender<(String, String)>>,
) -> Result<(), String> {
    let (ws_stream, _) = connect_async(GATEWAY_URL).await.map_err(|e| e.to_string())?;
    let (mut write, mut read) = ws_stream.split();
    let mut last_seq: u64 = 0;
    let mut heartbeat_interval: Option<u64> = None;
    let mut identified = false;
    let mut heartbeat = tokio::time::interval(tokio::time::Duration::from_millis(100));
    heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            msg = read.next() => {
                let Some(msg) = msg else { break };
                let msg = msg.map_err(|e| e.to_string())?;
                let text = match msg {
                    WsMessage::Text(t) => t,
                    WsMessage::Close(_) => break,
                    _ => continue,
                };

                let frame: GatewayFrame = serde_json::from_str(&text).map_err(|e| e.to_string())?;

                if let Some(s) = frame.s {
                    last_seq = s;
                }

                match frame.op {
                    10 => {
                        let d: HelloData = serde_json::from_value(frame.d.unwrap_or_default())
                            .map_err(|e| e.to_string())?;
                        heartbeat_interval = Some(d.heartbeat_interval);
                        heartbeat = tokio::time::interval(
                            tokio::time::Duration::from_millis(d.heartbeat_interval),
                        );
                        heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

                        if !identified {
                            let identify = IdentifyPayload {
                                token: token.clone(),
                                intents: 4096 | 16384, // DIRECT_MESSAGES (1<<12) | DIRECT_MESSAGE_TYPING (1<<14)
                                properties: IdentifyProperties {
                                    os: std::env::consts::OS.to_string(),
                                    browser: "Velocity".to_string(),
                                    device: "Velocity".to_string(),
                                },
                            };
                            let payload = serde_json::json!({"op": 2, "d": identify});
                            write
                                .send(WsMessage::Text(payload.to_string()))
                                .await
                                .map_err(|e| e.to_string())?;
                            identified = true;
                        }
                    }
                    0 => {
                        if frame.t.as_deref() == Some("MESSAGE_CREATE") {
                            if let Some(d) = frame.d {
                                if let Ok(gm) = serde_json::from_value::<GatewayMessage>(d) {
                                    let msg = Message {
                                        id: gm.id,
                                        channel_id: gm.channel_id,
                                        author_id: gm.author.as_ref().map(|a| a.id.clone()).unwrap_or_default(),
                                        author_username: gm.author.map(|a| a.global_name.unwrap_or(a.username)),
                                        content: gm.content,
                                        timestamp: gm.timestamp,
                                        sending: false,
                                    };
                                    let _ = tx.send(msg);
                                }
                            }
                        } else if frame.t.as_deref() == Some("TYPING_START") {
                            if let (Some(ref ty), Some(d)) = (&tx_typing, frame.d) {
                                if let Ok(td) = serde_json::from_value::<TypingStartData>(d) {
                                    let _ = ty.send((td.channel_id, td.user_id));
                                }
                            }
                        }
                    }
                    9 => break,
                    _ => {}
                }
            }
            _ = heartbeat.tick() => {
                if identified {
                    let payload = serde_json::json!({"op": 1, "d": last_seq});
                    let _ = write.send(WsMessage::Text(payload.to_string())).await;
                }
            }
        }
    }

    Ok(())
}
