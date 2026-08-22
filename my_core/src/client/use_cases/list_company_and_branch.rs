use crate::client::utility::cache;
use crate::client::utility::client_traits;
use crate::client::utility::client_traits::ViewAndCache;
use crate::client::utility::commander;
use crate::client::utility::ui_model;
use crate::client::utility::ui_model::HashimSignal;
use crate::domain::cases;
use crate::domain::request_response;
use crate::domain::utility::resource_utils;
use crate::domain::utility::types::Branch;
use crate::domain::utility::types::Company;
use crate::domain::utility::types::ListOfCompanies;
use crate::domain::utility::types::RowId;
use crate::domain::utility::uuid;
use crate::domain::utility::uuid::User;
use crate::utility::tools;
use crate::utility::traits;
use crate::utility::utils::ReadAndSet;
use std::sync::Arc;

type Type1 = cases::list_company_and_branch::Input;
type Type2 = cases::list_company_and_branch::Input;
type Type3 = cases::list_company_and_branch::MyResult;
type Type4 = Result<ListOfCompanies, ()>;

impl tools::Sortable for Company {
    type Key = (String, uuid::Company);

    fn key(&self) -> Self::Key {
        (self.name.clone(), self.uuid.clone())
    }
}

impl tools::Sortable for Branch {
    type Key = (String, uuid::Branch);

    fn key(&self) -> Self::Key {
        (self.name.clone(), self.uuid.clone())
    }
}

pub fn sort_companies(companies: &mut ListOfCompanies) {
    tools::sort(companies);
    for company in companies {
        tools::sort(&mut company.branches);
    }
}

pub(crate) struct ViewAndCacheType;

impl<Ch, LongCache> ViewAndCache<Ch, LongCache> for ViewAndCacheType
where
    Ch: cache::Cache,
    LongCache: for<'a> cases::list_company_and_branch::DatabaseRead<Db<'a> = Ch>,
{
    type Type1 = Type1;
    type Type2 = Type2;
    type Type3 = Type3;
    type Type4 = Type4;

    fn subs() -> &'static [resource_utils::Subscribe] {
        &[
            resource_utils::Subscribe::TableCompanyBranchFieldName,
            resource_utils::Subscribe::TableCompanyFieldName,
            resource_utils::Subscribe::TableAccessControlForCompanyFieldRole,
        ]
    }

    fn wrap_input(data: Self::Type1) -> request_response::OperationsInput {
        request_response::OperationsInput::ListCompanyAndBranch(data)
    }

    fn user_uuid(data: &Self::Type2) -> Option<&User> {
        Some(&data.user_uuid)
    }

    async fn state_full_operation<Id: RowId>(data: &Self::Type2, state: &mut Ch) -> Self::Type3 {
        let result = data.state_full_operation::<LongCache>(state).await.unwrap();

        Ok(result)
    }

    fn extract_resource(data: &Self::Type3) -> Vec<resource_utils::ResourceInfo> {
        match data {
            Ok(ok) => {
                let mut resources = Vec::new();
                let user_uuid = &ok.user_uuid;

                for company in &ok.data {
                    let company_uuid = &company.company_uuid;

                    resources.push(resource_utils::ResourceInfo {
                        row_uuid: company_uuid.0.clone(),
                        resource: resource_utils::Resource::TableCompanyFieldName(
                            company.company_name.clone(),
                        ),
                    });
                    resources.push(resource_utils::ResourceInfo {
                        row_uuid: company_uuid.0.clone(),
                        resource: resource_utils::Resource::TableCompanyFieldCurrency(
                            company.company_currancy.clone(),
                        ),
                    });

                    for role in &company.user_roles {
                        resources.push(resource_utils::ResourceInfo {
                            row_uuid: company_uuid.0.clone(),
                            resource:
                                resource_utils::Resource::TableAccessControlForCompanyFieldRole(
                                    role.clone(),
                                ),
                        });
                    }
                    resources.push(resource_utils::ResourceInfo {
                        row_uuid: company_uuid.0.clone(),
                        resource: resource_utils::Resource::TableAccessControlForCompanyFieldUser(
                            user_uuid.clone(),
                        ),
                    });
                    resources.push(resource_utils::ResourceInfo {
                        row_uuid: company_uuid.0.clone(),
                        resource:
                            resource_utils::Resource::TableAccessControlForCompanyFieldDataGroup(
                                company_uuid.clone(),
                            ),
                    });

                    for branch in &company.branches {
                        let branch_uuid = &branch.branch_uuid;

                        resources.push(resource_utils::ResourceInfo {
                            row_uuid: branch_uuid.0.clone(),
                            resource: resource_utils::Resource::TableCompanyBranchFieldName(
                                branch.branch_name.clone(),
                            ),
                        });
                        resources.push(resource_utils::ResourceInfo {
                            row_uuid: branch_uuid.0.clone(),
                            resource: resource_utils::Resource::TableCompanyBranchFieldCurrency(
                                branch.branch_currancy.clone(),
                            ),
                        });
                        resources.push(resource_utils::ResourceInfo {
                            row_uuid: branch_uuid.0.clone(),
                            resource:
                                resource_utils::Resource::TableCompanyBranchFieldCompanyBelong(
                                    company_uuid.clone(),
                                ),
                        });

                        for role in &branch.user_roles {
                            resources.push(resource_utils::ResourceInfo {
                                row_uuid: branch_uuid.0.clone(),
                                resource: resource_utils::Resource::TableAccessControlForCompanyBranchFieldRole(
                                    role.clone(),
                                ),
                            });
                        }
                        resources.push(resource_utils::ResourceInfo {
                            row_uuid: branch_uuid.0.clone(),
                            resource: resource_utils::Resource::TableAccessControlForCompanyBranchFieldUser(
                                user_uuid.clone(),
                            ),
                        });
                        resources.push(resource_utils::ResourceInfo {
                            row_uuid: branch_uuid.0.clone(),
                            resource: resource_utils::Resource::TableAccessControlForCompanyBranchFieldDataGroup(
                                branch_uuid.clone(),
                            ),
                        });
                    }
                }

                resources
            }
            Err(_) => Vec::new(),
        }
    }

    fn unwrap_output(output: request_response::OperationsResult) -> Self::Type4 {
        if let request_response::OperationsResult::ListCompanyAndBranch(res) = output {
            match res {
                Ok(ok) => {
                    let mut companies = Vec::with_capacity(ok.data.len());

                    for company_entry in ok.data {
                        let branches = company_entry
                            .branches
                            .into_iter()
                            .map(|branch_entry| {
                                Branch {
                                    uuid: branch_entry.branch_uuid,
                                    name: branch_entry.branch_name,
                                }
                            })
                            .collect();

                        let role = company_entry.user_roles.first().cloned().unwrap_or_default();

                        companies.push(Company {
                            uuid: company_entry.company_uuid,
                            name: company_entry.company_name,
                            role,
                            branches,
                        });
                    }

                    sort_companies(&mut companies);

                    Ok(companies)
                }
                Err(_) => Err(()),
            }
        } else {
            unreachable!("Expected ListCompanyAndBranch, got {:?}", output)
        }
    }

    fn apply_on_the_model<As: ui_model::AllSignalTypes>(
        output: &Self::Type4,
        model: &ui_model::Model<As>,
    ) {
        match &output {
            Ok(ok) => model.page_company_branch_selection.list.set(ok.clone()),
            Err(_) => {
                model.navigator.set(ui_model::Navigator::SignIn);
            }
        }
    }
}

