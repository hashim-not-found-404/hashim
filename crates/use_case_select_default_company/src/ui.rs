use crate::client::AsyncState;
use crate::client::Intent;
use crate::client::LocalModel;
use dioxus::prelude::*;
use kernel::new_types::BranchUuid;
use kernel::new_types::CompanyUuid;
use serde::Deserialize;
use serde::Serialize;
use utility_ui::domain::HashimSignal;
use utility_ui::my_signal::MySignal;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct TypeLocalModel {
    async_state: MySignal<AsyncState>,
}

impl LocalModel for TypeLocalModel {
    fn async_state(&self) -> impl HashimSignal<AsyncState> {
        self.async_state.clone()
    }
}

#[component]
pub fn Component(
    sender: EventHandler<Intent>,
    user_name: String,
    companies: Vec<(CompanyUuid, String)>,
    branches: Vec<(BranchUuid, String)>,
    selected_company: Option<CompanyUuid>,
    selected_company_name: Option<String>,
    selected_branch: Option<BranchUuid>,
    selected_branch_name: Option<String>,
) -> Element {
    use_effect(move || {
        sender(Intent::Subscribe);
    });

    use_drop(move || {
        sender(Intent::UnSubscribe);
    });

    let mut show_companies = use_signal(|| false);
    let mut show_branches = use_signal(|| false);

    let companies_open = *show_companies.read();
    let branches_open = *show_branches.read();

    let company_label = selected_company_name
        .clone()
        .unwrap_or_else(|| "select company".to_string());
    let branch_label = selected_branch_name
        .clone()
        .unwrap_or_else(|| "select branch".to_string());
    let has_company = selected_company.is_some();

    rsx! {
        div {
            span { "{user_name}" }
            span { " > " }

            div {
                button {
                    onclick: move |_| {
                        *show_companies.write() ^= true;
                        *show_branches.write() = false;
                    },
                    "{company_label}"
                }

                if companies_open {
                    div {
                        if companies.is_empty() {
                            span { "loading companies…" }
                        } else {
                            for (uuid, name) in companies.iter() {
                                {
                                    let uuid = uuid.clone();
                                    let name = name.clone();
                                    rsx! {
                                        button {
                                            onclick: move |_| {
                                                sender(Intent::SelectCompany(uuid.clone()));
                                                *show_companies.write() = false;
                                            },
                                            "{name}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            span { " > " }

            div {
                button {
                    disabled: !has_company,
                    onclick: move |_| {
                        if !has_company {
                            return;
                        }
                        *show_branches.write() ^= true;
                        *show_companies.write() = false;
                    },
                    "{branch_label}"
                }

                if branches_open && has_company {
                    div {
                        if branches.is_empty() {
                            span { "no branches for this company" }
                        } else {
                            for (uuid, name) in branches.iter() {
                                {
                                    let uuid = uuid.clone();
                                    let name = name.clone();
                                    rsx! {
                                        button {
                                            onclick: move |_| {
                                                sender(Intent::SelectBranch(uuid.clone()));
                                                *show_branches.write() = false;
                                            },
                                            "{name}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
