use dioxus::prelude::*;

use crate::http::GuildChannel;

/// Text channel type in Discord API.
const CHANNEL_TYPE_TEXT: i32 = 0;
/// Voice channel type.
const CHANNEL_TYPE_VOICE: i32 = 2;
/// Category channel type.
const CHANNEL_TYPE_CATEGORY: i32 = 4;
/// Private thread (only for users with permission).
const CHANNEL_TYPE_PRIVATE_THREAD: i32 = 12;

#[component]
fn ChannelButton(
    channel_id: String,
    channel_name: String,
    color: &'static str,
    show_lock: bool,
    on_select_channel: EventHandler<Option<String>>,
) -> Element {
    let prefix = if show_lock { "🔒" } else { "#" };
    let prefix_style = if show_lock {
        "opacity: 0.9; font-size: 0.75rem;"
    } else {
        "opacity: 0.8;"
    };
    rsx! {
        button {
            class: "anim-btn",
            style: "
                display: flex; align-items: center; gap: 0.5rem;
                width: 100%; padding: 0.4rem 0.75rem;
                text-align: left; border: none; background: transparent;
                color: {color};
                font-size: 0.9rem; cursor: pointer;
                border-radius: 0.25rem; margin: 0 0.25rem;
            ",
            onclick: move |_| on_select_channel.call(Some(channel_id.clone())),
            span { style: "{prefix_style}", "{prefix}" }
            "{channel_name}"
        }
    }
}

