use dioxus::prelude::*;

use crate::http::DiscordUser;
use crate::state::{save_settings, AppSettings};
use crate::updater;

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

#[derive(Clone, Copy, PartialEq)]
enum SettingsTab {
    General,
    Appearance,
}

#[component]
pub fn SettingsModal(
    open: Signal<bool>,
    settings: Signal<AppSettings>,
    current_user: Signal<Option<DiscordUser>>,
    on_close: EventHandler<()>,
    on_show_toast: EventHandler<String>,
) -> Element {
    let mut closing = use_signal(|| false);

    use_effect(move || {
        if open() {
            closing.set(false);
        } else if closing() {
            closing.set(false);
        }
    });

    if !open() && !closing() {
        return rsx! {};
    }

    let mut active_tab = use_signal(|| SettingsTab::General);
    let mut update_available = use_signal(|| None::<String>);
    let s = settings();
    let user = current_user();
    let is_closing = closing();

    rsx! {
        div {
            class: if is_closing { "anim-modal-backdrop closing" } else { "anim-modal-backdrop" },
            style: "
                position: fixed; inset: 0;
                background: rgba(0,0,0,0.6);
                display: flex; align-items: center; justify-content: center;
                z-index: 1000;
            ",
            onclick: move |_| {
                if !closing() {
                    closing.set(true);
                    spawn(async move {
                        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                        on_close.call(());
                    });
                }
            },
            div {
                class: if is_closing { "anim-modal-content closing" } else { "anim-modal-content" },
                style: "
                    background: #12121a;
                    border: 1px solid rgba(255,255,255,0.12);
                    border-radius: 12px;
                    width: 70vw; height: 70vh;
                    max-width: 900px; max-height: 600px;
                    display: flex; overflow: hidden;
                    box-shadow: 0 8px 32px rgba(0,0,0,0.4);
                ",
                onclick: move |evt| evt.stop_propagation(),

                // Left sidebar
                aside {
                    style: "
                        width: 12rem; flex-shrink: 0;
                        background: rgba(0,0,0,0.2);
                        border-right: 1px solid rgba(255,255,255,0.08);
                        display: flex; flex-direction: column;
                    ",
                    // User block at top
                    div {
                        style: "
                            padding: 1rem;
                            border-bottom: 1px solid rgba(255,255,255,0.08);
                            display: flex; align-items: center; gap: 0.75rem;
                        ",
                        {user.as_ref().map(|u| {
                            let avatar_block = avatar_url(u).map(|url| {
                                rsx! {
                                    img {
                                        src: "{url}",
                                        alt: "",
                                        style: "width: 2.5rem; height: 2.5rem; border-radius: 50%; object-fit: cover;",
                                    }
                                }
                            }).unwrap_or_else(|| {
                                rsx! {
                                    div {
                                        style: "
                                            width: 2.5rem; height: 2.5rem; border-radius: 50%;
                                            background: rgba(0,255,245,0.2);
                                            display: flex; align-items: center; justify-content: center;
                                            font-size: 0.9rem; font-weight: 600; color: #00fff5;
                                        ",
                                        "{display_name(u).chars().next().unwrap_or('?')}"
                                    }
                                }
                            });
                            rsx! {
                                div {
                                    style: "display: flex; align-items: center; gap: 0.75rem; min-width: 0;",
                                    {avatar_block}
                                    span {
                                        style: "
                                            flex: 1; overflow: hidden; text-overflow: ellipsis;
                                            white-space: nowrap; color: #e5e7eb;
                                            font-size: 0.9375rem; font-weight: 500;
                                        ",
                                        title: "{display_name(u)}",
                                        "{display_name(u)}"
                                    }
                                }
                            }
                        }).unwrap_or_else(|| rsx! {
                            div {
                                style: "
                                    width: 2.5rem; height: 2.5rem; border-radius: 50%;
                                    background: rgba(255,255,255,0.1);
                                    display: flex; align-items: center; justify-content: center;
                                    font-size: 0.75rem; color: #6b7280;
                                ",
                                "?"
                            }
                            span { style: "color: #6b7280; font-size: 0.875rem;", "Not logged in" }
                        })}
                    }
                    // Category tabs
                    nav {
                        class: "custom-scroll",
                        style: "flex: 1; padding: 0.5rem 0; overflow-y: auto; min-height: 0;",
                        button {
                            class: "settings-tab anim-btn",
                            style: if active_tab() == SettingsTab::General {
                                "
                                    width: 100%; padding: 0.5rem 1rem; text-align: left;
                                    background: rgba(0,255,245,0.1); color: #00fff5;
                                    border: none; font-size: 0.9375rem; cursor: pointer;
                                    border-left: 2px solid #00fff5;
                                "
                            } else {
                                "
                                    width: 100%; padding: 0.5rem 1rem; text-align: left;
                                    background: transparent; color: #9ca3af;
                                    border: none; font-size: 0.9375rem; cursor: pointer;
                                    border-left: 2px solid transparent;
                                "
                            },
                            onclick: move |_| active_tab.set(SettingsTab::General),
                            "General"
                        }
                        button {
                            class: "settings-tab anim-btn",
                            style: if active_tab() == SettingsTab::Appearance {
                                "
                                    width: 100%; padding: 0.5rem 1rem; text-align: left;
                                    background: rgba(0,255,245,0.1); color: #00fff5;
                                    border: none; font-size: 0.9375rem; cursor: pointer;
                                    border-left: 2px solid #00fff5;
                                "
                            } else {
                                "
                                    width: 100%; padding: 0.5rem 1rem; text-align: left;
                                    background: transparent; color: #9ca3af;
                                    border: none; font-size: 0.9375rem; cursor: pointer;
                                    border-left: 2px solid transparent;
                                "
                            },
                            onclick: move |_| active_tab.set(SettingsTab::Appearance),
                            "Appearance"
                        }
                    }
                }

                // Main content
                main {
                    class: "custom-scroll",
                    style: "
                        flex: 1; padding: 0; overflow-y: auto; min-height: 0;
                        display: flex; flex-direction: column;
                    ",
                    header {
                        style: "
                            flex-shrink: 0; padding: 1rem 1.5rem;
                            border-bottom: 1px solid rgba(255,255,255,0.08);
                            display: flex; align-items: center; justify-content: space-between;
                        ",
                        h2 {
                            style: "margin: 0; font-size: 1.125rem; color: #e5e7eb;",
                            "Settings"
                        }
                        button {
                            class: "anim-btn",
                            style: "
                                padding: 0.375rem 0.75rem; font-size: 0.875rem;
                                background: rgba(255,255,255,0.08); border: 1px solid rgba(255,255,255,0.15);
                                border-radius: 6px; color: #9ca3af; cursor: pointer;
                            ",
                            onclick: move |_| {
                                if !closing() {
                                    closing.set(true);
                                    spawn(async move {
                                        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                                        on_close.call(());
                                    });
                                }
                            },
                            "Close"
                        }
                    }
                    {match active_tab() {
                        SettingsTab::General => rsx! {
                            div {
                                style: "padding: 1.5rem;",
                                h3 {
                                    style: "margin: 0 0 1rem 0; font-size: 1rem; color: #9ca3af;",
                                    "General"
                                }
                                div {
                                    style: "display: flex; flex-direction: column; gap: 1rem;",
                                    label {
                                        style: "display: flex; align-items: center; gap: 0.75rem; cursor: pointer;",
                                        input {
                                            r#type: "checkbox",
                                            checked: "{s.show_metrics_overlay}",
                                            oninput: move |evt| {
                                                let mut s = settings();
                                                s.show_metrics_overlay = evt.checked();
                                                settings.set(s.clone());
                                                let _ = save_settings(&s);
                                            },
                                        }
                                        span { style: "color: #e5e7eb; font-size: 0.9375rem;", "Show metrics overlay" }
                                    }
                                    div {
                                        style: "display: flex; flex-direction: column; gap: 0.75rem; margin-top: 0.5rem;",
                                        div {
                                            style: "display: flex; align-items: center; gap: 1rem;",
                                            span { style: "color: #9ca3af; font-size: 0.875rem;", "Version {crate::VERSION}" }
                                            button {
                                                class: "anim-btn",
                                                style: "
                                                    padding: 0.375rem 0.75rem; font-size: 0.875rem;
                                                    background: rgba(0,255,245,0.15); border: 1px solid rgba(0,255,245,0.4);
                                                    border-radius: 6px; color: #00fff5; cursor: pointer;
                                                ",
                                                onclick: move |_| {
                                                    let toast = on_show_toast;
                                                    let mut update_avail = update_available;
                                                    spawn(async move {
                                                        match updater::check_for_updates() {
                                                            Ok(Some(ver)) => {
                                                                update_avail.set(Some(ver.clone()));
                                                                toast.call(format!("Update v{ver} available. Click Update below to install."));
                                                            }
                                                            Ok(None) => {
                                                                update_avail.set(None);
                                                                toast.call("You're on the latest version.".to_string());
                                                            }
                                                            Err(e) => {
                                                                update_avail.set(None);
                                                                toast.call(format!("Update check failed: {e}"));
                                                            }
                                                        }
                                                    });
                                                },
                                                "Check for updates"
                                            }
                                        }
                                        if let Some(ref ver) = update_available() {
                                            button {
                                                class: "anim-btn",
                                                style: "
                                                    padding: 0.5rem 1rem; font-size: 0.875rem;
                                                    background: rgba(34,197,94,0.2); border: 1px solid rgba(34,197,94,0.5);
                                                    border-radius: 6px; color: #22c55e; cursor: pointer;
                                                    align-self: flex-start;
                                                ",
                                                onclick: move |_| {
                                                    let toast = on_show_toast;
                                                    spawn(async move {
                                                        let result = tokio::task::spawn_blocking(|| updater::perform_update()).await;
                                                        match result {
                                                            Ok(Ok(())) => {
                                                                toast.call("Update complete. Restarting...".to_string());
                                                            }
                                                            Ok(Err(e)) => {
                                                                toast.call(format!("Update failed: {e}"));
                                                            }
                                                            Err(e) => {
                                                                toast.call(format!("Update failed: {e}"));
                                                            }
                                                        }
                                                    });
                                                },
                                                "Update to v{ver}"
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        SettingsTab::Appearance => rsx! {
                            div {
                                style: "padding: 1.5rem;",
                                h3 {
                                    style: "margin: 0 0 1rem 0; font-size: 1rem; color: #9ca3af;",
                                    "Animations"
                                }
                                div {
                                    style: "display: flex; flex-direction: column; gap: 1rem;",
                                    label {
                                        style: "display: flex; align-items: center; gap: 0.75rem; cursor: pointer;",
                                        input {
                                            r#type: "checkbox",
                                            checked: "{s.animations_enabled}",
                                            oninput: move |evt| {
                                                let mut s = settings();
                                                s.animations_enabled = evt.checked();
                                                settings.set(s.clone());
                                                let _ = save_settings(&s);
                                            },
                                        }
                                        span { style: "color: #e5e7eb; font-size: 0.9375rem;", "Enable UI animations" }
                                    }
                                    p {
                                        style: "margin: 0; color: #6b7280; font-size: 0.8125rem; line-height: 1.4;",
                                        "Transitions, fade effects, and hover animations."
                                    }
                                }
                            }
                        },
                    }}
                }
            }
        }
    }
}
