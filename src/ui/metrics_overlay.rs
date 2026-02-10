use dioxus::prelude::*;

#[component]
pub fn MetricsOverlay(visible: bool) -> Element {
    if !visible {
        return rsx! {};
    }
    rsx! {
        div {
            class: "metrics-overlay",
            style: "
                position: fixed; bottom: 0.5rem; right: 0.5rem;
                font-size: 0.75rem; font-family: ui-monospace, monospace;
                color: #6b7280; background: rgba(0,0,0,0.5);
                padding: 0.25rem 0.5rem; border-radius: 4px;
            ",
            "Metrics"
        }
    }
}
