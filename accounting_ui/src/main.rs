mod signals_adapters;
mod ui;

pub mod prelude {
    pub(crate) use crate::{
        signals_adapters::{MyAllSignalTypes, MySignal},
        ui,
    };
    pub(crate) use adapters::prelude::*;
    pub(crate) use cache_rusqlite::prelude::*;
    pub(crate) use my_core::prelude::{Signal as HashimSignal, *};
}

fn main() {
    dioxus_logger::init(dioxus_logger::tracing::Level::INFO).unwrap();
    dioxus::launch(crate::ui::App);
}
