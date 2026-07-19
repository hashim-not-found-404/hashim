use crate::{
    accounting_client::use_cases::client_domain::{
        cache, cache_actor,
        client_traits::{
            self, CacheAndServerType1, CacheAndServerType2, Mvu, ViewType1, ViewType2,
        },
        commander,
        ui_model::{self, HashimSignal},
    },
    accounting_domain::{
        cases::{
            self,
            utility::{resource_utils, types},
        },
        request_response,
    },
    utility::{
        traits::{self, JoinHandle, Receiver},
        utils::ReadAndSet,
    },
};
use std::{cmp::Ordering, sync::Arc};

pub(crate) type Type1 = cases::list_company_and_branch::Input;
type Type2 = cases::list_company_and_branch::Input;
type Type3 = cases::list_company_and_branch::MyResult;
pub(crate) struct Type4(pub(crate) Result<types::ListOfCompanies, ()>);

/// Sort a list of companies by name then by UUID, and sort branches inside each company similarly.
pub fn sort_companies(companies: &mut types::ListOfCompanies) {
    companies.sort_by(|a, b| compare_by_name_then_uuid(&a.name, &a.uuid, &b.name, &b.uuid));
    for company in companies {
        company
            .branches
            .sort_by(|a, b| compare_by_name_then_uuid(&a.name, &a.uuid, &b.name, &b.uuid));
    }
}

/// Helper that compares two items by name (lexicographically) and, if equal, by UUID.
fn compare_by_name_then_uuid(
    name_a: &str,
    uuid_a: &types::UuidType,
    name_b: &str,
    uuid_b: &types::UuidType,
) -> Ordering {
    match name_a.cmp(name_b) {
        Ordering::Equal => uuid_a.cmp(uuid_b),
        other => other,
    }
}

impl Into<Vec<resource_utils::ResourceInfo>> for &cases::list_company_and_branch::Ok {
    fn into(self) -> Vec<resource_utils::ResourceInfo> {
        use resource_utils::{Resource, ResourceInfo};

        let mut resources = Vec::new();
        let user_uuid = &self.user_uuid;

        for company in &self.data {
            let company_uuid = &company.company_uuid;

            // ---- Company fields ----
            resources.push(ResourceInfo {
                row_uuid: company_uuid.clone(),
                resource: Resource::TableCompanyFieldName(company.company_name.clone()),
            });
            resources.push(ResourceInfo {
                row_uuid: company_uuid.clone(),
                resource: Resource::TableCompanyFieldCurrency(company.company_currancy.clone()),
            });

            // ---- Company access control ----
            // One resource per role (multiple roles possible)
            for role in &company.user_roles {
                resources.push(ResourceInfo {
                    row_uuid: company_uuid.clone(),
                    resource: Resource::TableAccessControlForCompanyFieldRole(role.clone()),
                });
            }
            // Always add the user and data_group (self) once per company
            resources.push(ResourceInfo {
                row_uuid: company_uuid.clone(),
                resource: Resource::TableAccessControlForCompanyFieldUser(user_uuid.clone()),
            });
            resources.push(ResourceInfo {
                row_uuid: company_uuid.clone(),
                resource: Resource::TableAccessControlForCompanyFieldDataGroup(
                    company_uuid.clone(),
                ),
            });

            // ---- Branches ----
            for branch in &company.branches {
                let branch_uuid = &branch.branch_uuid;

                resources.push(ResourceInfo {
                    row_uuid: branch_uuid.clone(),
                    resource: Resource::TableCompanyBranchFieldName(branch.branch_name.clone()),
                });
                resources.push(ResourceInfo {
                    row_uuid: branch_uuid.clone(),
                    resource: Resource::TableCompanyBranchFieldCurrency(
                        branch.branch_currancy.clone(),
                    ),
                });
                resources.push(ResourceInfo {
                    row_uuid: branch_uuid.clone(),
                    resource: Resource::TableCompanyBranchFieldCompanyBelong(company_uuid.clone()),
                });

                // Branch access control (roles)
                for role in &branch.user_roles {
                    resources.push(ResourceInfo {
                        row_uuid: branch_uuid.clone(),
                        resource: Resource::TableAccessControlForCompanyBranchFieldRole(
                            role.clone(),
                        ),
                    });
                }
                // Add user and data_group for each branch
                resources.push(ResourceInfo {
                    row_uuid: branch_uuid.clone(),
                    resource: Resource::TableAccessControlForCompanyBranchFieldUser(
                        user_uuid.clone(),
                    ),
                });
                resources.push(ResourceInfo {
                    row_uuid: branch_uuid.clone(),
                    resource: Resource::TableAccessControlForCompanyBranchFieldDataGroup(
                        branch_uuid.clone(),
                    ),
                });
            }
        }

        resources
    }
}

