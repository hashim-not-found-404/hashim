use crate::use_cases::create_company;
use crate::use_cases::create_company_branch;
use crate::utility::tools;
use dioxus::prelude::*;
use my_core::client::utility::ui_model;
use my_core::client::utility::ui_model::HashimSignal;

#[component]
pub(crate) fn GetCompaniesAndBranches() -> Element {
    use_effect(move || {
        tools::send(ui_model::Message::CompanyAndBranchSelection(
            ui_model::CompanyAndBranchSelection::Subscribe,
        ));
    });

    use_drop(move || {
        tools::send(ui_model::Message::CompanyAndBranchSelection(
            ui_model::CompanyAndBranchSelection::UnSubscribe,
        ));
    });

    let local_state = &tools::MODEL.page_company_branch_selection.list;

    // let selected_company = &tools::MODEL.selected_company;

    rsx! {
        div {
            match tools::MODEL.navigator.read() {
                ui_model::Navigator::GetCompaniesAndBranches(n) => {
                    match n {
                        ui_model::GetCompaniesAndBranches::None => rsx! {},
                        ui_model::GetCompaniesAndBranches::CreateCompany => rsx! {
                            create_company::CreateCompany {}
                        },
                        ui_model::GetCompaniesAndBranches::CreateCompanyBranch => {
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

                            // if selected_company.read() == Some(company.uuid.clone()) {
                            //     button {
                            //         onclick: move |_| {
                            //             tools::send(
                            //                 ui_model::Message::CompanyAndBranchSelection(
                            //                     ui_model::CompanyAndBranchSelection::ShowCreateCompanyBranch,
                            //                 ),
                            //             )
                            //         },
                            //         "Add New Branch"
                            //     }
                            //     div {
                            //         for branch in company.branches {
                            //             {
                            //                 rsx! {
                            //                     button {
                            //                         onclick: {
                            //                             move |_| {
                            //                                 tools::send(
                            //                                     ui_model::Message::CompanyAndBranchSelection(
                            //                                         ui_model::CompanyAndBranchSelection::SelectedCompanyBranch(
                            //                                             branch.uuid.clone(),
                            //                                         ),
                            //                                     ),
                            //                                 )
                            //                             }
                            //                         },
                            //                         "{branch.name}"
                            //                     }
                            //                 }
                            //             }
                            //         }
                            //     }
                            // }
                        }
                    }
                }
            }
        }
    }
}
