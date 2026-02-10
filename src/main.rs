mod app;
mod gateway;
mod http;
mod state;
mod ui;
mod updater;
#[cfg(feature = "voice")]
mod voice;
#[cfg(feature = "voice")]
mod voice_audio;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    #[cfg(feature = "desktop")]
    {
        use dioxus::prelude::LaunchBuilder;
        use dioxus_desktop::{Config, WindowBuilder};

        LaunchBuilder::new()
            .with_cfg(
                Config::new().with_window(

                    WindowBuilder::new().with_title("Velocity"),
                ),
            )
            .launch(app::App);
    }
    #[cfg(not(feature = "desktop"))]
    dioxus::launch(app::App);
}