#[component]
pub fn GuildChannelList(
    guild_channels: Signal<Vec<GuildChannel>>,
    selected_channel_id: Signal<Option<String>>,
    show_private_channels: bool,
    access_denied_channel_ids: Signal<std::collections::HashSet<String>>,
    current_voice_channel_id: Signal<Option<String>>,
    current_voice_guild_id: Signal<Option<String>>,
    selected_guild_id: Signal<Option<String>>,
    on_select_channel: EventHandler<Option<String>>,
    on_join_voice: EventHandler<(Option<String>, String)>,
    on_leave_voice: EventHandler<()>,
) -> Element {
    let channels = guild_channels();
    let selected = selected_channel_id();
    let access_denied = access_denied_channel_ids();
    let in_voice = current_voice_channel_id();
    let voice_guild = current_voice_guild_id();
    let guild_id = selected_guild_id();

    let text_channels: Vec<&GuildChannel> = channels
        .iter()
        .filter(|c| c.r#type == CHANNEL_TYPE_TEXT || c.r#type == CHANNEL_TYPE_PRIVATE_THREAD)
        .collect();
    let voice_channels: Vec<&GuildChannel> = channels
        .iter()
        .filter(|c| c.r#type == CHANNEL_TYPE_VOICE)
        .collect();
    let categories: Vec<&GuildChannel> = channels
        .iter()
        .filter(|c| c.r#type == CHANNEL_TYPE_CATEGORY)
        .collect();

    type ChannelRow = (String, String, bool, &'static str, bool);
    type VoiceRow = (String, String, bool, Option<String>); // (id, name, is_connected, guild_id for join)
    let selected_ref = selected.as_ref().map(|s| s.as_str());

    let is_private = |c: &GuildChannel| -> bool {
        c.r#type == CHANNEL_TYPE_PRIVATE_THREAD || access_denied.contains(&c.id)
    };

    let mut cat_names: Vec<Option<String>> = Vec::new();
    let mut cat_rows: Vec<Vec<ChannelRow>> = Vec::new();
    let mut voice_entries: Vec<VoiceRow> = Vec::new();
    for c in voice_channels.iter().filter(|c| c.parent_id.is_none()) {
        let connected = in_voice.as_ref() == Some(&c.id) && voice_guild.as_ref() == guild_id.as_ref();
        voice_entries.push((c.id.clone(), c.name.clone(), connected, guild_id.clone()));
    }
    voice_entries.sort_by(|a, b| a.1.cmp(&b.1));
    for cat in categories.iter() {
        for c in voice_channels.iter().filter(|c| c.parent_id.as_deref() == Some(cat.id.as_str())) {
            let connected = in_voice.as_ref() == Some(&c.id) && voice_guild.as_ref() == guild_id.as_ref();
            voice_entries.push((c.id.clone(), c.name.clone(), connected, guild_id.clone()));
        }
    }
    voice_entries.sort_by(|a, b| a.1.cmp(&b.1));

    let mut uncategorized: Vec<ChannelRow> = text_channels
        .iter()
        .filter(|c| c.parent_id.is_none())
        .filter(|c| show_private_channels || !is_private(c))
        .map(|c| {
            let is_sel = selected_ref == Some(c.id.as_str());
            let priv_ = is_private(c);
            (
                c.id.clone(),
                c.name.clone(),
                is_sel,
                if is_sel { "#00fff5" } else { "#9ca3af" },
                priv_,
            )
        })
        .collect();
    uncategorized.sort_by(|a, b| a.1.cmp(&b.1));
    if !uncategorized.is_empty() {
        cat_names.push(None);
        cat_rows.push(uncategorized);
    }
    for cat in categories.iter() {
        let mut chs: Vec<ChannelRow> = text_channels
            .iter()
            .filter(|c| c.parent_id.as_deref() == Some(cat.id.as_str()))
            .filter(|c| show_private_channels || !is_private(c))
            .map(|c| {
                let is_sel = selected_ref == Some(c.id.as_str());
                let priv_ = is_private(c);
                (
                    c.id.clone(),
                    c.name.clone(),
                    is_sel,
                    if is_sel { "#00fff5" } else { "#9ca3af" },
                    priv_,
                )
            })
            .collect();
        chs.sort_by(|a, b| a.1.cmp(&b.1));
        if !chs.is_empty() {
            cat_names.push(Some(cat.name.clone()));
            cat_rows.push(chs);
        }
    }

    rsx! {
        div {
            class: "custom-scroll",
            style: "
                display: flex; flex-direction: column; flex: 1;
                min-height: 0; overflow-y: auto; padding: 0.5rem 0;
            ",
            div {
                style: "
                    padding: 0 0.75rem 0.5rem;
                    font-size: 0.7rem; font-weight: 600;
                    color: #9ca3af; text-transform: uppercase;
                    letter-spacing: 0.05em;
                ",
                "Channels"
            }
            for (cat_name, rows) in cat_names.iter().zip(cat_rows.iter()) {
                if let Some(ref name) = cat_name {
                    div {
                        style: "
                            padding: 0.25rem 0.75rem 0.15rem;
                            font-size: 0.7rem; font-weight: 600;
                            color: #6b7280;
                        ",
                        "{name}"
                    }
                }
                for (ch_s, ch_n, _is_sel, ch_color, ch_private) in rows.iter() {
                    ChannelButton {
                        channel_id: ch_s.clone(),
                        channel_name: ch_n.clone(),
                        color: ch_color,
                        show_lock: *ch_private,
                        on_select_channel,
                    }
                }
            }
            if cat_rows.is_empty() {
                div {
                    style: "
                        padding: 1rem 0.75rem;
                        font-size: 0.85rem; color: #6b7280;
                    ",
                    "No text channels"
                }
            }
            div {
                style: "
                    padding: 0 0.75rem 0.5rem;
                    font-size: 0.7rem; font-weight: 600;
                    color: #9ca3af; text-transform: uppercase;
                    letter-spacing: 0.05em; margin-top: 0.5rem;
                ",
                "Voice"
            }
            if voice_entries.is_empty() {
                div {
                    style: "
                        padding: 0.4rem 0.75rem; font-size: 0.85rem;
                        color: #6b7280; margin: 0 0.25rem;
                    ",
                    "No voice channels"
                }
            } else {
                for (ch_id, ch_name, connected, gid) in voice_entries.into_iter() {
                    button {
                        class: "anim-btn",
                        style: "
                            display: flex; align-items: center; justify-content: space-between; gap: 0.5rem;
                            width: 100%; padding: 0.4rem 0.75rem;
                            text-align: left; border: none; background: transparent;
                            color: #9ca3af; font-size: 0.9rem; cursor: pointer;
                            border-radius: 0.25rem; margin: 0 0.25rem;
                        ",
                        span { style: "opacity: 0.8;", "🔊 {ch_name}" }
                        if connected {
                            span {
                                style: "font-size: 0.75rem; color: #22c55e; cursor: pointer;",
                                onclick: move |_| on_leave_voice.call(()),
                                "Leave"
                            }
                        } else {
                            span {
                                style: "font-size: 0.75rem; color: #00fff5; cursor: pointer;",
                                onclick: move |_| on_join_voice.call((gid.clone(), ch_id.clone())),
                                "Join"
                            }
                        }
                    }
                }
            }
        }
    }
}
