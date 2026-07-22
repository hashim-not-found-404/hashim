mod ui;
mod use_cases;
mod utils;

fn main() {
    dioxus_logger::init(dioxus_logger::tracing::Level::INFO).unwrap();
    dioxus::launch(crate::ui::App);
}
