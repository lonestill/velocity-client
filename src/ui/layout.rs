use dioxus::prelude::*;
use std::collections::HashMap;

use crate::http::{DiscordUser, DmChannel, Relationship};
use crate::state::{AppSettings, Guild, Message};

use super::{ChannelList, MessageList, Sidebar};

#[component]
pub fn Layout(
    guilds: Signal<Vec<Guild>>,
    friends: Signal<Vec<Relationship>>,
    dm_channels: Signal<Vec<DmChannel>>,
    messages: Signal<Vec<Message>>,
    current_user: Signal<Option<DiscordUser>>,
    selected_channel_id: Signal<Option<String>>,
    has_more_older: Signal<bool>,
    loading_older: Signal<bool>,
    loading_messages: Signal<bool>,
    settings: Signal<AppSettings>,
    unread_counts: Signal<HashMap<String, u32>>,
    typing_users: Signal<HashMap<String, std::collections::HashMap<String, i64>>>,
    on_select_channel: EventHandler<Option<String>>,
    on_send_message: EventHandler<(String, String)>,
    on_load_older: EventHandler<(String, String)>,
    on_open_friend: EventHandler<String>,
    on_trigger_typing: EventHandler<String>,
    on_logout: EventHandler<()>,
    on_open_settings: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            style: "display: flex; flex: 1 1 0; min-height: 0; overflow: hidden; width: 100%;",
            Sidebar {
                guilds,
                current_user,
                on_logout,
                on_open_settings,
            }
            div {
                class: "glass-panel channel-list",
                style: "flex: 0 0 15rem; display: flex; flex-direction: column; min-height: 0; overflow: hidden; border-right: 1px solid rgba(255,255,255,0.1);",
                ChannelList {
                    friends,
                    dm_channels,
                    selected_channel_id,
                    unread_counts,
                    on_select_channel,
                    on_open_friend,
                    on_mark_read: move |id: String| {
                        let mut counts = unread_counts();
                        counts.insert(id, 0);
                        unread_counts.set(counts);
                    },
                }
            }
            div {
                style: "flex: 1 1 0; min-width: 0; min-height: 0; overflow: hidden; display: flex; flex-direction: column;",
                {if selected_channel_id().is_some() {
                    rsx! {
                        MessageList {
                            messages,
                            selected_channel_id,
                            dm_channels,
                            current_user,
                            has_more_older,
                            loading_older,
                            loading_messages,
                            typing_users,
                            on_send_message,
                            on_load_older,
                            on_trigger_typing,
                        }
                    }
                } else {
                    rsx! {
                        div {
                            style: "
                                flex: 1; display: flex; flex-direction: column;
                                align-items: center; justify-content: center;
                                background: #0a0a0f; color: #6b7280;
                            ",
                            div {
                                style: "
                                    text-align: center; padding: 2rem;
                                    max-width: 20rem;
                                ",
                                div {
                                    style: "
                                        font-size: 3rem; margin-bottom: 1rem;
                                        opacity: 0.5;
                                    ",
                                    "💬"
                                }
                                h2 {
                                    style: "
                                        font-size: 1.25rem; font-weight: 600;
                                        color: #9ca3af; margin: 0 0 0.5rem 0;
                                    ",
                                    "Select a conversation"
                                }
                                p {
                                    style: "font-size: 0.9375rem; margin: 0; line-height: 1.5;",
                                    "Choose a chat from the list to start messaging, or open a friend to create a new DM."
                                }
                            }
                        }
                    }
                }}
            }
        }
    }
}
