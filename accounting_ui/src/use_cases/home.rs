use crate::{
    use_cases::create_account,
    utils::tools,
};
use dioxus::prelude::*;
use my_core::accounting_client::use_cases::client_domain::ui_model::{self, HashimSignal};

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
