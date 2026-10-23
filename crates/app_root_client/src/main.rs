mod app_ui;
mod model;
mod use_case_navigator;
mod utils;
mod wire;

use anyhow::Result;
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
fn main() -> Result<()> {
    use std::sync::mpsc::channel;
    use std::thread::Builder;

    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .with_level(false)
        .with_writer(std::io::stderr)
        .init();

    let (init_done_tx, init_done_rx) = channel::<()>();
    Builder::new()
        .name("local-executor".into())
        .spawn(move || -> Result<()> {
            use crate::utils::init_commander_and_model;
            use std::future::pending;
            use tokio::runtime::Builder;
            use tokio::task::LocalSet;

            let rt = Builder::new_current_thread().enable_all().build()?;

            let local = LocalSet::new();
            local.block_on(&rt, async move {
                init_commander_and_model();
                let _ = init_done_tx.send(());

                pending::<()>().await;
            });

            Ok(())
        })?;

    init_done_rx.recv()?;

    launch(app_ui::App);

    Ok(())
}
