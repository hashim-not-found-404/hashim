use crate::use_cases::create_account;
use crate::utility::tools;
use dioxus::prelude::*;
use my_core::accounting_client::client_domain::ui_model;
use my_core::accounting_client::client_domain::ui_model::HashimSignal;

#[component]
pub(crate) fn Home() -> Element {
    rsx! {
        div {
            div {
                h1 { "Home" }
                match tools::MODEL.navigator.read() {
                    ui_model::Navigator::Home(page) => {
                        if page.show_menu {
                            rsx! {}
                        } else {
                            rsx! {}
                        }
                    }
                    _ => rsx! {},
                }
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
                    ui_model::Navigator::Home(page) => {
                        match page.page_to_present {
                            ui_model::Menu::Dashboard => rsx! {},
                            ui_model::Menu::CreateAccount => rsx! {
                                create_account::CreateAccount {}
                            },
                        }
                    }
                    _ => rsx! {},
                }
            }
        }
    }
}
