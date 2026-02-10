use chrono::{DateTime, Datelike, Utc};
use dioxus::prelude::*;

use crate::http::{DiscordUser, DmChannel, GuildChannel};
use crate::state::Message;
use crate::ui::{MessageContextMenu, UserCard};

fn format_message_time(ts: Option<&str>) -> String {
    let Some(s) = ts else {
        return String::new();
    };
    let Ok(dt) = DateTime::parse_from_rfc3339(s) else {
        return String::new();
    };
    let dt_utc: DateTime<Utc> = dt.with_timezone(&Utc);
    let now = Utc::now();
    let same_day = dt_utc.date_naive() == now.date_naive();
    if same_day {
        dt_utc.format("%H:%M").to_string()
    } else if dt_utc > now - chrono::Duration::days(7) {
        dt_utc.format("%a %H:%M").to_string()
    } else if dt_utc.year() == now.year() {
        dt_utc.format("%d %b %H:%M").to_string()
    } else {
        dt_utc.format("%d %b %Y %H:%M").to_string()
    }
}

fn dm_channel_title(ch: &DmChannel) -> String {
    if ch.recipients.is_empty() {
        ch.name.as_deref().unwrap_or("DM").to_string()
    } else {
        ch.recipients
            .iter()
            .map(|u| u.global_name.as_deref().unwrap_or(u.username.as_str()).to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[component]
fn MessageRow(
    msg: Message,
    is_mine: bool,
    author_user: Option<DiscordUser>,
    on_avatar_click: Option<EventHandler<(DiscordUser, f64, f64, bool)>>,
    on_context_menu: Option<EventHandler<(f64, f64, String)>>,
) -> Element {
    let author = msg
        .author_username
        .as_deref()
        .unwrap_or(msg.author_id.as_str());
    let (bubble_style, bubble_margin) = if is_mine {
        (
            "max-width: 75%; margin-left: auto; margin-right: 0; padding: 0.5rem 0.75rem; \
         border-radius: 12px 12px 4px 12px; background: rgba(0,255,245,0.15); \
         color: #e5e7eb; font-size: 0.9375rem; line-height: 1.4;",
            "margin-right: 0.5rem;",
        )
    } else {
        (
            "max-width: 75%; margin-left: 0; margin-right: auto; padding: 0.5rem 0.75rem; \
         border-radius: 12px 12px 12px 4px; background: rgba(255,255,255,0.08); \
         color: #e5e7eb; font-size: 0.9375rem; line-height: 1.4;",
            "margin-left: 0.5rem;",
        )
    };
    let time_str = format_message_time(msg.timestamp.as_deref());
    let row_style = if is_mine {
        "display: flex; flex-direction: row-reverse; padding: 0.25rem 1rem; margin-bottom: 0.25rem;"
    } else {
        "display: flex; padding: 0.25rem 1rem; margin-bottom: 0.25rem;"
    };
    let content_html = if msg.sending {
        "Sending…".to_string()
    } else {
        crate::ui::markdown::discord_markdown_to_html(&msg.content)
    };
    let avatar_el = if let Some(ref u) = author_user {
        let url = u.avatar.as_ref().map(|hash| {
            let ext = if hash.starts_with("a_") { "gif" } else { "png" };
            format!(
                "https://cdn.discordapp.com/avatars/{}/{}.{}",
                u.id, hash, ext
            )
        });
        let handler = on_avatar_click.clone();
        let user = u.clone();
        match url {
            Some(url) => rsx! {
                img {
                    src: "{url}",
                    alt: "",
                    style: "width: 2rem; height: 2rem; border-radius: 50%; object-fit: cover; cursor: pointer; flex-shrink: 0;",
                    onclick: move |evt| {
                        if let Some(ref h) = handler {
                            let coords = evt.client_coordinates();
                            h.call((user.clone(), coords.x, coords.y, is_mine));
                        }
                    },
                }
            },
            None => rsx! {
                div {
                    style: "
                        width: 2rem; height: 2rem; border-radius: 50%;
                        background: rgba(0,255,245,0.2);
                        display: flex; align-items: center; justify-content: center;
                        font-size: 0.65rem; font-weight: 600; color: #00fff5;
                        cursor: pointer; flex-shrink: 0;
                    ",
                    onclick: move |evt| {
                        if let Some(ref h) = handler {
                            let coords = evt.client_coordinates();
                            h.call((user.clone(), coords.x, coords.y, is_mine));
                        }
                    },
                    "{author.chars().next().unwrap_or('?')}"
                }
            },
        }
    } else {
        rsx! { div { style: "width: 2rem; height: 2rem; flex-shrink: 0;" } }
    };
    rsx! {
        div {
            class: "anim-message-row",
            style: "{row_style}",
            oncontextmenu: move |evt| {
                evt.prevent_default();
                if let Some(ref h) = on_context_menu {
                    let coords = evt.client_coordinates();
                    h.call((coords.x, coords.y, msg.content.clone()));
                }
            },
            {avatar_el}
            div {
                style: "{bubble_style} {bubble_margin}",
                div {
                    style: "display: flex; align-items: baseline; gap: 0.5rem; margin-bottom: 0.2rem;",
                    span {
                        style: "color: #00fff5; font-size: 0.75rem;",
                        "{author}"
                    }
                    if !time_str.is_empty() {
                        span {
                            style: "color: #6b7280; font-size: 0.65rem;",
                            "{time_str}"
                        }
                    }
                }
                div {
                    style: "word-break: break-word; white-space: pre-wrap;",
                    dangerous_inner_html: "{content_html}"
                }
            }
        }
    }
}

fn resolve_author(
    msg: &Message,
    current_user: &Option<DiscordUser>,
    channels: &[DmChannel],
    selected_id: &Option<String>,
) -> Option<DiscordUser> {
    if current_user.as_ref().map(|u| u.id.as_str()) == Some(msg.author_id.as_str()) {
        return current_user.clone();
    }
    let ch = selected_id
        .as_ref()
        .and_then(|id| channels.iter().find(|c| c.id == *id))?;
    ch.recipients
        .iter()
        .find(|u| u.id == msg.author_id)
        .cloned()
}

#[component]
pub fn MessageList(
    messages: Signal<Vec<Message>>,
    selected_channel_id: Signal<Option<String>>,
    dm_channels: Signal<Vec<DmChannel>>,
    guild_channels: Signal<Vec<GuildChannel>>,
    current_user: Signal<Option<DiscordUser>>,
    current_voice_channel_id: Signal<Option<String>>,
    current_voice_guild_id: Signal<Option<String>>,
    has_more_older: Signal<bool>,
    loading_older: Signal<bool>,
    loading_messages: Signal<bool>,
    typing_users: Signal<std::collections::HashMap<String, std::collections::HashMap<String, i64>>>,
    access_denied_channel_ids: Signal<std::collections::HashSet<String>>,
    channel_error_display: Signal<Option<(String, String)>>,
    on_join_voice: EventHandler<(Option<String>, String)>,
    on_leave_voice: EventHandler<()>,
    on_send_message: EventHandler<(String, String)>,
    on_load_older: EventHandler<(String, String)>,
    on_trigger_typing: EventHandler<String>,
) -> Element {
    let mut user_card = use_signal(|| None::<(DiscordUser, f64, f64, bool)>);
    let mut context_menu = use_signal(|| None::<(f64, f64, String)>);
    let mut last_typing_trigger = use_signal(|| 0i64);
    const GUILD_PRIVATE_THREAD: i32 = 12;

    let list = messages();
    let selected = selected_channel_id();
    let channels = dm_channels();
    let guild_chs = guild_channels();
    let access_denied = access_denied_channel_ids();
    let channel_error = channel_error_display();
    let current_user_id: Option<String> = current_user().as_ref().map(|u| u.id.clone());

    let is_private_channel = selected.as_ref().and_then(|sid| {
        guild_chs.iter().find(|c| c.id == *sid).map(|c| {
            c.r#type == GUILD_PRIVATE_THREAD || access_denied.contains(sid)
        })
    }).unwrap_or(false);

    let private_debug: Option<(String, String, i32, String)> = (is_private_channel && selected.is_some()).then(|| {
        let sid = selected.as_ref().unwrap();
        let ch = guild_chs.iter().find(|c| c.id == *sid);
        let (name, ch_type) = ch
            .map(|c| (c.name.clone(), c.r#type))
            .unwrap_or_else(|| ("?".to_string(), 0));
        let err = channel_error
            .as_ref()
            .filter(|(id, _)| id == sid)
            .map(|(_, e)| e.clone())
            .unwrap_or_else(|| "No error message".to_string());
        (sid.clone(), name, ch_type, err)
    });

    let header_icon = if is_private_channel { "🔒" } else { "💬" };
    let header_title = if let Some(sel_id) = selected.as_ref() {
        if let Some(dm) = channels.iter().find(|c| c.id == *sel_id) {
            dm_channel_title(dm)
        } else if let Some(gc) = guild_chs.iter().find(|c| c.id == *sel_id) {
            format!("#{}", gc.name)
        } else {
            "Select a chat".to_string()
        }
    } else {
        "Select a chat".to_string()
    };

    let is_dm_selected = selected
        .as_ref()
        .and_then(|sel_id| channels.iter().find(|c| c.id == *sel_id))
        .is_some();
    let dm_call_connected = is_dm_selected
        && selected.as_ref() == current_voice_channel_id().as_ref()
        && current_voice_guild_id().is_none();

    let mut draft = use_signal(|| String::new());
    let can_send = selected.is_some() && !draft().trim().is_empty();

    let load_older_visible = has_more_older() && !list.is_empty() && selected.is_some();
    let loading = loading_older();
    let loading_msgs = loading_messages();
    let typing = selected
        .as_ref()
        .and_then(|cid| typing_users().get(cid).cloned())
        .unwrap_or_default();
    let now = chrono::Utc::now().timestamp();
    let typing_names: Vec<String> = typing
        .iter()
        .filter(|(_, &expiry)| expiry > now)
        .filter(|(uid, _)| Some(uid.as_str()) != current_user_id.as_deref())
        .filter_map(|(uid, _)| {
            let ch = selected
                .as_ref()
                .and_then(|cid| channels.iter().find(|c| c.id == *cid))?;
            let u = ch.recipients.iter().find(|u| u.id == *uid)?;
            Some(
                u.global_name
                    .as_deref()
                    .unwrap_or(u.username.as_str())
                    .to_string(),
            )
        })
        .collect();
    let typing_text = if typing_names.is_empty() {
        String::new()
    } else {
        format!("{} typing…", typing_names.join(", "))
    };

    // Auto-scroll to bottom when opening a chat or when messages load
    use_effect(move || {
        let _ = selected_channel_id();
        let _ = messages();
        spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            let _ = document::eval(
                r#"
                const el = document.getElementById('message-list-scroll');
                if (el) { el.scrollTop = el.scrollHeight; }
                "#,
            );
        });
    });

    let messages_content = if let Some((ref ch_id, ref ch_name, ch_type, ref err)) = private_debug {
        rsx! {
            div {
                style: "
                    flex: 1; display: flex; flex-direction: column;
                    align-items: center; justify-content: center;
                    padding: 2rem; text-align: center;
                    background: #0a0a0f; color: #9ca3af;
                ",
                div {
                    style: "font-size: 3rem; margin-bottom: 1rem; opacity: 0.7;",
                    "🔒"
                }
                h2 {
                    style: "font-size: 1.25rem; font-weight: 600; color: #e5e7eb; margin: 0 0 0.5rem 0;",
                    "This channel is private"
                }
                p {
                    style: "font-size: 0.9375rem; margin: 0 0 1rem 0; max-width: 24rem;",
                    "Only users with the right server permissions can see it."
                }
                pre {
                    style: "
                        text-align: left; font-size: 0.75rem;
                        background: rgba(0,0,0,0.3); padding: 1rem;
                        border-radius: 8px; overflow-x: auto;
                        color: #6b7280; margin: 0;
                    ",
                    "channel_id: {ch_id}\nname: {ch_name}\ntype: {ch_type}\nerror: {err}"
                }
            }
        }
    } else if loading_msgs {
        rsx! {
            div {
                style: "
                    flex: 1; display: flex; flex-direction: column;
                    padding: 1rem; gap: 1rem;
                ",
                for _ in 0..5 {
                    div {
                        style: "display: flex; gap: 0.75rem; align-items: flex-start;",
                        div {
                            style: "
                                width: 2.5rem; height: 2.5rem; border-radius: 50%;
                                background: rgba(255,255,255,0.08);
                                flex-shrink: 0;
                            ",
                        }
                        div {
                            style: "flex: 1; display: flex; flex-direction: column; gap: 0.5rem;",
                            div {
                                style: "
                                    height: 1rem; border-radius: 4px;
                                    background: rgba(255,255,255,0.08);
                                    width: 40%;
                                ",
                            }
                            div {
                                style: "
                                    height: 2.5rem; border-radius: 8px;
                                    background: rgba(255,255,255,0.06);
                                    width: 80%;
                                ",
                            }
                        }
                    }
                }
            }
        }
    } else if list.is_empty() && !load_older_visible {
        rsx! {
            div {
                style: "padding: 1rem; color: #6b7280; font-size: 0.875rem;",
                "No messages yet."
            }
        }
    } else {
        rsx! {
            if load_older_visible {
                div {
                    key: "{selected_channel_id().as_deref().unwrap_or(\"none\")}-load-older",
                    style: "display: flex; justify-content: center; padding: 0.5rem;",
                    button {
                        class: "anim-btn",
                        style: "
                            padding: 0.375rem 0.75rem; font-size: 0.8125rem;
                            background: rgba(255,255,255,0.08); border: 1px solid rgba(255,255,255,0.15);
                            border-radius: 6px; color: #9ca3af; cursor: pointer;
                        ",
                        disabled: "{loading}",
                        onclick: move |_| {
                            let cid = selected_channel_id();
                            let list = messages();
                            let before = list.first().map(|m| m.id.clone()).unwrap_or_default();
                            if let (Some(cid), true) = (cid.as_ref(), !before.is_empty()) {
                                on_load_older.call((cid.clone(), before));
                            }
                        },
                        if loading { "Loading…" } else { "Load older messages" }
                    }
                }
            }
            for msg in list.iter() {
                MessageRow {
                    key: "{msg.id}",
                    msg: msg.clone(),
                    is_mine: current_user_id.as_deref() == Some(msg.author_id.as_str()),
                    author_user: resolve_author(msg, &current_user(), &channels, &selected),
                    on_avatar_click: Some(EventHandler::new(move |(u, x, y, is_mine): (DiscordUser, f64, f64, bool)| {
                        user_card.set(Some((u.clone(), x, y, is_mine)));
                    })),
                    on_context_menu: Some(EventHandler::new(move |(x, y, content): (f64, f64, String)| {
                        context_menu.set(Some((x, y, content)));
                    })),
                }
            }
        }
    };

    rsx! {
        if let Some((ref u, x, y, anchor_right)) = user_card() {
            UserCard {
                user: u.clone(),
                x: x as i32,
                y: y as i32,
                anchor_right,
                on_close: move |_| user_card.set(None),
            }
        }
        if let Some((x, y, ref content)) = context_menu() {
            MessageContextMenu {
                x,
                y,
                content: content.clone(),
                on_close: move |_| context_menu.set(None),
            }
        }
        div {
            style: "flex: 1 1 0; display: flex; flex-direction: column; min-width: 0; min-height: 0; overflow: hidden;",
            header {
                style: "flex-shrink: 0; padding: 0.75rem 1rem; border-bottom: 1px solid rgba(255,255,255,0.1); display: flex; align-items: center; gap: 0.5rem;",
                span { style: "color: #00fff5;", "{header_icon}" }
                span { style: "font-weight: 500;", "{header_title}" }
                if is_dm_selected {
                    div { style: "margin-left: auto; display: flex; align-items: center; gap: 0.5rem;",
                        button {
                            class: "anim-btn",
                            style: "
                                padding: 0.35rem 0.65rem;
                                border-radius: 8px;
                                border: 1px solid rgba(255,255,255,0.12);
                                background: rgba(255,255,255,0.06);
                                color: #e5e7eb;
                                font-size: 0.85rem;
                                cursor: pointer;
                            ",
                            onclick: move |_| {
                                if let Some(sel_id) = selected.as_ref() {
                                    if dm_call_connected {
                                        on_leave_voice.call(());
                                    } else {
                                        on_join_voice.call((None, sel_id.clone()));
                                    }
                                }
                            },
                            if dm_call_connected { "Leave call" } else { "Call" }
                        }
                    }
                }
            }
            div {
                id: "message-list-scroll",
                class: "message-list-scroll custom-scroll",
                style: "flex: 1 1 0; min-height: 0; overflow-y: auto; overflow-x: hidden; display: flex; flex-direction: column; align-items: stretch;",
                {messages_content}
            }
            if selected_channel_id().is_some() && private_debug.is_none() {
                div {
                    style: "
                        flex-shrink: 0;
                        border-top: 1px solid rgba(255,255,255,0.1);
                        display: flex; flex-direction: column; gap: 0;
                    ",
                    {if !typing_text.is_empty() {
                        rsx! {
                            div {
                                style: "padding: 0.25rem 1rem; font-size: 0.75rem; color: #6b7280; font-style: italic;",
                                "{typing_text}"
                            }
                        }
                    } else {
                        rsx! { }
                    }}
                    div {
                        style: "padding: 0.75rem 1rem; display: flex; gap: 0.5rem; align-items: flex-end;",
                    input {
                        style: "
                            flex: 1; padding: 0.5rem 0.75rem; font-size: 0.9375rem;
                            background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.12);
                            border-radius: 8px; color: #e5e7eb; outline: none;
                        ",
                        placeholder: "Message",
                        value: "{draft()}",
                        oninput: move |evt| {
                            draft.set(evt.value());
                            if let Some(cid) = selected_channel_id().as_ref() {
                                let now = chrono::Utc::now().timestamp();
                                if now - last_typing_trigger() >= 5 {
                                    last_typing_trigger.set(now);
                                    on_trigger_typing.call(cid.clone());
                                }
                            }
                        },
                        onkeydown: move |evt| {
                            if evt.key() == Key::Enter {
                                evt.prevent_default();
                                if let Some(cid) = selected_channel_id().as_ref() {
                                    let text = draft();
                                    if !text.trim().is_empty() {
                                        on_send_message.call((cid.clone(), text));
                                        draft.set(String::new());
                                    }
                                }
                            }
                        },
                    }
                    button {
                        class: "anim-btn",
                        style: "
                            padding: 0.5rem 1rem; font-size: 0.9375rem; font-weight: 500;
                            background: rgba(0,255,245,0.2); border: 1px solid rgba(0,255,245,0.4);
                            border-radius: 8px; color: #00fff5; cursor: pointer;
                        ",
                        disabled: "{!can_send}",
                        onclick: move |_| {
                            if let Some(cid) = selected_channel_id().as_ref() {
                                let text = draft();
                                if !text.trim().is_empty() {
                                    on_send_message.call((cid.clone(), text));
                                    draft.set(String::new());
                                }
                            }
                        },
                        "Send"
                    }
                    }
                }
            }
        }
    }
}
