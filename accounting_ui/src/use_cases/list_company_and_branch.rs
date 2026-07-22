use crate::{
    use_cases::{create_company, create_company_branch},
    utils::tools,
};
use dioxus::prelude::*;
use my_core::accounting_client::use_cases::client_domain::ui_model::{self, HashimSignal};

#[component]
pub(crate) fn ListCompanyAndBranch() -> Element {
    let local_state = &tools::MODEL
        .page_root
        .page_after_auth
        .page_company_branch_selection
        .list;

    let selected_company = &tools::MODEL
        .page_root
        .page_after_auth
        .page_company_branch_selection
        .selected_company;

    rsx! {
        div {
            match tools::MODEL.navigator.read() {
                ui_model::Navigator::CompanyBranchSelection(n) => {
                    match n {
                        ui_model::CompanyBranchSelection::None => rsx! {},
                        ui_model::CompanyBranchSelection::CreateCompany => rsx! {
                            create_company::CreateCompany {}
                        },
                        ui_model::CompanyBranchSelection::CreateCompanyBranch => {
                            rsx! {
                                create_company_branch::CreateCompanyBranch {}
                            }
                        }
                    }
                }
                _ => rsx! {},
            }

            button {
                onclick: move |_| {
                    tools::send(
                        ui_model::Message::CompanyAndBranchSelection(
                            ui_model::CompanyAndBranchSelection::ShowCreateCompany,
                        ),
                    )
                },
                "Add New Company"
            }

            div {
                for company in local_state.read() {
                    {
                        rsx! {
                            button {
                                onclick: move |_| {
                                    tools::send(
                                        ui_model::Message::CompanyAndBranchSelection(
                                            ui_model::CompanyAndBranchSelection::SelectedCompany(
                                                company.uuid.clone(),
                                            ),
                                        ),
                                    );
                                },
                                "{company.name}"
                            }

                            if selected_company.read() == Some(company.uuid.clone()) {
                                button {
                                    onclick: move |_| {
                                        tools::send(
                                            ui_model::Message::CompanyAndBranchSelection(
                                                ui_model::CompanyAndBranchSelection::ShowCreateCompanyBranch,
                                            ),
                                        )
                                    },
                                    "Add New Branch"
                                }
                                div {
                                    for branch in company.branches {
                                        {
                                            rsx! {
                                                button {
                                                    onclick: {
                                                        move |_| {
                                                            tools::send(
                                                                ui_model::Message::CompanyAndBranchSelection(
                                                                    ui_model::CompanyAndBranchSelection::SelectedCompanyBranch(
                                                                        branch.uuid.clone(),
                                                                    ),
                                                                ),
                                                            )
                                                        }
                                                    },
                                                    "{branch.name}"
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
    }
}
