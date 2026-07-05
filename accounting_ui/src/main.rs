mod my_signal;
mod my_types;
mod ui;

fn main() {
    dioxus_logger::init(dioxus_logger::tracing::Level::INFO).unwrap();
    dioxus::launch(crate::ui::App);
}
