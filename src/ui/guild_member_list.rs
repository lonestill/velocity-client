use dioxus::prelude::*;

use crate::http::{DiscordUser, GuildMember};

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

fn member_display_name(m: &GuildMember) -> String {
    m.nick
        .clone()
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| {
            m.user
                .as_ref()
                .map(|u| display_name(u).to_string())
                .unwrap_or_else(|| "Unknown".to_string())
        })
}

/// Precomputed row for one member to avoid .id in rsx.
struct MemberRow {
    name: String,
    avatar_url: Option<String>,
    fallback_char: char,
}

#[component]
fn MemberRowView(
    name: String,
    avatar_url: Option<String>,
    fallback_char: char,
) -> Element {
    let avatar_block = avatar_url.map(|url| {
        rsx! {
            img {
                src: "{url}",
                alt: "",
                style: "
                    width: 1.75rem; height: 1.75rem;
                    border-radius: 50%; object-fit: cover;
                ",
            }
        }
    }).unwrap_or_else(|| {
        rsx! {
            div {
                style: "
                    width: 1.75rem; height: 1.75rem;
                    border-radius: 50%;
                    background: rgba(0,255,245,0.2);
                    display: flex; align-items: center; justify-content: center;
                    font-size: 0.7rem; color: #00fff5;
                ",
                "{fallback_char}"
            }
        }
    });

    rsx! {
        div {
            style: "
                display: flex; align-items: center; gap: 0.5rem;
                padding: 0.35rem 0.75rem;
                border-radius: 0.25rem; margin: 0 0.25rem;
            ",
            {avatar_block}
            span {
                style: "
                    font-size: 0.875rem; color: #e5e7eb;
                    overflow: hidden; text-overflow: ellipsis;
                    white-space: nowrap;
                ",
                title: "{name}",
                "{name}"
            }
        }
    }
}

#[component]
pub fn GuildMemberList(
    guild_members: Signal<Vec<GuildMember>>,
    current_user: Signal<Option<DiscordUser>>,
) -> Element {
    let members = guild_members();
    let user = current_user();

    let member_rows: Vec<MemberRow> = members
        .iter()
        .map(|m| {
            let name = member_display_name(m);
            let (avatar_url, fallback_char) = m
                .user
                .as_ref()
                .map(|u| (avatar_url(u), display_name(u).chars().next().unwrap_or('?')))
                .unwrap_or((None, '?'));
            MemberRow {
                name,
                avatar_url,
                fallback_char,
            }
        })
        .collect();

    // When API returns empty (user token has no permission to list members), show at least "You".
    let rows_to_show: Vec<MemberRow> = if member_rows.is_empty() {
        user.as_ref()
            .map(|u| {
                let name = format!("{} (you)", display_name(u));
                let (avatar_url, fallback_char) = (
                    avatar_url(u),
                    display_name(u).chars().next().unwrap_or('?'),
                );
                MemberRow {
                    name,
                    avatar_url,
                    fallback_char,
                }
            })
            .into_iter()
            .collect()
    } else {
        member_rows
    };

    let show_api_note = members.is_empty() && user.is_some();

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
                "Members — {rows_to_show.len()}"
            }
            if show_api_note {
                p {
                    style: "
                        padding: 0 0.75rem 0.5rem;
                        font-size: 0.7rem; color: #6b7280;
                        margin: 0; line-height: 1.3;
                    ",
                    "Full list requires an iq < 1 developer"
                }
            }
            for r in rows_to_show.iter() {
                MemberRowView {
                    name: r.name.clone(),
                    avatar_url: r.avatar_url.clone(),
                    fallback_char: r.fallback_char,

                }
            }
        }
    }
}