impl ViewType1 for Type1 {
    fn subs() -> &'static [resource_utils::Subscribe] {
        &[
            resource_utils::Subscribe::TableCompanyBranchFieldName,
            resource_utils::Subscribe::TableCompanyFieldName,
            resource_utils::Subscribe::TableAccessControlForCompanyFieldRole,
        ]
    }

    fn wrap_input(self) -> request_response::push_data::OperationsInput {
        request_response::push_data::OperationsInput::ListCompanyAndBranch(self)
    }
}

impl CacheAndServerType1 for Type2 {
    fn user_uuid(&self) -> Option<&types::UuidType> {
        Some(&self.user_uuid)
    }

    type Output = Type3;

    async fn state_full_operation<Id: types::RowId, Ch: cache::Cache>(
        &self,
        state: &mut cache::State<Ch>,
    ) -> Self::Output {
        let result = state.read_list_company_and_branch(&self.user_uuid).await;
        return Ok(cases::list_company_and_branch::Ok {
            user_uuid: self.user_uuid.clone(),
            data: result,
        });
    }
}

impl CacheAndServerType2 for Type3 {
    fn extract_resource(&self) -> Vec<resource_utils::ResourceInfo> {
        match self {
            Ok(ok) => ok.into(),
            Err(_) => Vec::new(),
        }
    }

    fn wrap_output(self) -> request_response::push_data::OperationsResult {
        request_response::push_data::OperationsResult::ListCompanyAndBranch(self)
    }
}

impl ViewType2 for Type4 {
    fn unwrap_output(result: request_response::push_data::OperationsResult) -> Self {
        if let request_response::push_data::OperationsResult::ListCompanyAndBranch(res) = result {
            match res {
                Ok(ok) => {
                    let mut companies = Vec::with_capacity(ok.data.len());

                    for company_entry in ok.data {
                        // Convert branches for this company
                        let branches = company_entry
                            .branches
                            .into_iter()
                            .map(|branch_entry| types::Branch {
                                uuid: branch_entry.branch_uuid,
                                name: branch_entry.branch_name,
                            })
                            .collect();

                        // Pick a single role (e.g., the first one, or highest privilege)
                        // If no role, provide a sensible default (adjust as needed)
                        let role = company_entry
                            .user_roles
                            .first()
                            .cloned()
                            .unwrap_or_default();

                        companies.push(types::Company {
                            uuid: company_entry.company_uuid,
                            name: company_entry.company_name,
                            role,
                            branches,
                        });
                    }

                    sort_companies(&mut companies);

                    Type4(Ok(companies))
                }
                Err(_) => Type4(Err(())),
            }
        } else {
            unreachable!("Expected ListCompanyAndBranch, got {:?}", result)
        }
    }
}

impl Mvu for ui_model::CompanyAndBranchSelection {
    async fn update<
        Rn: traits::RandomNumber,
        Rt: traits::Runtime,
        Id: types::RowId,
        Mpsc: traits::MultiProducerSingleConsumer,
        Rg: traits::Regex,
        As: ui_model::AllSignalTypes,
    >(
        self,
        model: &'static ui_model::Model<As>,
        cache: client_traits::CacheActorStruct<Mpsc>,
        commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
    ) {
        match self {
            Self::Subscribe => {
                model
                    .navigator
                    .set(ui_model::Navigator::CompanyBranchSelection(
                        ui_model::CompanyBranchSelection::None,
                    ));

                handle_list_company_and_branch::<Rn, Rt, Id, Mpsc, Rg, As>(
                    model,
                    cache.clone(),
                    commander_local_state.clone(),
                )
                .await;

                let listener_aborter =
                    handle_list_company_and_branch_listener::<Rn, Rt, Id, Mpsc, Rg, As>(
                        model,
                        cache,
                        commander_local_state.clone(),
                    );

                *commander_local_state
                    .aborter_to_company_and_branch_listener
                    .lock()
                    .unwrap() = Some(Box::new(listener_aborter));
            }
            Self::UnSubscribe => {
                let mut guard = commander_local_state
                    .aborter_to_company_and_branch_listener
                    .lock()
                    .unwrap();

                if let Some(f) = guard.take() {
                    f();
                }
            }
            Self::ShowCreateCompany => {
                model
                    .navigator
                    .set(ui_model::Navigator::CompanyBranchSelection(
                        ui_model::CompanyBranchSelection::CreateCompany,
                    ));
            }
            Self::ShowCreateCompanyBranch => {
                model
                    .navigator
                    .set(ui_model::Navigator::CompanyBranchSelection(
                        ui_model::CompanyBranchSelection::CreateCompanyBranch,
                    ));
            }
            Self::SelectedCompany(i) => {
                let selected_company = &model
                    .page_root
                    .page_after_auth
                    .page_company_branch_selection
                    .selected_company;

                match selected_company.read() {
                    Some(old_one) => {
                        if old_one == i {
                            selected_company.set(None)
                        } else {
                            selected_company.set(Some(i))
                        }
                    }
                    None => selected_company.set(Some(i)),
                }
            }
            Self::SelectedCompanyBranch(i) => {
                commander_local_state.selected_company_branch.put(Some(i));
                model
                    .navigator
                    .set(ui_model::Navigator::Home(ui_model::Menu::Dashboard));
            }
        }
    }
}

