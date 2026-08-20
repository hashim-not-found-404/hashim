mod ui;
mod use_cases;
mod utility;

use dioxus::launch;
use dioxus_logger::init;
use dioxus_logger::tracing::Level;

fn main() {
    init(Level::INFO).unwrap();
    launch(ui::App);
}
