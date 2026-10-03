use crate::client::Intent;
use crate::client::LocalModel;
use dioxus::prelude::*;
use kernel::new_types::BranchUuid;
use kernel::new_types::CompanyUuid;
use serde::Deserialize;
use serde::Serialize;
use use_case_get_companies_and_branches::domain::CompanyWithBranches;
use utility_ui::domain::HashimSignal;
use utility_ui::my_signal::MySignal;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct TypeLocalModel {
    list_of_companies: MySignal<Vec<CompanyWithBranches>>,
    list_companies: MySignal<Vec<(CompanyUuid, String)>>,
    list_branches: MySignal<Vec<(BranchUuid, String)>>,
    is_loading: MySignal<bool>,
}

impl LocalModel for TypeLocalModel {
    fn list_of_companies_and_branches(&self) -> impl HashimSignal<Vec<CompanyWithBranches>> {
        self.list_of_companies.clone()
    }

    fn list_of_companies(&self) -> impl HashimSignal<Vec<(CompanyUuid, String)>> {
        self.list_companies.clone()
    }

    fn list_of_branches(&self) -> impl HashimSignal<Vec<(BranchUuid, String)>> {
        self.list_branches.clone()
    }

    fn is_loading(&self) -> impl HashimSignal<bool> {
        self.is_loading.clone()
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

    let company_label = match &selected_company_name {
        Some(n) => n.clone(),
        None => "select company".to_string(),
    };
    let branch_label = match &selected_branch_name {
        Some(n) => n.clone(),
        None => "select branch".to_string(),
    };
    let has_company = selected_company.is_some();

    rsx! {
        div {
            span { "{user_name}" }
            span { " > " }

            div {
                button {
                    onclick: move |_| {
                        // Use `*write()` exactly like PasswordInput does.
                        *show_companies.write() ^= true;
                        *show_branches.write() = false;
                    },
                    "{company_label}"
                }

                if companies_open {
                    div {
                        if companies.is_empty() {
                            // So you can *see* the dropdown is open but
                            // the list hasn't arrived yet.
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