fn handle_list_company_and_branch_listener<
    Rn: traits::RandomNumber,
    Rt: traits::Runtime,
    Id: types::RowId,
    Mpsc: traits::MultiProducerSingleConsumer,
    Rg: traits::Regex,
    As: ui_model::AllSignalTypes,
>(
    model: &'static ui_model::Model<As>,
    mut cache: client_traits::CacheActorStruct<Mpsc>,
    commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
) -> impl FnOnce() {
    let component_id = Rn::generate() as u16;
    let mut cache1 = cache.clone();

    let mut handle = Rt::abortable_spawn_local(async move {
        let mut receiver_to_poke = cache
            .send_subs_to_cache_actor(component_id, Type1::subs())
            .await;

        let data: types::UuidType = commander_local_state.user_uuid.read().clone().unwrap();

        loop {
            receiver_to_poke.recv().await.unwrap();

            let value = cache
                .send_to_cache_actor(
                    cache_actor::CachingStrategy::ReadCacheOnly,
                    Type1 {
                        user_uuid: data.clone(),
                    }
                    .wrap_input(),
                )
                .await
                .recv()
                .await
                .unwrap();

            let value = match value {
                cache_actor::Response::CloseTheChannel => break,
                cache_actor::Response::ServerCannotBeReached => break,
                cache_actor::Response::Data {
                    is_response_from_server: _,
                    data,
                } => Type4::unwrap_output(data),
            };

            match value.0 {
                Ok(ok) => model
                    .page_root
                    .page_after_auth
                    .page_company_branch_selection
                    .list
                    .set(ok),
                Err(_) => {
                    model
                        .navigator
                        .set(ui_model::Navigator::Auth(ui_model::Auth::SignIn));
                    break;
                }
            };
        }

        cache.send_unsubs_to_cache_actor(component_id).await
    });

    move || {
        Rt::spawn_local(async move {
            handle.abort().await;
            cache1.send_unsubs_to_cache_actor(component_id).await;
        });
    }
}

async fn handle_list_company_and_branch<
    Rn: traits::RandomNumber,
    Rt: traits::Runtime,
    Id: types::RowId,
    Mpsc: traits::MultiProducerSingleConsumer,
    Rg: traits::Regex,
    As: ui_model::AllSignalTypes,
>(
    model: &'static ui_model::Model<As>,
    mut cache: client_traits::CacheActorStruct<Mpsc>,
    commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
) {
    let user_uuid = commander_local_state.user_uuid.read().clone().unwrap();

    let mut receiver_to_response = cache
        .send_to_cache_actor(
            cache_actor::CachingStrategy::ReadCacheAndServer,
            Type1 { user_uuid }.wrap_input(),
        )
        .await;

    loop {
        let value = match receiver_to_response.recv().await.unwrap() {
            cache_actor::Response::CloseTheChannel => break,
            cache_actor::Response::ServerCannotBeReached => break,
            cache_actor::Response::Data {
                is_response_from_server: _,
                data,
            } => Type4::unwrap_output(data),
        };

        match value.0 {
            Ok(ok) => model
                .page_root
                .page_after_auth
                .page_company_branch_selection
                .list
                .set(ok),
            Err(_) => {
                model
                    .navigator
                    .set(ui_model::Navigator::Auth(ui_model::Auth::SignIn));
                break;
            }
        };
    }
}
