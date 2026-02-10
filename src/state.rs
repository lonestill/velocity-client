//! In-memory state: guilds, channels, messages cache.
//! Serde for (de)serialization of Discord JSON.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Guild {
    pub id: String,
    pub name: String,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub guild_id: Option<String>,
}

/// A message that may be pending (optimistic send).
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Message {
    pub id: String,
    pub channel_id: String,
    pub author_id: String,
    pub author_username: Option<String>,
    pub content: String,
    pub timestamp: Option<String>,
    /// When true, shows "Sending…" instead of content.
    #[allow(dead_code)]
    pub sending: bool,
}

#[derive(Default, Clone, Debug)]
pub struct State {
    pub token: Option<String>,
    pub guilds: Vec<Guild>,
    pub channels: HashMap<String, Vec<Channel>>,
    pub messages: HashMap<String, Vec<Message>>,
}

/// Save token to OS keyring and update in-memory state.
pub fn login(token: String) -> Result<(), String> {
    let entry = keyring::Entry::new("velocity", "discord_token").map_err(|e| e.to_string())?;
    entry.set_password(&token).map_err(|e| e.to_string())?;
    Ok(())
}

/// Read token from OS keyring (if any).
pub fn load_token() -> Option<String> {
    let entry = keyring::Entry::new("velocity", "discord_token").ok()?;
    entry.get_password().ok()
}

/// Remove token from OS keyring (logout).
pub fn logout() -> Result<(), String> {
    let entry = keyring::Entry::new("velocity", "discord_token").map_err(|e| e.to_string())?;
    entry.delete_password().map_err(|e| e.to_string())?;
    Ok(())
}

/// Preferred presence status for the user.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PresenceStatus {
    #[serde(rename = "online")]
    Online,
    #[serde(rename = "idle")]
    Idle,
    #[serde(rename = "dnd")]
    DoNotDisturb,
    #[serde(rename = "invisible")]
    Invisible,
}

/// Application settings persisted to disk.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppSettings {
    /// Show metrics overlay (FPS, etc).
    #[serde(default)]
    pub show_metrics_overlay: bool,
    /// User has dismissed the welcome screen (first-run).
    #[serde(default)]
    pub welcome_seen: bool,
    /// Enable UI animations (transitions, fade-in, etc).
    #[serde(default = "default_true")]
    pub animations_enabled: bool,
    /// Preferred presence (online / idle / dnd / invisible).
    #[serde(default = "default_presence")]
    pub presence: PresenceStatus,
    /// When true, do not send typing events ("ghost typing").
    #[serde(default)]
    pub ghost_typing: bool,
}

fn default_true() -> bool {
    true
}

fn default_presence() -> PresenceStatus {
    PresenceStatus::Online
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            show_metrics_overlay: false,
            welcome_seen: false,
            animations_enabled: true,
            presence: PresenceStatus::Online,
            ghost_typing: false,
        }
    }
}

fn settings_path() -> Option<PathBuf> {
    let config_dir = dirs::config_dir()?;
    let dir = config_dir.join("velocity");
    let _ = fs::create_dir_all(&dir).ok();
    Some(dir.join("settings.json"))
}

/// Load app settings from disk.
pub fn load_settings() -> AppSettings {
    let path = match settings_path() {
        Some(p) => p,
        None => return AppSettings::default(),
    };
    match fs::read_to_string(&path) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => AppSettings::default(),
    }
}

/// Save app settings to disk.
pub fn save_settings(settings: &AppSettings) -> Result<(), String> {
    let path = settings_path().ok_or("Could not determine config path")?;
    let s = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(&path, s).map_err(|e| e.to_string())?;
    Ok(())
}
