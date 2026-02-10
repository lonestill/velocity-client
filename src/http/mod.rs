//! HTTP client (reqwest): REST API.
//! - GET /users/@me — verify token, get current user
//! - GET /users/@me/relationships — friends (undocumented, user token)
//! - GET /users/@me/channels — DM channels (user token)
//! - GET /channels/{id}/messages — channel messages (with optional before)
//! - POST /channels/{id}/messages — send message

use serde::Deserialize;

const API_BASE: &str = "https://discord.com/api/v10";

fn api_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent("Velocity (https://github.com/velocity)")
        .build()
        .map_err(|e| e.to_string())
}

/// Current user from Discord API (GET /users/@me).
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    #[serde(default)]
    pub global_name: Option<String>,
    #[serde(default)]
    pub discriminator: Option<String>,
    #[serde(default)]
    pub avatar: Option<String>,
}

/// Relationship (friend, blocked, etc.) — GET /users/@me/relationships.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Relationship {
    #[serde(default)]
    pub r#type: i32, // 1 = friend, 2 = blocked, 3 = incoming, 4 = outgoing
    pub user: DiscordUser,
}

/// DM or Group DM channel — GET /users/@me/channels.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DmChannel {
    pub id: String,
    #[serde(default)]
    pub r#type: i32, // 1 = DM, 3 = Group DM
    #[serde(default)]
    pub recipients: Vec<DiscordUser>,
    #[serde(default)]
    pub last_message_id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
}

/// Message from Discord API (GET /channels/{id}/messages).
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ApiMessage {
    pub id: String,
    pub channel_id: String,
    pub content: String,
    #[serde(default)]
    pub author: Option<DiscordUser>,
    /// ISO8601 timestamp when the message was sent.
    #[serde(default)]
    pub timestamp: Option<String>,
}

/// Verify token by fetching current user. Returns user on success, error message on 401/invalid.
pub async fn verify_token(token: &str) -> Result<DiscordUser, String> {
    let client = api_client()?;
    let resp = client
        .get(format!("{API_BASE}/users/@me"))
        .header("Authorization", token.trim())
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.status().as_u16() == 401 {
        return Err("Invalid token".to_string());
    }
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Discord API error {}: {}", status, body));
    }

    let user: DiscordUser = resp.json().await.map_err(|e| e.to_string())?;
    Ok(user)
}

/// Get friends list (type=1). Undocumented; returns empty on 404/403.
pub async fn get_relationships(token: &str) -> Result<Vec<Relationship>, String> {
    let client = api_client()?;
    let resp = client
        .get(format!("{API_BASE}/users/@me/relationships"))
        .header("Authorization", token.trim())
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Ok(Vec::new());
    }
    let list: Vec<Relationship> = resp.json().await.map_err(|e| e.to_string())?;
    Ok(list)
}

/// Get DM channels. May return empty if endpoint not available for user token.
pub async fn get_dm_channels(token: &str) -> Result<Vec<DmChannel>, String> {
    let client = api_client()?;
    let resp = client
        .get(format!("{API_BASE}/users/@me/channels"))
        .header("Authorization", token.trim())
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Ok(Vec::new());
    }
    let list: Vec<DmChannel> = resp.json().await.map_err(|e| e.to_string())?;
    Ok(list)
}

/// Create or get DM channel with a user. POST /users/@me/channels.
pub async fn create_dm(token: &str, recipient_id: &str) -> Result<DmChannel, String> {
    let client = api_client()?;
    let body = serde_json::json!({ "recipient_id": recipient_id });
    let resp = client
        .post(format!("{API_BASE}/users/@me/channels"))
        .header("Authorization", token.trim())
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("API error {}: {}", status, body));
    }
    let ch: DmChannel = resp.json().await.map_err(|e| e.to_string())?;
    Ok(ch)
}

/// Get messages for a channel (DM or guild). Limit 1–100. Returns newest first.
pub async fn fetch_channel_messages(
    token: &str,
    channel_id: &str,
    limit: u32,
) -> Result<Vec<ApiMessage>, String> {
    let client = api_client()?;
    let resp = client
        .get(format!("{API_BASE}/channels/{channel_id}/messages"))
        .query(&[("limit", limit)])
        .header("Authorization", token.trim())
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("API error {}: {}", status, body));
    }
    let list: Vec<ApiMessage> = resp.json().await.map_err(|e| e.to_string())?;
    Ok(list)
}

/// Get messages before a given message ID (for loading older messages). Returns newest first in batch.
pub async fn fetch_channel_messages_before(
    token: &str,
    channel_id: &str,
    before_message_id: &str,
    limit: u32,
) -> Result<Vec<ApiMessage>, String> {
    let client = api_client()?;
    let resp = client
        .get(format!("{API_BASE}/channels/{channel_id}/messages"))
        .query(&[
            ("before", before_message_id),
            ("limit", &limit.to_string()),
        ])
        .header("Authorization", token.trim())
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("API error {}: {}", status, body));
    }
    let list: Vec<ApiMessage> = resp.json().await.map_err(|e| e.to_string())?;
    Ok(list)
}

/// Trigger typing indicator in a channel. Rate limit: ~5 sec per channel.
pub async fn trigger_typing(token: &str, channel_id: &str) -> Result<(), String> {
    let client = api_client()?;
    let resp = client
        .post(format!("{API_BASE}/channels/{channel_id}/typing"))
        .header("Authorization", token.trim())
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("Typing API error: {}", resp.status()));
    }
    Ok(())
}

/// Send a message to a channel. Returns the created message.
pub async fn send_message(
    token: &str,
    channel_id: &str,
    content: &str,
) -> Result<ApiMessage, String> {
    let client = api_client()?;
    let body = serde_json::json!({ "content": content });
    let resp = client
        .post(format!("{API_BASE}/channels/{channel_id}/messages"))
        .header("Authorization", token.trim())
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("API error {}: {}", status, body));
    }
    let msg: ApiMessage = resp.json().await.map_err(|e| e.to_string())?;
    Ok(msg)
}
