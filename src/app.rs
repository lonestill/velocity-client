use dioxus::prelude::*;
use std::collections::HashMap;
use tokio::sync::mpsc;

use crate::gateway;
use crate::http::{self, DiscordUser, DmChannel, Relationship};
use crate::state::{load_settings, load_token, login, logout, Guild, Message, PresenceStatus};
use crate::ui::{Layout, LoginForm, MetricsOverlay, SettingsModal, ToastContainer, WelcomeModal};

#[component]
pub fn App() -> Element {
    let guilds = use_signal(|| Vec::<Guild>::new());
    let mut token = use_signal(|| load_token());
    let mut current_user = use_signal(|| None::<DiscordUser>);
    let mut friends = use_signal(|| Vec::<Relationship>::new());
    let mut dm_channels = use_signal(|| Vec::<DmChannel>::new());
    let mut selected_channel_id = use_signal(|| None::<String>);
    let mut messages = use_signal(|| Vec::<Message>::new());
    let mut has_more_older = use_signal(|| false);
    let mut loading_older = use_signal(|| false);
    let mut loading_messages = use_signal(|| false);
    let token_input = use_signal(|| String::new());
    let mut login_error = use_signal(|| None::<String>);
    let mut login_loading = use_signal(|| false);
    let settings = use_signal(|| load_settings());
    let mut settings_open = use_signal(|| false);
    let mut toast_messages = use_signal(|| Vec::<(usize, String)>::new());
    let mut toast_counter = use_signal(|| 0usize);
    let mut unread_counts = use_signal(|| HashMap::<String, u32>::new());
    let typing_users = use_signal(|| HashMap::<String, std::collections::HashMap<String, i64>>::new());
    // Channel to push presence updates (status) to the Gateway task.
    let mut presence_tx = use_signal(|| None::<mpsc::UnboundedSender<PresenceStatus>>);

    use_effect(move || {
        let tok = token();
        let user = current_user();
        if tok.is_some() && user.is_none() {
            let t = tok.unwrap();
            spawn(async move {
                if let Ok(u) = http::verify_token(&t).await {
                    current_user.set(Some(u));
                }
            });
        }
    });

    use_effect(move || {
        let tok = token();
        let user = current_user();
        if tok.is_some() && user.is_some() {
            let t = tok.unwrap();
            spawn(async move {
                if let Ok(list) = http::get_relationships(&t).await {
                    friends.set(list);
                }
                if let Ok(list) = http::get_dm_channels(&t).await {
                    dm_channels.set(list);
                }
            });
        }
    });

    // Gateway: spawn when logged in, receive real-time messages and typing.
    // Use signal to avoid re-spawning on every effect run (would create duplicate connections).
    let mut gateway_spawned = use_signal(|| None::<String>);
    use_effect(move || {
        let tok = token();
        if tok.is_none() {
            gateway_spawned.set(None);
            presence_tx.set(None);
            return;
        }
        let t = tok.unwrap();
        if gateway_spawned().as_ref() == Some(&t) {
            return;
        }
        gateway_spawned.set(Some(t.clone()));
        let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
        let (tx_typing, mut rx_typing) = mpsc::unbounded_channel::<(String, String)>();
        let (tx_presence, rx_presence) = mpsc::unbounded_channel::<PresenceStatus>();
        // Store sender so Settings UI can push live status updates.
        presence_tx.set(Some(tx_presence.clone()));
        let initial_presence = settings().presence;
        gateway::spawn_gateway(t.clone(), tx, Some(tx_typing), initial_presence, rx_presence);
        let mut msgs_sig = messages;
        let sel_sig = selected_channel_id;
        let mut unread_sig = unread_counts;
        let mut typing_sig = typing_users;
        spawn(async move {
            while let Some(msg) = rx.recv().await {
                let sel = sel_sig();
                let is_current = sel.as_ref() == Some(&msg.channel_id);
                if is_current {
                    let mut list = msgs_sig();
                    if !list.iter().any(|m| m.id == msg.id) {
                        list.push(msg.clone());
                        msgs_sig.set(list);
                    }
                } else {
                    let mut counts = unread_sig();
                    *counts.entry(msg.channel_id).or_insert(0) += 1;
                    unread_sig.set(counts);
                }
            }
        });
        spawn(async move {
            while let Some((channel_id, user_id)) = rx_typing.recv().await {
                let now = chrono::Utc::now().timestamp();
                let mut map = typing_sig();
                map.entry(channel_id)
                    .or_default()
                    .insert(user_id, now + 10);
                typing_sig.set(map);
            }
        });
    });

    use_effect(move || {
        let tok = token();
        let ch_id = selected_channel_id();
        let mut msgs_signal = messages;
        let mut has_more = has_more_older;
        let mut loading = loading_messages;
        let mut toast = toast_messages;
        let mut counter = toast_counter;
        if let (Some(t), Some(cid)) = (tok, ch_id) {
            messages.set(Vec::new());
            has_more_older.set(false);
            loading_messages.set(true);
            spawn(async move {
                match http::fetch_channel_messages(&t, &cid, 50).await {
                    Ok(api_msgs) => {
                        let msgs: Vec<Message> = api_msgs
                            .into_iter()
                            .map(|m| Message {
                                id: m.id,
                                channel_id: m.channel_id,
                                author_id: m.author.as_ref().map(|a| a.id.clone()).unwrap_or_default(),
                                author_username: m
                                    .author
                                    .as_ref()
                                    .map(|a| a.global_name.clone().or(Some(a.username.clone())).unwrap()),
                                content: m.content,
                                timestamp: m.timestamp,
                                sending: false,
                            })
                            .collect();
                        has_more.set(msgs.len() == 50);
                        msgs_signal.set(msgs.into_iter().rev().collect::<Vec<_>>());
                    }
                    Err(e) => {
                        has_more.set(false);
                        msgs_signal.set(Vec::new());
                        let id = counter() + 1;
                        counter.set(id);
                        let mut t = toast();
                        t.push((id, e));
                        toast.set(t);
                        let mut toast_rm = toast;
                        spawn(async move {
                            tokio::time::sleep(tokio::time::Duration::from_secs(4)).await;
                            let mut t = toast_rm();
                            t.retain(|(i, _)| *i != id);
                            toast_rm.set(t);
                        });
                    }
                }
                loading.set(false);
            });
        } else {
            has_more_older.set(false);
            messages.set(Vec::new());
            loading_messages.set(false);
        }
    });

    let main_content = if token().is_some() {
        rsx! {
            Layout {
                guilds,
                friends,
                dm_channels,
                messages,
                current_user,
                selected_channel_id,
                has_more_older,
                loading_older,
                loading_messages,
                settings,
                unread_counts,
                typing_users,
                on_select_channel: move |id: Option<String>| {
                    if let Some(ref cid) = id {
                        let mut counts = unread_counts();
                        counts.insert(cid.clone(), 0);
                        unread_counts.set(counts);
                    }
                    selected_channel_id.set(id);
                },
                on_send_message: move |arg: (String, String)| {
                    let (channel_id, content) = arg;
                    let trimmed = content.trim().to_string();
                    if trimmed.is_empty() {
                        return;
                    }
                    let tok = match token() {
                        Some(t) => t,
                        None => return,
                    };
                    let current_uid = current_user().as_ref().map(|u| u.id.clone());
                    let temp_id = format!("sending-{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0));
                    let optimistic = Message {
                        id: temp_id.clone(),
                        channel_id: channel_id.clone(),
                        author_id: current_uid.clone().unwrap_or_default(),
                        author_username: current_user().as_ref().map(|u| u.global_name.clone().or(Some(u.username.clone())).unwrap()),
                        content: trimmed.clone(),
                        timestamp: None,
                        sending: true,
                    };
                    let mut msgs = messages;
                    let mut list = msgs();
                    list.push(optimistic);
                    msgs.set(list);
                    let mut toast = toast_messages;
                    let mut counter = toast_counter;
                    spawn(async move {
                        match http::send_message(&tok, &channel_id, &trimmed).await {
                            Ok(api_msg) => {
                                let real = Message {
                                    id: api_msg.id.clone(),
                                    channel_id: api_msg.channel_id,
                                    author_id: api_msg.author.as_ref().map(|a| a.id.clone()).unwrap_or_default(),
                                    author_username: api_msg
                                        .author
                                        .as_ref()
                                        .map(|a| a.global_name.clone().or(Some(a.username.clone())).unwrap()),
                                    content: api_msg.content,
                                    timestamp: api_msg.timestamp,
                                    sending: false,
                                };
                                let mut list = msgs();
                                list.retain(|m| m.id != temp_id);
                                if !list.iter().any(|m| m.id == api_msg.id) {
                                    list.push(real);
                                }
                                msgs.set(list);
                            }
                            Err(e) => {
                                let mut list = msgs();
                                list.retain(|m| m.id != temp_id);
                                msgs.set(list);
                                let id = counter() + 1;
                                counter.set(id);
                                let mut t = toast();
                                t.push((id, e));
                                toast.set(t);
                                let mut toast_rm = toast;
                                spawn(async move {
                                    tokio::time::sleep(tokio::time::Duration::from_secs(4)).await;
                                    let mut t = toast_rm();
                                    t.retain(|(i, _)| *i != id);
                                    toast_rm.set(t);
                                });
                            }
                        }
                    });
                },
                on_load_older: move |arg: (String, String)| {
                    let (channel_id, before_message_id) = arg;
                    let tok = match token() {
                        Some(t) => t,
                        None => return,
                    };
                    if loading_older() {
                        return;
                    }
                    loading_older.set(true);
                    let mut msgs = messages;
                    let mut has_more = has_more_older;
                    let mut loading = loading_older;
                    spawn(async move {
                        if let Ok(api_msgs) = http::fetch_channel_messages_before(&tok, &channel_id, &before_message_id, 50).await {
                            let older: Vec<Message> = api_msgs
                                .into_iter()
                                .map(|m| Message {
                                    id: m.id,
                                    channel_id: m.channel_id,
                                    author_id: m.author.as_ref().map(|a| a.id.clone()).unwrap_or_default(),
                                    author_username: m
                                        .author
                                        .as_ref()
                                        .map(|a| a.global_name.clone().or(Some(a.username.clone())).unwrap()),
                                    content: m.content,
                                    timestamp: m.timestamp,
                                    sending: false,
                                })
                                .collect();
                            has_more.set(older.len() == 50);
                            let mut list = msgs();
                            list.splice(0..0, older.into_iter().rev());
                            msgs.set(list);
                        }
                        loading.set(false);
                    });
                },
                on_trigger_typing: {
                    let settings_sig = settings;
                    move |channel_id: String| {
                        let tok = match token() {
                            Some(t) => t,
                            None => return,
                        };
                        // Respect ghost typing setting: when enabled, do not send typing events.
                        if settings_sig().ghost_typing {
                            return;
                        }
                        spawn(async move {
                            let _ = http::trigger_typing(&tok, &channel_id).await;
                        });
                    }
                },
                on_open_friend: move |user_id: String| {
                    let tok = match token() {
                        Some(t) => t,
                        None => return,
                    };
                    let mut chs = dm_channels;
                    let mut sel = selected_channel_id;
                    spawn(async move {
                        if let Ok(ch) = http::create_dm(&tok, &user_id).await {
                            let mut list = chs();
                            let exists = list.iter().any(|c| c.id == ch.id);
                            if !exists {
                                list.insert(0, ch.clone());
                                chs.set(list);
                            }
                            sel.set(Some(ch.id));
                        }
                    });
                },
                on_logout: move |_| {
                    let _ = logout();
                    token.set(None);
                    current_user.set(None);
                    friends.set(Vec::new());
                    dm_channels.set(Vec::new());
                    selected_channel_id.set(None);
                },
                on_open_settings: move |_| settings_open.set(true),
            }
        }
    } else {
        rsx! {
            LoginForm {
                token_input,
                login_error,
                login_loading,
                on_submit: move |t: String| {
                    let t = t.trim().to_string();
                    if t.is_empty() {
                        return;
                    }
                    login_loading.set(true);
                    login_error.set(None);
                    spawn(async move {
                        match http::verify_token(&t).await {
                            Ok(user) => {
                                if let Err(e) = login(t.clone()) {
                                    login_error.set(Some(e));
                                    login_loading.set(false);
                                    return;
                                }
                                token.set(Some(t));
                                current_user.set(Some(user));
                                login_error.set(None);
                            }
                            Err(e) => login_error.set(Some(e)),
                        }
                        login_loading.set(false);
                    });
                },
            }
        }
    };

    let animations_on = settings().animations_enabled;
    let global_css = "html,body{margin:0;padding:0;border:none;outline:none;background:#0a0a0f}*{box-sizing:border-box}\
        @keyframes message-load-spin{to{transform:rotate(360deg)}}\
        @keyframes modal-fade-in{from{opacity:0}to{opacity:1}}\
        @keyframes modal-fade-out{from{opacity:1}to{opacity:0}}\
        @keyframes modal-scale-in{from{opacity:0;transform:scale(0.96)}to{opacity:1;transform:scale(1)}}\
        @keyframes modal-scale-out{from{opacity:1;transform:scale(1)}to{opacity:0;transform:scale(0.96)}}\
        .anim-modal-backdrop{animation:modal-fade-in 0.2s ease-out}\
        .anim-modal-backdrop.closing{animation:modal-fade-out 0.2s ease-out forwards}\
        .anim-modal-content{animation:modal-scale-in 0.25s ease-out}\
        .anim-modal-content.closing{animation:modal-scale-out 0.2s ease-in forwards}\
        .anim-btn{transition:opacity 0.15s,transform 0.1s,background 0.15s}\
        .anim-btn:hover{opacity:0.9}\
        .anim-channel-item{transition:background 0.15s}\
        .anim-channel-item:hover{background:rgba(255,255,255,0.06)}\
        .anim-message-row{animation:modal-fade-in 0.2s ease-out}\
        .animations-disabled *{animation:none!important;transition:none!important}\
        .custom-scroll{scrollbar-width:thin;scrollbar-color:rgba(0,255,245,0.3) transparent}\
        .custom-scroll::-webkit-scrollbar{width:6px;height:6px}\
        .custom-scroll::-webkit-scrollbar-track{background:transparent}\
        .custom-scroll::-webkit-scrollbar-thumb{background:rgba(0,255,245,0.25);border-radius:3px}\
        .custom-scroll::-webkit-scrollbar-thumb:hover{background:rgba(0,255,245,0.4)}\
        .custom-scroll::-webkit-scrollbar-corner{background:transparent}";

    rsx! {
        link {
            rel: "stylesheet",
            href: "https://fonts.googleapis.com/css2?family=Outfit:wght@400;500;600;700&display=swap",
        }
        style { "{global_css}" }
        div {
            class: if animations_on { "app-root" } else { "app-root animations-disabled" },
            style: "
                display: flex; flex-direction: column;
                position: absolute; inset: 0;
                margin: 0; padding: 0; border: none; outline: none;
                background: #0a0a0f;
                font-family: 'Outfit', system-ui, sans-serif;
                overflow: hidden;
            ",
            {main_content}
            WelcomeModal {
                settings,
                on_dismiss: move |_| {},
            }
            SettingsModal {
                open: settings_open,
                settings,
                current_user,
                on_close: move |_| settings_open.set(false),
                on_show_toast: move |msg: String| {
                    let id = toast_counter() + 1;
                    toast_counter.set(id);
                    let mut list = toast_messages();
                    list.push((id, msg));
                    toast_messages.set(list);
                    let mut toast = toast_messages;
                    spawn(async move {
                        tokio::time::sleep(tokio::time::Duration::from_secs(4)).await;
                        let mut t = toast();
                        t.retain(|(i, _)| *i != id);
                        toast.set(t);
                    });
                },
                on_change_presence: move |status: PresenceStatus| {
                    if let Some(tx) = presence_tx() {
                        let _ = tx.send(status);
                    }
                },
            }
            MetricsOverlay { visible: settings().show_metrics_overlay }
            ToastContainer { messages: toast_messages }
        }
    }
}
