use dioxus::prelude::*;

use crate::http::{ApiGuild, DiscordUser};

/// Logo as base64 data URL — works with both cargo run and dx serve
fn logo_src() -> &'static str {
    use base64::Engine;
    static LOGO: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    LOGO.get_or_init(|| {
        let bytes = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/logo.jpg"));
        format!("data:image/jpeg;base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes))
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
    user.global_name
        .as_deref()
        .unwrap_or(user.username.as_str())
}

fn guild_icon_url(guild: &ApiGuild) -> Option<String> {
    guild.icon.as_ref().map(|hash| {
        let ext = if hash.starts_with("a_") { "gif" } else { "png" };
        format!(
            "https://cdn.discordapp.com/icons/{}/{}.{}",
            guild.id, hash, ext
        )
    })
}

/// Precomputed guild button to avoid .id in rsx.
struct GuildButtonEntry {
    gid: String,
    name: String,
    bg: &'static str,
    border: &'static str,
    letter: char,
    icon_url: Option<String>,
}

#[component]
fn GuildButton(
    gid: String,
    name: String,
    bg: &'static str,
    border: &'static str,
    letter: char,
    icon_url: Option<String>,
    on_select_guild: EventHandler<Option<String>>,
) -> Element {
    let content = icon_url
        .as_ref()
        .map(|url| {
            rsx! {
                img {
                    src: "{url}",
                    alt: "",
                    style: "
                        width: 2.1rem; height: 2.1rem;
                        border-radius: 50%; object-fit: cover;
                    ",
                }
            }
        })
        .unwrap_or_else(|| {
            rsx! {
                span {
                    style: "font-size: 0.9rem; font-weight: 600;",
                    "{letter}"
                }
            }
        });

    rsx! {
        button {
            key: "{gid}",
            class: "anim-btn",
            style: "
                width: 2.5rem; height: 2.5rem; border-radius: 50%;
                background: {bg};
                border: 1px solid {border};
                color: #d1d5db; cursor: pointer; font-size: 0.75rem;
                display: flex; align-items: center; justify-content: center;
            ",
            title: "{name}",
            onclick: move |_| on_select_guild.call(Some(gid.clone())),
            {content}
        }
    }
}

#[component]
pub fn Sidebar(
    guilds: Signal<Vec<ApiGuild>>,
    selected_guild_id: Signal<Option<String>>,
    on_select_guild: EventHandler<Option<String>>,
    current_user: Signal<Option<DiscordUser>>,
    on_logout: EventHandler<()>,
    on_open_settings: EventHandler<()>,
) -> Element {
    let list = guilds();
    let selected = selected_guild_id();
    let user = current_user();
    let logo = logo_src();

    let guild_buttons: Vec<GuildButtonEntry> = list
        .iter()
        .take(10)
        .map(|g| {
            let gid = g.id.clone();
            let name = g.name.clone();
            let is_sel = selected.as_ref() == Some(&g.id);
            let (bg, border) = if is_sel {
                ("rgba(0,255,245,0.25)", "rgba(0,255,245,0.4)")
            } else {
                ("rgba(255,255,255,0.1)", "transparent")
            };
            let letter = name.chars().next().unwrap_or('?');
             let icon_url = guild_icon_url(g);
            GuildButtonEntry {
                gid,
                name,
                bg,
                border,
                letter,
                icon_url,
            }
        })
        .collect();

    rsx! {
        aside {
            class: "glass-panel sidebar",
            style: "width: 4rem; flex-shrink: 0; display: flex; flex-direction: column; align-items: center; padding: 0.5rem 0; gap: 0.5rem;",
            div {
                style: "
                    width: 2.5rem; height: 2.5rem; min-width: 2.5rem; min-height: 2.5rem;
                    border-radius: 0.75rem; overflow: hidden;
                    display: flex; align-items: center; justify-content: center;
                    flex-shrink: 0;
                    background: rgba(0,255,245,0.15);
                    border: 1px solid rgba(0,255,245,0.3);
                ",
                title: "Velocity",
                img {
                    src: "{logo}",
                    alt: "Velocity",
                    style: "width: 100%; height: 100%; object-fit: cover; display: block;",
                }
            }
            {{
                let dm_bg = if selected.is_none() {
                    "rgba(0,255,245,0.25)"
                } else {
                    "rgba(255,255,255,0.1)"
                };
                let dm_border = if selected.is_none() {
                    "rgba(0,255,245,0.4)"
                } else {
                    "transparent"
                };
                rsx! {
                    button {
                        class: "anim-btn",
                        style: "
                            width: 2.5rem; height: 2.5rem; border-radius: 50%;
                            background: {dm_bg};
                            border: 1px solid {dm_border};
                            color: #d1d5db; cursor: pointer; font-size: 1rem;
                            display: flex; align-items: center; justify-content: center;
                        ",
                        title: "Direct Messages",
                        onclick: move |_| on_select_guild.call(None),
                        "💬"
                    }
                }
            }}
            for g_ent in guild_buttons.iter() {
                GuildButton {
                    gid: g_ent.gid.clone(),
                    name: g_ent.name.clone(),
                    bg: g_ent.bg,
                    border: g_ent.border,
                    letter: g_ent.letter,
                    icon_url: g_ent.icon_url.clone(),
                    on_select_guild,
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


