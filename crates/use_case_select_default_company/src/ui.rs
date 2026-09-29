use crate::client::LocalModel;
use crate::client::Message;
use dioxus::prelude::*;
use kernel::new_types::BranchUuid;
use kernel::new_types::CompanyUuid;
use serde::Deserialize;
use serde::Serialize;
use std::sync::Arc;
use std::sync::RwLock;
use use_case_get_companies_and_branches::domain::CompanyWithBranches;
use utility::types::ReadAndSet;
use utility::ui_effect::PageId;
use utility_ui::domain::HashimSignal;
use utility_ui::my_signal::MySignal;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct TypeLocalModel {
    #[serde(skip)]
    page_id: Arc<RwLock<Option<PageId>>>,

    list_of_companies: MySignal<Vec<CompanyWithBranches>>,
    list_companies: MySignal<Vec<(CompanyUuid, String)>>,
    list_branches: MySignal<Vec<(BranchUuid, String)>>,
    is_loading: MySignal<bool>,
}

impl LocalModel for TypeLocalModel {
    fn page_id(&self) -> impl ReadAndSet<Option<PageId>> {
        self.page_id.clone()
    }

    fn list_of_companies(&self) -> impl HashimSignal<Vec<CompanyWithBranches>> {
        self.list_of_companies.clone()
    }

    fn list_companies(&self) -> impl HashimSignal<Vec<(CompanyUuid, String)>> {
        self.list_companies.clone()
    }

    fn list_branches(&self) -> impl HashimSignal<Vec<(BranchUuid, String)>> {
        self.list_branches.clone()
    }

    fn is_loading(&self) -> impl HashimSignal<bool> {
        self.is_loading.clone()
    }
}

/// Top-left breadcrumb: `user > [company ▾] > [branch ▾]`.
///
/// * `user_name` — display label for the logged-in user.
/// * `companies` / `branches` — flat dropdown lists (`(uuid, name)`),
///   already filtered by the client for the current selection.
/// * The `*_name` props come straight from the global model so we don't
///   re-derive them from the list on every render.
#[component]
pub fn Component(
    sender: EventHandler<Message>,
    user_name: String,
    companies: Vec<(CompanyUuid, String)>,
    branches: Vec<(BranchUuid, String)>,
    selected_company: Option<CompanyUuid>,
    selected_company_name: Option<String>,
    selected_branch: Option<BranchUuid>,
    selected_branch_name: Option<String>,
) -> Element {
    use_effect(move || {
        sender(Message::Subscribe);
    });

    use_drop(move || {
        sender(Message::UnSubscribe);
    });

    // Local dropdown open/close state. Only one is ever open at a time.
    let mut company_open = use_signal(|| false);
    let mut branch_open = use_signal(|| false);

    let company_label = selected_company_name
        .clone()
        .unwrap_or_else(|| "select company".to_string());
    let branch_label = selected_branch_name
        .clone()
        .unwrap_or_else(|| "select branch".to_string());
    let has_company = selected_company.is_some();

    rsx! {
        div { class: "top-left-breadcrumb",
            span { class: "breadcrumb-user", "{user_name}" }
            span { class: "breadcrumb-sep", " > " }

            // ---------------- company selector ----------------
            div { class: "breadcrumb-selector",
                button {
                    class: "breadcrumb-button",
                    onclick: move |_| {
                        let next = !*company_open.read();
                        company_open.set(next);
                        branch_open.set(false);
                    },
                    "{company_label} ▾"
                }

                if *company_open.read() {
                    div { class: "breadcrumb-dropdown",
                        for (uuid, name) in companies.iter() {
                            {
                                let uuid = uuid.clone();
                                let name = name.clone();
                                let is_selected = selected_company.as_ref() == Some(&uuid);
                                rsx! {
                                    button {
                                        class: if is_selected { "active" } else { "" },
                                        onclick: move |_| {
                                            sender(Message::SelectCompany(uuid.clone()));
                                            company_open.set(false);
                                        },
                                        "{name}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            span { class: "breadcrumb-sep", " > " }

            // ---------------- branch selector ----------------
            div { class: "breadcrumb-selector",
                button {
                    class: "breadcrumb-button",
                    disabled: !has_company,
                    onclick: move |_| {
                        if !has_company {
                            return;
                        }
                        let next = !*branch_open.read();
                        branch_open.set(next);
                        company_open.set(false);
                    },
                    "{branch_label} ▾"
                }

                if *branch_open.read() && has_company {
                    div { class: "breadcrumb-dropdown",
                        for (uuid, name) in branches.iter() {
                            {
                                let uuid = uuid.clone();
                                let name = name.clone();
                                let is_selected = selected_branch.as_ref() == Some(&uuid);
                                rsx! {
                                    button {
                                        class: if is_selected { "active" } else { "" },
                                        onclick: move |_| {
                                            sender(Message::SelectBranch(uuid.clone()));
                                            branch_open.set(false);
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
