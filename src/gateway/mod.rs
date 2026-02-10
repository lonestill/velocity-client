//! Discord Gateway WebSocket: real-time message updates.

use crate::state::{Message, PresenceStatus};
use dioxus::prelude::spawn;
use tokio::sync::mpsc::UnboundedSender;
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
    presence: PresenceData,
}

#[derive(Serialize)]
struct IdentifyProperties {
    os: String,
    browser: String,
    device: String,
}

#[derive(Serialize)]
struct PresenceData {
    since: Option<u64>,
    activities: Vec<serde_json::Value>,
    status: String,
    afk: bool,
}

fn presence_to_payload(status: PresenceStatus) -> PresenceData {
    let (status_str, afk) = match status {
        PresenceStatus::Online => ("online", false),
        PresenceStatus::Idle => ("idle", true),
        PresenceStatus::DoNotDisturb => ("dnd", false),
        PresenceStatus::Invisible => ("invisible", false),
    };
    PresenceData {
        since: None,
        activities: Vec::new(),
        status: status_str.to_string(),
        afk,
    }
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

/// Discord sends user.id as snowflake (number or string in JSON).
fn deserialize_snowflake_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let v = serde_json::Value::deserialize(deserializer)?;
    match v {
        serde_json::Value::String(s) => Ok(s),
        serde_json::Value::Number(n) => Ok(n.to_string()),
        _ => Err(Error::custom("expected string or number for snowflake")),
    }
}

#[derive(Debug, Deserialize)]
struct PresenceUpdateUser {
    #[serde(deserialize_with = "deserialize_snowflake_string")]
    id: String,
}

#[derive(Debug, Deserialize)]
struct PresenceUpdateData {
    user: PresenceUpdateUser,
    #[serde(default)]
    status: Option<String>,
}

/// Data needed to connect the voice driver (from VOICE_STATE_UPDATE + VOICE_SERVER_UPDATE).
#[derive(Clone, Debug)]
pub struct VoiceConnectionInfo {
    pub guild_id: Option<String>,
    pub channel_id: String,
    pub endpoint: String,
    pub token: String,
    pub session_id: String,
    pub user_id: String,
}

/// Commands from app to gateway for voice.
#[derive(Clone, Debug)]
pub enum VoiceCommand {
    Join {
        guild_id: Option<String>,
        channel_id: String,
        self_mute: bool,
        self_deaf: bool,
        input_device: Option<String>,
        output_device: Option<String>,
    },
    Leave,
}

/// Message to the voice task: connect with info and optional device names, or leave.
#[derive(Clone, Debug)]
pub enum VoiceMessage {
    Connect {
        info: VoiceConnectionInfo,
        input_device: Option<String>,
        output_device: Option<String>,
    },
    Leave,
}

#[derive(Debug, Deserialize)]
struct VoiceStateUpdateData {
    #[serde(default, deserialize_with = "deserialize_snowflake_string_opt")]
    pub guild_id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_snowflake_string_opt")]
    pub channel_id: Option<String>,
    #[serde(deserialize_with = "deserialize_snowflake_string")]
    pub user_id: String,
    pub session_id: String,
}

#[derive(Debug, Deserialize)]
struct VoiceServerUpdateData {
    pub token: String,
    #[serde(default, deserialize_with = "deserialize_snowflake_string_opt")]
    pub guild_id: Option<String>,
    pub endpoint: Option<String>,
}

fn deserialize_snowflake_string_opt<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v: Option<serde_json::Value> = Option::deserialize(deserializer)?;
    match v {
        None => Ok(None),
        Some(serde_json::Value::String(s)) => Ok(Some(s)),
        Some(serde_json::Value::Number(n)) => Ok(Some(n.to_string())),
        _ => Ok(None),
    }
}

/// Spawn Gateway task. Sends new messages, typing events, presence and voice.
pub fn spawn_gateway(
    token: String,
    current_user_id: Option<String>,
    tx: mpsc::UnboundedSender<Message>,
    tx_typing: Option<mpsc::UnboundedSender<(String, String)>>,
    tx_presence_updates: Option<mpsc::UnboundedSender<(String, String)>>,
    presence: PresenceStatus,
    presence_rx: mpsc::UnboundedReceiver<PresenceStatus>,
    rx_voice_cmd: mpsc::UnboundedReceiver<VoiceCommand>,
    tx_voice_message: UnboundedSender<VoiceMessage>,
) {
    spawn(async move {
        if let Err(e) = run_gateway_loop(
            token,
            current_user_id,
            tx,
            tx_typing,
            tx_presence_updates,
            presence,
            presence_rx,
            rx_voice_cmd,
            tx_voice_message,
        )
        .await
        {
            eprintln!("Gateway error: {}", e);
        }
    });
}

