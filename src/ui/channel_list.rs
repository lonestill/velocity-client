use dioxus::prelude::*;

use crate::http::{DiscordUser, DmChannel, Relationship};
use crate::ui::ChannelContextMenu;

fn display_name(u: &DiscordUser) -> String {
    u.global_name
        .as_deref()
        .unwrap_or(u.username.as_str())
        .to_string()
}

fn avatar_url(u: &DiscordUser) -> Option<String> {
    u.avatar.as_ref().map(|hash| {
        let ext = if hash.starts_with("a_") { "gif" } else { "png" };
        format!(
            "https://cdn.discordapp.com/avatars/{}/{}.{}",
            u.id, hash, ext
        )
    })
}

fn dm_channel_label(ch: &DmChannel) -> String {
    if ch.recipients.is_empty() {
        ch.name.as_deref().unwrap_or("DM").to_string()
    } else {
        ch.recipients.iter().map(display_name).collect::<Vec<_>>().join(", ")
    }
}

#[component]
pub fn ChannelList(
    friends: Signal<Vec<Relationship>>,
    dm_channels: Signal<Vec<DmChannel>>,
    selected_channel_id: Signal<Option<String>>,
    unread_counts: Signal<std::collections::HashMap<String, u32>>,
    on_select_channel: EventHandler<Option<String>>,
    on_open_friend: EventHandler<String>,
    on_mark_read: EventHandler<String>,
) -> Element {
    let mut channel_context = use_signal(|| None::<(f64, f64, String)>);
    let friends_list = friends();
    let dm_list = dm_channels();
    let selected = selected_channel_id();
    let unread = unread_counts();
    let friends_filtered: Vec<(String, String, String, Option<String>)> = friends_list
        .iter()
        .filter(|r| r.r#type == 1)
        .map(|r| {
            (
                r.user.id.clone(),
                r.user.id.clone(),
                display_name(&r.user),
                avatar_url(&r.user),
            )
        })
        .collect();
    let mut seen_dm_ids = std::collections::HashSet::new();
    let dm_owned: Vec<(String, String, String, Option<String>, char)> = dm_list
        .iter()
        .filter(|ch| seen_dm_ids.insert(ch.id.clone()))
        .map(|ch| {
            let label = dm_channel_label(ch);
            let (avatar_opt, fallback) = ch
                .recipients
                .first()
                .map(|u| (avatar_url(u), display_name(u).chars().next().unwrap_or('?')))
                .unwrap_or((None, '?'));
            (ch.id.clone(), ch.id.clone(), label, avatar_opt, fallback)
        })
        .collect();

    let key_section_friends = "section-friends";
    let key_section_dms = "section-dms";
    let key_empty = "empty-state";

    rsx! {
        if let Some((x, y, ref ch_id)) = channel_context() {
            ChannelContextMenu {
                x,
                y,
                channel_id: ch_id.clone(),
                on_mark_read,
                on_close: move |_| channel_context.set(None),
            }
        }
        div {
            style: "
                display: flex; flex-direction: column;
                flex: 1; min-height: 0; overflow: hidden; height: 100%;
            ",
            header {
                style: "flex-shrink: 0; padding: 0.75rem; border-bottom: 1px solid rgba(255,255,255,0.1);",
                h2 {
                    style: "font-size: 0.875rem; font-weight: 600; color: #9ca3af; margin: 0;",
                    "Friends & DMs"
                }
            }
            ul {
                class: "custom-scroll",
                style: "
                    flex: 1 1 0; min-height: 0; overflow-y: auto; overflow-x: hidden;
                    padding: 0.5rem; list-style: none; margin: 0;
                ",
            li {
                key: "{key_section_friends}",
                style: "padding: 0.25rem 0.5rem; font-size: 0.65rem; text-transform: uppercase; letter-spacing: 0.05em; color: #6b7280;",
                "Friends"
            }
            for (uid, uid_click, name, avatar_opt) in friends_filtered.clone().into_iter() {
                li {
                    key: "friend-{uid}",
                    class: "anim-channel-item",
                    style: "
                        display: flex; align-items: center; gap: 0.5rem;
                        padding: 0.375rem 0.5rem; border-radius: 6px;
                        color: #e5e7eb; font-size: 0.875rem; cursor: pointer;
                    ",
                    onclick: move |_| on_open_friend.call(uid_click.clone()),
                    {match avatar_opt.as_ref() {
                        Some(url) => rsx! {
                            img {
                                src: "{url}",
                                alt: "",
                                style: "width: 1.75rem; height: 1.75rem; border-radius: 50%; object-fit: cover;",
                            }
                        },
                        None => rsx! {
                            div {
                                style: "
                                    width: 1.75rem; height: 1.75rem; border-radius: 50%;
                                    background: rgba(0,255,245,0.2);
                                    display: flex; align-items: center; justify-content: center;
                                    font-size: 0.65rem; font-weight: 600; color: #00fff5;
                                ",
                                "{name.chars().next().unwrap_or('?')}"
                            }
                        },
                    }}
                    span {
                        style: "flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                        title: "{name}",
                        "{name}"
                    }
                }
            }
            li {
                key: "{key_section_dms}",
                style: "padding: 0.25rem 0.5rem; font-size: 0.65rem; text-transform: uppercase; letter-spacing: 0.05em; color: #6b7280;",
                "Direct Messages"
            }
            for (ch_id, ch_id_click, label, avatar_opt, fallback) in dm_owned.clone().into_iter() {
                li {
                    key: "dm-{ch_id}",
                    class: "anim-channel-item",
                    style: if selected.as_ref() == Some(&ch_id) {
                        "display: flex; align-items: center; gap: 0.5rem; padding: 0.375rem 0.5rem; border-radius: 6px; color: #9ca3af; font-size: 0.875rem; cursor: pointer; background: rgba(0,255,245,0.1);"
                    } else {
                        "display: flex; align-items: center; gap: 0.5rem; padding: 0.375rem 0.5rem; border-radius: 6px; color: #9ca3af; font-size: 0.875rem; cursor: pointer; background: transparent;"
                    },
                    onclick: move |_| on_select_channel.call(Some(ch_id_click.clone())),
                    oncontextmenu: move |evt| {
                        evt.prevent_default();
                        let coords = evt.client_coordinates();
                        channel_context.set(Some((coords.x, coords.y, ch_id.clone())));
                    },
                    {match avatar_opt.as_ref() {
                        Some(url) => rsx! {
                            img {
                                src: "{url}",
                                alt: "",
                                style: "width: 1.75rem; height: 1.75rem; border-radius: 50%; object-fit: cover; flex-shrink: 0;",
                            }
                        },
                        None => rsx! {
                            div {
                                style: "
                                    width: 1.75rem; height: 1.75rem; border-radius: 50%;
                                    background: rgba(0,255,245,0.2);
                                    display: flex; align-items: center; justify-content: center;
                                    font-size: 0.65rem; font-weight: 600; color: #00fff5;
                                    flex-shrink: 0;
                                ",
                                "{fallback}"
                            }
                        },
                    }}
                    span {
                        style: "flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                        title: "{label}",
                        "{label}"
                    }
                    if unread.get(&ch_id).copied().unwrap_or(0) > 0 {
                        span {
                            style: "
                                background: #00fff5; color: #0a0a0f;
                                font-size: 0.65rem; font-weight: 700;
                                padding: 0.15em 0.4em; border-radius: 10px;
                                min-width: 1.25em; text-align: center;
                            ",
                            "{unread.get(&ch_id).copied().unwrap_or(0).min(99)}"
                        }
                    }
                }
            }
            if friends_filtered.is_empty() && dm_owned.is_empty() {
                li {
                    key: "{key_empty}",
                    style: "padding: 0.75rem; color: #6b7280; font-size: 0.8125rem;",
                    "No friends or DMs yet."
                }
            }
        }
        }
    }
}
