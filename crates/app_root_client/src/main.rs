mod app_ui;
mod model;
mod use_case_home;
mod use_case_navigator;
mod utils;
mod wire;

use dioxus::launch;

#[cfg(target_arch = "wasm32")]
fn main() {
    use console_error_panic_hook::set_once;
    set_once();

    use tracing_subscriber::prelude::*;
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .without_time()
                .with_target(false)
                .with_level(false)
                .with_ansi(false)
                .with_writer(tracing_web::MakeWebConsoleWriter::new()),
        )
        .init();

    launch(app_ui::App);
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // stderr only — stdout belongs to the MCP wire on native.
    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .with_level(false)
        .with_writer(std::io::stderr)
        .init();

    // 1. Send one job to the executor thread that:
    //    - forces the LazyLocks (Commander, MODEL)
    //    - spawns the commander actor, cache actor, network actor
    //    - spawns the MCP stdio server (if enabled)
    // All those `Rt::spawn_local` calls run *inside* a LocalSet,
    // so they no longer panic — and the futures stay `!Send`.
    infrastructure::runtime::target::run_on_executor(|| {
        utils::init_commander_and_model();
    });

    // 2. Main thread: no tokio runtime active.
    //    Dioxus's `launch` panics if it finds one, and it panics for good reason.
    launch(app_ui::App);
}
