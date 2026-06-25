mod my_signal;
mod my_types;
mod ui;

pub mod prelude {
    pub(crate) use crate::*;
    pub(crate) use adapters::prelude::*;
    pub(crate) use cache_rusqlite::prelude::*;
    pub(crate) use my_core::prelude::*;
}

fn main() {
    dioxus_logger::init(dioxus_logger::tracing::Level::INFO).unwrap();
    dioxus::launch(crate::ui::App);
}