async fn run_gateway_loop(
    token: String,
    current_user_id: Option<String>,
    tx: mpsc::UnboundedSender<Message>,
    tx_typing: Option<mpsc::UnboundedSender<(String, String)>>,
    tx_presence_updates: Option<mpsc::UnboundedSender<(String, String)>>,
    presence: PresenceStatus,
    mut presence_rx: mpsc::UnboundedReceiver<PresenceStatus>,
    mut rx_voice_cmd: mpsc::UnboundedReceiver<VoiceCommand>,
    tx_voice_message: UnboundedSender<VoiceMessage>,
) -> Result<(), String> {
    let (ws_stream, _) = connect_async(GATEWAY_URL).await.map_err(|e| e.to_string())?;
    let (mut write, mut read) = ws_stream.split();
    let mut last_seq: u64 = 0;
    let mut heartbeat_interval: Option<u64> = None;
    let mut identified = false;
    let mut current_presence = presence;
    let mut heartbeat = tokio::time::interval(tokio::time::Duration::from_millis(100));
    heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    // Voice: we send op 4 on Join/Leave; when we get our VOICE_STATE_UPDATE + VOICE_SERVER_UPDATE we send ConnectionInfo.
    let mut current_voice: Option<(Option<String>, String, bool, bool, Option<String>, Option<String>)> = None; // guild_id, channel_id, mute, deaf, input_device, output_device
    let mut my_voice_session: Option<(String, String)> = None; // (session_id, channel_id)
    let mut last_voice_server: Option<(String, Option<String>, Option<String>)> = None; // (token, endpoint, guild_id)

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
                                intents: 4096 | 16384 | 256 | 128 | 2, // DM | DM_TYPING | GUILD_VOICE_STATES | GUILD_MEMBERS | ...
                                properties: IdentifyProperties {
                                    os: std::env::consts::OS.to_string(),
                                    browser: "Velocity".to_string(),
                                    device: "Velocity".to_string(),
                                },
                                presence: presence_to_payload(current_presence),
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
                        } else if frame.t.as_deref() == Some("PRESENCE_UPDATE") {
                            if let (Some(ref tx_pres), Some(d)) = (&tx_presence_updates, frame.d.clone()) {
                                let raw_str = d.to_string();
                                match serde_json::from_value::<PresenceUpdateData>(d) {
                                    Ok(pu) => {
                                        let status = pu.status.unwrap_or_else(|| "offline".to_string());
                                        eprintln!("[presence] user_id={} status={}", pu.user.id, status);
                                        let _ = tx_pres.send((pu.user.id, status));
                                    }
                                    Err(e) => {
                                        let preview = if raw_str.len() > 400 {
                                            format!("{}...", &raw_str[..400])
                                        } else {
                                            raw_str
                                        };
                                        eprintln!("[presence] parse error: {}", e);
                                        eprintln!("[presence] raw d: {}", preview);
                                    }
                                }
                            }
                        } else if frame.t.as_deref() == Some("VOICE_STATE_UPDATE") {
                            if let Some(d) = &frame.d {
                                match serde_json::from_value::<VoiceStateUpdateData>(d.clone()) {
                                    Ok(vs) => {
                                        let is_self = current_user_id.as_ref().map(|my_id| vs.user_id == *my_id).unwrap_or(false);
                                        eprintln!("[voice gateway] VOICE_STATE_UPDATE user_id={} channel_id={:?} is_self={}",
                                            vs.user_id, vs.channel_id, is_self);
                                        if is_self {
                                            let my_id = current_user_id.as_ref().unwrap();
                                            let ch = vs.channel_id.unwrap_or_default();
                                            my_voice_session = Some((vs.session_id.clone(), ch.clone()));
                                            eprintln!("[voice gateway] VOICE_STATE_UPDATE self session_id={} channel_id={} last_voice_server={}",
                                                vs.session_id, ch, last_voice_server.is_some());
                                            if let (Some(ref tx_v), Some((token, endpoint, gid)), Some((session_id, channel_id))) =
                                                (Some(&tx_voice_message), last_voice_server.clone(), my_voice_session.clone())
                                            {
                                                if !channel_id.is_empty() {
                                                    if let Some(ep) = endpoint {
                                                        let info = VoiceConnectionInfo {
                                                            guild_id: gid.clone(),
                                                            channel_id: channel_id.clone(),
                                                            endpoint: ep,
                                                            token: token.clone(),
                                                            session_id: session_id.clone(),
                                                            user_id: my_id.clone(),
                                                        };
                                                    let (in_dev, out_dev) = current_voice.as_ref()
                                                        .map(|(_, _, _, _, i, o)| (i.clone(), o.clone()))
                                                        .unwrap_or((None, None));
                                                    eprintln!("[voice gateway] sending Connect (from VOICE_STATE_UPDATE) channel={}", info.channel_id);
                                                    let _ = tx_v.send(VoiceMessage::Connect {
                                                        info,
                                                        input_device: in_dev,
                                                        output_device: out_dev,
                                                    });
                                                    last_voice_server = None;
                                                    my_voice_session = None;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("[voice gateway] VOICE_STATE_UPDATE parse error: {}", e);
                                    }
                                }
                            }
                        } else if frame.t.as_deref() == Some("VOICE_SERVER_UPDATE") {
                            if let Some(d) = frame.d {
                                match serde_json::from_value::<VoiceServerUpdateData>(d.clone()) {
                                    Ok(vs) => {
                                        eprintln!("[voice gateway] VOICE_SERVER_UPDATE endpoint={:?} my_voice_session={}",
                                            vs.endpoint, my_voice_session.is_some());
                                        last_voice_server = Some((vs.token.clone(), vs.endpoint.clone(), vs.guild_id.clone()));
                                    if let (Some(ref tx_v), Some((token, endpoint, gid)), Some((session_id, channel_id))) =
                                        (Some(&tx_voice_message), last_voice_server.clone(), my_voice_session.clone())
                                    {
                                        if !channel_id.is_empty() {
                                            if let Some(ep) = endpoint {
                                                if let Some(ref my_id) = current_user_id {
                                                    let info = VoiceConnectionInfo {
                                                        guild_id: gid.clone(),
                                                        channel_id: channel_id.clone(),
                                                        endpoint: ep,
                                                        token: token.clone(),
                                                        session_id: session_id.clone(),
                                                        user_id: my_id.clone(),
                                                    };
                                                    let (in_dev, out_dev) = current_voice.as_ref()
                                                        .map(|(_, _, _, _, i, o)| (i.clone(), o.clone()))
                                                        .unwrap_or((None, None));
                                                    eprintln!("[voice gateway] sending Connect (from VOICE_SERVER_UPDATE) channel={}", info.channel_id);
                                                    let _ = tx_v.send(VoiceMessage::Connect {
                                                        info,
                                                        input_device: in_dev,
                                                        output_device: out_dev,
                                                    });
                                                    last_voice_server = None;
                                                    my_voice_session = None;
                                                }
                                            }
                                        }
                                    }
                                    }
                                    Err(e) => {
                                        eprintln!("[voice gateway] VOICE_SERVER_UPDATE parse error: {}", e);
                                    }
                                }
                            }
                        }
                    }
                    9 => break,
                    _ => {}
                }
            }
            // Heartbeat
            _ = heartbeat.tick() => {
                if identified {
                    let payload = serde_json::json!({"op": 1, "d": last_seq});
                    let _ = write.send(WsMessage::Text(payload.to_string())).await;
                }
            }
            // Presence updates from the UI
            Some(new_status) = presence_rx.recv() => {
                current_presence = new_status;
                if identified {
                    let payload = serde_json::json!({
                        "op": 3,
                        "d": presence_to_payload(current_presence),
                    });
                    let _ = write.send(WsMessage::Text(payload.to_string())).await;
                }
            }
            // Voice: join or leave channel
            Some(cmd) = rx_voice_cmd.recv() => {
                if !identified {
                    continue;
                }
                match cmd {
                    VoiceCommand::Join { guild_id, channel_id, self_mute, self_deaf, input_device, output_device } => {
                        eprintln!("[voice gateway] Join received guild_id={:?} channel_id={}", guild_id, channel_id);
                        current_voice = Some((guild_id.clone(), channel_id.clone(), self_mute, self_deaf, input_device.clone(), output_device.clone()));
                        my_voice_session = None;
                        last_voice_server = None;
                        let d = serde_json::json!({
                            "guild_id": guild_id,
                            "channel_id": channel_id,
                            "self_mute": self_mute,
                            "self_deaf": self_deaf,
                        });
                        let payload = serde_json::json!({"op": 4, "d": d});
                        let _ = write.send(WsMessage::Text(payload.to_string())).await;
                    }
                    VoiceCommand::Leave => {
                        let guild_id = current_voice.as_ref().and_then(|(g, _, _, _, _, _)| g.clone());
                        current_voice = None;
                        my_voice_session = None;
                        last_voice_server = None;
                        let _ = tx_voice_message.send(VoiceMessage::Leave);
                        let d = serde_json::json!({
                            "guild_id": guild_id,
                            "channel_id": serde_json::Value::Null,
                            "self_mute": false,
                            "self_deaf": false,
                        });
                        let payload = serde_json::json!({"op": 4, "d": d});
                        let _ = write.send(WsMessage::Text(payload.to_string())).await;
                    }
                }
            }
        }
    }

    Ok(())
}
