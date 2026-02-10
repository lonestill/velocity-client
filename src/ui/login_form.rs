use dioxus::prelude::*;

#[component]
pub fn LoginForm(
    token_input: Signal<String>,
    login_error: Signal<Option<String>>,
    login_loading: Signal<bool>,
    on_submit: EventHandler<String>,
) -> Element {
    let err = login_error();
    let loading = login_loading();
    let error_block = err.as_ref().map(|e| {
        rsx! {
            div {
                style: "
                    display: flex;
                    align-items: center;
                    gap: 0.5rem;
                    padding: 0.5rem 0;
                    color: #f87171;
                    font-size: 0.8125rem;
                ",
                span { style: "flex: 1;", "{e}" }
            }
        }
    });

    rsx! {
        div {
            style: "
                display: flex;
                align-items: center;
                justify-content: center;
                min-height: 100vh;
                width: 100%;
                background: linear-gradient(165deg, #0a0a0f 0%, #0d0d14 40%, #0a0a12 100%);
                position: relative;
                overflow: hidden;
                font-family: 'Outfit', system-ui, sans-serif;
            ",
            div {
                style: "
                    position: absolute;
                    inset: 0;
                    background-image:
                        linear-gradient(rgba(0, 255, 245, 0.03) 1px, transparent 1px),
                        linear-gradient(90deg, rgba(0, 255, 245, 0.03) 1px, transparent 1px);
                    background-size: 48px 48px;
                    pointer-events: none;
                ",
            }
            div {
                style: "
                    position: absolute;
                    width: 600px;
                    height: 600px;
                    top: -200px;
                    right: -200px;
                    background: radial-gradient(circle, rgba(0, 255, 245, 0.08) 0%, transparent 70%);
                    pointer-events: none;
                ",
            }
            div {
                style: "
                    position: absolute;
                    width: 400px;
                    height: 400px;
                    bottom: -100px;
                    left: -100px;
                    background: radial-gradient(circle, rgba(255, 0, 170, 0.05) 0%, transparent 70%);
                    pointer-events: none;
                ",
            }
            div {
                style: "
                    position: relative;
                    width: 100%;
                    max-width: 380px;
                    margin: 2rem;
                    padding: 2.5rem 2rem;
                    background: rgba(255, 255, 255, 0.03);
                    backdrop-filter: blur(20px);
                    border: 1px solid rgba(255, 255, 255, 0.08);
                    border-radius: 16px;
                    box-shadow:
                        0 0 0 1px rgba(0, 255, 245, 0.05),
                        0 24px 48px -12px rgba(0, 0, 0, 0.5),
                        inset 0 1px 0 rgba(255, 255, 255, 0.04);
                ",
                div {
                    style: "
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        margin-bottom: 0.5rem;
                    ",
                    span {
                        style: "
                            font-family: 'Outfit', system-ui, sans-serif;
                            font-size: 2rem;
                            font-weight: 700;
                            letter-spacing: -0.03em;
                            background: linear-gradient(135deg, #00fff5 0%, #00d4cc 50%, #00fff5 100%);
                            background-size: 200% 200%;
                            -webkit-background-clip: text;
                            color: transparent;
                            text-shadow: 0 0 40px rgba(0, 255, 245, 0.3);
                        ",
                        "Velocity"
                    }
                }
                p {
                    style: "
                        text-align: center;
                        color: #6b7280;
                        font-size: 0.875rem;
                        margin: 0 0 1.75rem 0;
                        letter-spacing: 0.02em;
                    ",
                    "High-performance Discord client"
                }
                label {
                    style: "
                        display: block;
                        color: #9ca3af;
                        font-size: 0.75rem;
                        font-weight: 500;
                        text-transform: uppercase;
                        letter-spacing: 0.08em;
                        margin-bottom: 0.5rem;
                    ",
                    "User token"
                }
                input {
                    class: "auth-input",
                    r#type: "password",
                    placeholder: "Paste your token here",
                    value: "{token_input()}",
                    disabled: loading,
                    oninput: move |ev| token_input.set(ev.value().clone()),
                    style: "
                        width: 100%;
                        box-sizing: border-box;
                        padding: 0.75rem 1rem;
                        margin-bottom: 0.25rem;
                        background: rgba(0, 0, 0, 0.25);
                        border: 1px solid rgba(255, 255, 255, 0.1);
                        border-radius: 10px;
                        color: #e5e7eb;
                        font-size: 0.9375rem;
                        outline: none;
                        transition: border-color 0.2s, box-shadow 0.2s;
                    ",
                }
                {error_block}
                button {
                    class: "auth-btn",
                    disabled: loading,
                    onclick: move |_| {
                        let t = token_input().trim().to_string();
                        if !t.is_empty() {
                            on_submit.call(t);
                        }
                    },
                    style: "
                        width: 100%;
                        margin-top: 1rem;
                        padding: 0.875rem 1.25rem;
                        background: linear-gradient(135deg, #00fff5 0%, #00c9c2 100%);
                        color: #0a0a0f;
                        border: none;
                        border-radius: 10px;
                        font-size: 0.9375rem;
                        font-weight: 600;
                        letter-spacing: 0.02em;
                        cursor: pointer;
                        transition: opacity 0.2s, transform 0.1s;
                        box-shadow: 0 0 24px rgba(0, 255, 245, 0.25);
                    ",
                    if loading { "Signing in…" } else { "Sign in" }
                }
                p {
                    style: "
                        margin-top: 1.25rem;
                        padding-top: 1rem;
                        border-top: 1px solid rgba(255, 255, 255, 0.06);
                        color: #4b5563;
                        font-size: 0.6875rem;
                        text-align: center;
                        line-height: 1.4;
                    ",
                    "Token is stored in your system keyring. Never share it."
                }
            }
        }
    }
}
