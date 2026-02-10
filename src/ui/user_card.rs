use dioxus::prelude::*;

use crate::http::DiscordUser;

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

/// anchor_right: when true (for "my" messages on the right), position card to the left of (x,y)
#[component]
pub fn UserCard(
    user: DiscordUser,
    x: i32,
    y: i32,
    anchor_right: bool,
    on_close: EventHandler<()>,
) -> Element {
    let avatar = avatar_url(&user);
    let name = display_name(&user);
    let pos_style = if anchor_right {
        format!("position: fixed; left: {}px; top: {}px; transform: translateX(-100%);", x, y)
    } else {
        format!("position: fixed; left: {}px; top: {}px;", x, y)
    };

    rsx! {
        div {
            style: "position: fixed; inset: 0; z-index: 1499;",
            onclick: move |_| on_close.call(()),
            oncontextmenu: move |evt| {
                evt.prevent_default();
                on_close.call(());
            },
        }
        div {
            class: "anim-modal-content",
            style: "
                {pos_style}
                background: #12121a;
                border: 1px solid rgba(255,255,255,0.12);
                border-radius: 12px;
                padding: 1rem;
                min-width: 12rem;
                box-shadow: 0 8px 24px rgba(0,0,0,0.4);
                z-index: 1500;
            ",
            div {
                style: "display: flex; align-items: center; gap: 0.75rem;",
                {match avatar {
                    Some(url) => rsx! {
                        img {
                            src: "{url}",
                            alt: "",
                            style: "width: 3rem; height: 3rem; border-radius: 50%; object-fit: cover;",
                        }
                    },
                    None => rsx! {
                        div {
                            style: "
                                width: 3rem; height: 3rem; border-radius: 50%;
                                background: rgba(0,255,245,0.2);
                                display: flex; align-items: center; justify-content: center;
                                font-size: 1.25rem; font-weight: 600; color: #00fff5;
                            ",
                            "{name.chars().next().unwrap_or('?')}"
                        }
                    },
                }}
                div {
                    style: "flex: 1; min-width: 0;",
                    div {
                        style: "font-weight: 600; color: #e5e7eb; font-size: 0.9375rem;",
                        "{name}"
                    }
                    div {
                        style: "font-size: 0.75rem; color: #6b7280;",
                        "@{user.username}"
                    }
                }
            }
        }
    }
}
