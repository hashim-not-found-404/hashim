mod app_ui;
mod model;
mod navigator;
mod utils;
mod wire;

use console_error_panic_hook::set_once;
use dioxus::launch;

fn main() {
    set_once();

    #[cfg(target_arch = "wasm32")]
    {
        use tracing_subscriber::prelude::*;

        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .without_time() // remove timestamp
                    .with_target(false) // remove file:line
                    .with_level(false) // remove DEBUG/TRACE
                    .with_ansi(false) // remove %c color codes
                    .with_writer(tracing_web::MakeWebConsoleWriter::new()),
            )
            .init();
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        tracing_subscriber::fmt()
            .without_time()
            .with_target(false)
            .with_level(false)
            .init();
    }

    launch(app_ui::App);
}
