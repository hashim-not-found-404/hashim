mod app_ui;
mod model;
mod navigator;
mod utils;
mod wire;

use crate::utils::COMMANDER;
use crate::utils::MODEL;
use dioxus::launch;
use dioxus_logger::init;
use dioxus_logger::tracing::Level;
use std::sync::LazyLock;

fn main() {
    LazyLock::force(&MODEL);
    LazyLock::force(&COMMANDER);

    init(Level::INFO).unwrap();
    launch(app_ui::App);
}
