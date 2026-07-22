use crate::{
    use_cases::create_account,
    utils::{components, my_signals, tools},
};
use adapters::{
    actors, encode_decode, functions, random_number, row_id, runtime, web_socket_adapter,
};
use cache_rusqlite::{db_bundle, read_cases::utils};
use dioxus::prelude::*;
use my_core::accounting_client::{
    ui_construct, ui_effect,
    use_cases::client_domain::ui_model::{self, HashimSignal},
};
use std::sync::LazyLock;

#[component]
pub(crate) fn MyHome() -> Element {
    rsx! {
        div {
            div {
                h1 { "Home" }
                button {
                    onclick: move |_| {
                        tools::send(ui_model::Message::Home(ui_model::Home::ShowDashboard));
                    },
                    "Dashboard"
                }
                button {
                    onclick: move |_| {
                        tools::send(ui_model::Message::Home(ui_model::Home::ShowCreateAccount));
                    },
                    "Add New Account"
                }
            }

            div {
                match tools::MODEL.navigator.read() {
                    ui_model::Navigator::Auth(_) => {
                        rsx! {}
                    }
                    ui_model::Navigator::CompanyBranchSelection(_) => {
                        rsx! {}
                    }
                    ui_model::Navigator::Home(page) => {
                        match page {
                            ui_model::Menu::Dashboard => rsx! {

                            },
                            ui_model::Menu::CreateAccount => rsx! {
                                create_account::CreateAccount {}
                            },
                        }
                    }
                }
            }
        }
    }
}
