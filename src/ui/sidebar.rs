use dioxus::prelude::*;

use crate::http::DiscordUser;
use crate::state::Guild;

/// Logo embedded at compile time so it always displays regardless of asset bundling.
fn logo_data_url() -> &'static str {
    use base64::prelude::{Engine as _, BASE64_STANDARD};
    static LOGO_B64: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    LOGO_B64.get_or_init(|| {
        const LOGO: &[u8] = include_bytes!("../../assets/logo.jpg");
        format!("data:image/jpeg;base64,{}", BASE64_STANDARD.encode(LOGO))
    })
}

fn avatar_url(user: &DiscordUser) -> Option<String> {
    user.avatar.as_ref().map(|hash| {
        let ext = if hash.starts_with("a_") { "gif" } else { "png" };
        format!(
            "https://cdn.discordapp.com/avatars/{}/{}.{}",
            user.id, hash, ext
        )
    })
}

fn display_name(user: &DiscordUser) -> &str {
    user.global_name.as_deref().unwrap_or(user.username.as_str())
}

#[component]
pub fn Sidebar(
    guilds: Signal<Vec<Guild>>,
    current_user: Signal<Option<DiscordUser>>,
    on_logout: EventHandler<()>,
    on_open_settings: EventHandler<()>,
) -> Element {
    let list = guilds();
    let user = current_user();

    rsx! {
        aside {
            class: "glass-panel sidebar",
            style: "width: 4rem; flex-shrink: 0; display: flex; flex-direction: column; align-items: center; padding: 0.5rem 0; gap: 0.5rem;",
            div {
                style: "width: 2.5rem; height: 2.5rem; border-radius: 0.75rem; overflow: hidden; display: flex; align-items: center; justify-content: center;",
                img {
                    src: "{logo_data_url()}",
                    alt: "Velocity",
                    style: "width: 100%; height: 100%; object-fit: cover;",
                }
            }
            for guild in list.iter().take(10) {
                div {
                    key: "{guild.id}",
                    style: "width: 2.5rem; height: 2.5rem; border-radius: 50%; background: rgba(255,255,255,0.1); display: flex; align-items: center; justify-content: center; font-size: 0.75rem;",
                    title: "{guild.name}",
                    "{guild.name.chars().next().unwrap_or('?')}"
                }
            }
            button {
                class: "anim-btn",
                style: "
                    width: 2.5rem; height: 2.5rem; border-radius: 50%;
                    border: 1px solid rgba(255,255,255,0.15);
                    background: rgba(255,255,255,0.06);
                    color: #9ca3af; cursor: pointer; font-size: 0.9rem;
                    display: flex; align-items: center; justify-content: center;
                ",
                title: "Settings",
                onclick: move |_| on_open_settings.call(()),
                "⚙"
            }
            div { style: "flex: 1; min-height: 0.5rem;" }
            {user.as_ref().map(|u| {
                let avatar_block = avatar_url(u).map(|url| {
                    rsx! {
                        img {
                            src: "{url}",
                            alt: "Avatar",
                            style: "width: 2.25rem; height: 2.25rem; border-radius: 50%; object-fit: cover;",
                        }
                    }
                }).unwrap_or_else(|| {
                    rsx! {
                        div {
                            style: "
                                width: 2.25rem; height: 2.25rem; border-radius: 50%;
                                background: rgba(0,255,245,0.2);
                                display: flex; align-items: center; justify-content: center;
                                font-size: 0.75rem; font-weight: 600; color: #00fff5;
                            ",
                            "{display_name(u).chars().next().unwrap_or('?')}"
                        }
                    }
                });
                rsx! {
                    div {
                        style: "
                            display: flex; flex-direction: column; align-items: center; gap: 0.25rem;
                            padding: 0.25rem 0;
                            border-top: 1px solid rgba(255,255,255,0.08);
                        ",
                        {avatar_block}
                        span {
                            style: "
                                font-size: 0.6rem; color: #9ca3af;
                                max-width: 3rem; overflow: hidden; text-overflow: ellipsis;
                                white-space: nowrap; text-align: center;
                            ",
                            title: "{display_name(u)}",
                            "{display_name(u)}"
                        }
                    }
                }
            })}
            button {
                class: "anim-btn",
                style: "
                    width: 2.5rem; height: 2.5rem;
                    border-radius: 50%;
                    border: 1px solid rgba(255,255,255,0.15);
                    background: rgba(239,68,68,0.15);
                    color: #f87171;
                    cursor: pointer;
                    font-size: 0.65rem;
                    font-weight: 600;
                    display: flex; align-items: center; justify-content: center;
                ",
                title: "Log out",
                onclick: move |_| on_logout.call(()),
                "Out"
            }
        }
    }
}
