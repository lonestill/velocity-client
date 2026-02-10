use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct ContextMenuItem {
    pub label: String,
    pub on_click: Option<EventHandler<()>>,
}

/// Context menu for a channel: Mark as read.
#[component]
pub fn ChannelContextMenu(
    x: f64,
    y: f64,
    channel_id: String,
    on_mark_read: EventHandler<String>,
    on_close: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            style: "position: fixed; inset: 0; z-index: 1499;",
            onclick: move |_| on_close.call(()),
            oncontextmenu: move |evt| {
                evt.prevent_default();
                evt.stop_propagation();
                on_close.call(());
            },
        }
        div {
            class: "anim-modal-content",
            style: "
                position: fixed; left: {x}px; top: {y}px;
                background: #12121a;
                border: 1px solid rgba(255,255,255,0.12);
                border-radius: 8px;
                padding: 0.25rem;
                min-width: 10rem;
                box-shadow: 0 8px 24px rgba(0,0,0,0.4);
                z-index: 1500;
            ",
            oncontextmenu: move |evt| evt.prevent_default(),
            button {
                class: "anim-btn",
                style: "
                    display: block; width: 100%; padding: 0.5rem 0.75rem;
                    text-align: left; font-size: 0.875rem;
                    background: transparent; border: none;
                    color: #e5e7eb; cursor: pointer;
                    border-radius: 4px;
                ",
                onclick: move |_| {
                    on_mark_read.call(channel_id.clone());
                    on_close.call(());
                },
                "Mark as read"
            }
        }
    }
}

/// Context menu for a message: Copy text, etc.
#[component]
pub fn MessageContextMenu(
    x: f64,
    y: f64,
    content: String,
    on_close: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            style: "position: fixed; inset: 0; z-index: 1499;",
            onclick: move |_| on_close.call(()),
            oncontextmenu: move |evt| {
                evt.prevent_default();
                evt.stop_propagation();
                on_close.call(());
            },
        }
        div {
            class: "anim-modal-content",
            style: "
                position: fixed; left: {x}px; top: {y}px;
                background: #12121a;
                border: 1px solid rgba(255,255,255,0.12);
                border-radius: 8px;
                padding: 0.25rem;
                min-width: 10rem;
                box-shadow: 0 8px 24px rgba(0,0,0,0.4);
                z-index: 1500;
            ",
            oncontextmenu: move |evt| evt.prevent_default(),
            button {
                class: "anim-btn",
                style: "
                    display: block; width: 100%; padding: 0.5rem 0.75rem;
                    text-align: left; font-size: 0.875rem;
                    background: transparent; border: none;
                    color: #e5e7eb; cursor: pointer;
                    border-radius: 4px;
                ",
                onclick: move |_| {
                    if let Ok(mut clip) = arboard::Clipboard::new() {
                        let _ = clip.set_text(&content);
                    }
                    on_close.call(());
                },
                "Copy text"
            }
        }
    }
}

#[component]
pub fn ContextMenu(
    x: i32,
    y: i32,
    items: Vec<ContextMenuItem>,
    on_close: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            style: "position: fixed; inset: 0; z-index: 1499;",
            onclick: move |_| on_close.call(()),
            oncontextmenu: move |evt| {
                evt.prevent_default();
                evt.stop_propagation();
                on_close.call(());
            },
        }
        div {
            class: "anim-modal-content",
            style: "
                position: fixed; left: {x}px; top: {y}px;
                background: #12121a;
                border: 1px solid rgba(255,255,255,0.12);
                border-radius: 8px;
                padding: 0.25rem;
                min-width: 10rem;
                box-shadow: 0 8px 24px rgba(0,0,0,0.4);
                z-index: 1500;
            ",
            for item in items {
                if let Some(handler) = item.on_click {
                    button {
                        class: "anim-btn",
                        style: "
                            display: block; width: 100%; padding: 0.5rem 0.75rem;
                            text-align: left; font-size: 0.875rem;
                            background: transparent; border: none;
                            color: #e5e7eb; cursor: pointer;
                            border-radius: 4px;
                        ",
                        onclick: move |_| {
                            handler.call(());
                            on_close.call(());
                        },
                        "{item.label}"
                    }
                } else {
                    div {
                        style: "
                            padding: 0.5rem 0.75rem;
                            font-size: 0.75rem; color: #6b7280;
                        ",
                        "{item.label}"
                    }
                }
            }
        }
    }
}
