use crate::utility::my_signals;
use adapters::actors;
use adapters::encode_decode;
use adapters::functions;
use adapters::random_number;
use adapters::row_id;
use adapters::runtime;
use adapters::time;
use adapters::web_socket_adapter;
use cache_rusqlite::db_bundle;
use cache_rusqlite::utility;
use dioxus::prelude::*;
use my_core::accounting_client::client_domain::ui_model;
use my_core::accounting_client::ui_construct;
use my_core::accounting_client::ui_effect;
use std::sync::LazyLock;

type TheModel = ui_model::Model<my_signals::S>;
type TheCommander = ui_effect::Commander<actors::target::S>;

pub(crate) static MODEL: LazyLock<TheModel> = LazyLock::new(TheModel::default);

pub(crate) fn send(msg: ui_model::Message) {
    static COMMANDER: LazyLock<TheCommander> = LazyLock::new(|| {
        ui_construct::new::<
            random_number::target::S,
            runtime::target::S,
            row_id::target::S,
            actors::target::S,
            encode_decode::target::S,
            functions::target::S,
            time::target::S,
            utility::cache_adapter::S,
            web_socket_adapter::target::S,
            my_signals::S,
            db_bundle::S,
        >(&MODEL)
    });

    COMMANDER.send::<runtime::target::S>(msg);
}

pub(crate) const ICONS_SHOW: Asset = asset!("/assets/icons/show.png");
pub(crate) const ICONS_HIDE: Asset = asset!("/assets/icons/hide.png");
// const MAIN_CSS: Asset = asset!("/assets/main.css");
