mod app_ui;
mod model;
mod navigator;
mod utils;
mod wire;

use dioxus::launch;
use dioxus_logger::init;
use dioxus_logger::tracing::Level;

fn main() {
    init(Level::INFO).unwrap();
    launch(app_ui::App);
}
