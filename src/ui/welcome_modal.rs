use dioxus::prelude::*;

use crate::state::{save_settings, AppSettings};

#[component]
pub fn WelcomeModal(settings: Signal<AppSettings>, on_dismiss: EventHandler<()>) -> Element {
    if settings().welcome_seen {
        return rsx! {};
    }

    rsx! {
        div {
            class: "anim-modal-backdrop",
            style: "
                position: fixed; inset: 0;
                background: rgba(0,0,0,0.7);
                display: flex; align-items: center; justify-content: center;
                z-index: 1100;
            ",
            onclick: move |_| {
                let mut s = settings();
                s.welcome_seen = true;
                settings.set(s.clone());
                let _ = save_settings(&s);
                on_dismiss.call(());
            },
            div {
                class: "anim-modal-content",
                style: "
                    background: linear-gradient(135deg, #12121a 0%, #0d0d12 100%);
                    border: 1px solid rgba(0,255,245,0.2);
                    border-radius: 16px;
                    padding: 2rem;
                    max-width: 24rem;
                    text-align: center;
                    box-shadow: 0 0 48px rgba(0,255,245,0.08);
                ",
                onclick: move |evt| evt.stop_propagation(),
                div {
                    style: "font-size: 2.5rem; margin-bottom: 1rem;",
                    "🚀"
                }
                h2 {
                    style: "margin: 0 0 1rem 0; font-size: 1.5rem; color: #e5e7eb;",
                    "Welcome to Velocity"
                }
                p {
                    style: "
                        margin: 0 0 1.25rem 0;
                        color: #9ca3af;
                        font-size: 0.9375rem;
                        line-height: 1.5;
                    ",
                    "This is a beta Discord client. Some features may be unstable or missing. "
                    "Your feedback helps us improve."
                }
                p {
                    style: "
                        margin: 0 0 1.5rem 0;
                        color: #6b7280;
                        font-size: 0.8125rem;
                        line-height: 1.4;
                    ",
                    "Use at your own risk. Always keep your token secure."
                }
                button {
                    class: "anim-btn",
                    style: "
                        padding: 0.625rem 1.5rem; font-size: 0.9375rem; font-weight: 500;
                        background: rgba(0,255,245,0.2); border: 1px solid rgba(0,255,245,0.4);
                        border-radius: 8px; color: #00fff5; cursor: pointer;
                    ",
                    onclick: move |_| {
                        let mut s = settings();
                        s.welcome_seen = true;
                        settings.set(s.clone());
                        let _ = save_settings(&s);
                        on_dismiss.call(());
                    },
                    "Got it"
                }
            }
        }
    }
}