impl ui_model::CompanyAndBranchSelection {
    pub(crate) fn update<
        Rn: traits::RandomNumber,
        Rt: traits::Runtime,
        Mpsc: traits::MultiProducerSingleConsumer,
        As: ui_model::AllSignalTypes,
        Ch: cache::Cache + 'static,
        LongCache: for<'a> cases::list_company_and_branch::DatabaseRead<Db<'a> = Ch> + 'static,
    >(
        self,
        model: &'static ui_model::Model<As>,
        cache: client_traits::CacheActorStruct<Mpsc>,
        commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
    ) {
        match self {
            Self::Subscribe => {
                model.navigator.set(ui_model::Navigator::ListCompanyAndBranch(
                    ui_model::ListCompanyAndBranch::None,
                ));

                spawn_listener::<Rn, Rt, Mpsc, As, Ch, LongCache>(
                    model,
                    cache,
                    commander_local_state,
                );
            }
            Self::UnSubscribe => {
                commander_local_state.aborter_to_company_and_branch_listener.abort();
            }
            Self::ShowCreateCompany => {
                model.navigator.set(ui_model::Navigator::ListCompanyAndBranch(
                    ui_model::ListCompanyAndBranch::CreateCompany,
                ));
            }
            Self::ShowCreateCompanyBranch => {
                model.navigator.set(ui_model::Navigator::ListCompanyAndBranch(
                    ui_model::ListCompanyAndBranch::CreateCompanyBranch,
                ));
            }
            Self::SelectedCompany(i) => {
                let selected_company = &model.selected_company;

                match selected_company.read() {
                    Some(old_one) => {
                        if old_one == i {
                            selected_company.put(None)
                        } else {
                            selected_company.put(Some(i))
                        }
                    }
                    None => selected_company.put(Some(i)),
                }
            }
            Self::SelectedCompanyBranch(i) => {
                model.selected_company_branch.put(Some(i));
                model.navigator.set(ui_model::Navigator::Home(ui_model::HomeNav {
                    show_menu:       false,
                    page_to_present: ui_model::Menu::Dashboard,
                }))
            }
        }
    }
}

fn spawn_listener<
    Rn: traits::RandomNumber,
    Rt: traits::Runtime,
    Mpsc: traits::MultiProducerSingleConsumer,
    As: ui_model::AllSignalTypes,
    Ch: cache::Cache,
    LongCache: for<'a> cases::list_company_and_branch::DatabaseRead<Db<'a> = Ch>,
>(
    model: &'static ui_model::Model<As>,
    cache: client_traits::CacheActorStruct<Mpsc>,
    commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
) {
    let data = <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::wrap_input(Type1 {
        user_uuid: model.user_uuid.read().clone().unwrap(),
    });

    let listener_aborter = client_traits::spawn_listener::<Rn, Rt, Mpsc>(
        cache,
        <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::subs(),
        data,
        move |data| {
            let data = <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::unwrap_output(data);
            <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::apply_on_the_model(&data, model);
        },
    );

    commander_local_state.aborter_to_company_and_branch_listener.set(Box::new(listener_aborter));
}
