use crate::use_cases::create_account;
use crate::use_cases::create_account_for_branch;
use crate::use_cases::create_journal_entry;
use crate::utility::tools;
use dioxus::prelude::*;
use my_core::client::utility::ui_model;
use my_core::client::utility::ui_model::HashimSignal;

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
                button {
                    onclick: move |_| {
                        tools::send(ui_model::Message::Home(ui_model::Home::ShowCreateAccountForBranch));
                    },
                    "Add New Account For Branch"
                }
                button {
                    onclick: move |_| {
                        tools::send(ui_model::Message::Home(ui_model::Home::ShowCreateJournalEntry));
                    },
                    "Add Journal Entry"
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
                            ui_model::Menu::CreateAccountForBranch => rsx! {
                                create_account_for_branch::CreateAccountForBranch {}
                            },
                            ui_model::Menu::CreateJournalEntry => rsx! {
                                create_journal_entry::CreateJournalEntry {}
                            },
                        }
                    }
                    _ => rsx! {},
                }
            }
        }
    }
}
