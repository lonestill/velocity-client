use dioxus::prelude::*;

#[component]
pub fn ToastContainer(messages: Signal<Vec<(usize, String)>>) -> Element {
    let list = messages();
    if list.is_empty() {
        return rsx! {};
    }
    rsx! {
        div {
            style: "
                position: fixed; bottom: 1rem; left: 50%; transform: translateX(-50%);
                z-index: 2000; display: flex; flex-direction: column; gap: 0.5rem;
                pointer-events: none;
            ",
            for (id, msg) in list.iter() {
                div {
                    key: "{id}",
                    class: "anim-modal-backdrop",
                    style: "
                        padding: 0.75rem 1rem;
                        background: rgba(239,68,68,0.95);
                        color: white;
                        border-radius: 8px;
                        font-size: 0.875rem;
                        box-shadow: 0 4px 12px rgba(0,0,0,0.3);
                        max-width: 24rem;
                    ",
                    "{msg}"
                }
            }
        }
    }
}
