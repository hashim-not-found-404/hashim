use crate::accounting_client::client_domain::cache;
use crate::accounting_client::client_domain::cache_actor;
use crate::accounting_client::client_domain::client_traits;
use crate::accounting_client::client_domain::client_traits::ViewAndCache;
use crate::accounting_client::client_domain::commander;
use crate::accounting_client::client_domain::ui_model;
use crate::accounting_client::client_domain::ui_model::HashimSignal;
use crate::accounting_domain::cases;
use crate::accounting_domain::request_response;
use crate::accounting_domain::utility::resource_utils;
use crate::accounting_domain::utility::types;
use crate::utility::tools;
use crate::utility::traits;
use crate::utility::traits::JoinHandle;
use crate::utility::traits::Receiver;
use crate::utility::utils::ReadAndSet;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;

type Type1 = cases::list_company_and_branch::Input;
type Type2 = cases::list_company_and_branch::Input;
type Type3 = cases::list_company_and_branch::MyResult;
type Type4 = Result<types::ListOfCompanies, ()>;

impl tools::Sortable for types::Company {
    type Key = (String, types::UuidType);

    fn key(&self) -> Self::Key {
        (self.name.clone(), self.uuid.clone())
    }
}

impl tools::Sortable for types::Branch {
    type Key = (String, types::UuidType);

    fn key(&self) -> Self::Key {
        (self.name.clone(), self.uuid.clone())
    }
}

/// Sort a list of companies by name then by UUID, and sort branches inside each company similarly.
pub fn sort_companies(companies: &mut types::ListOfCompanies) {
    tools::sort(companies);
    for company in companies {
        tools::sort(&mut company.branches);
    }
}

struct Cache<Ch, LongCache>
where
    Ch: cache::Cache,
    LongCache: for<'a> cases::list_company_and_branch::DatabaseRead<Db<'a> = Ch>,
{
    _ph: PhantomData<(Ch, LongCache)>,
}

impl<Ch, LongCache> cases::list_company_and_branch::DatabaseRead for Cache<Ch, LongCache>
where
    Ch: cache::Cache,
    LongCache: for<'a> cases::list_company_and_branch::DatabaseRead<Db<'a> = Ch>,
{
    type Db<'a> = cache::State<Ch>;

    async fn read(
        db: &mut Self::Db<'_>,
        read_input: &cases::list_company_and_branch::ReadInput,
    ) -> Result<cases::list_company_and_branch::ReadOutput, traits::DynamicError> {
        let mut read_output = LongCache::read(&mut db.cache, read_input).await.unwrap();

        // Build a map for O(1) lookup and updates
        let mut company_map: HashMap<
            types::UuidType,
            cases::list_company_and_branch::AllCompaniesThatUserInWithRoles,
        > = read_output.data.into_iter().map(|c| (c.company_uuid.clone(), c)).collect();

        // Process pending access controls for this user
        for (_, acf) in &db.state_of_pending_txn.access_control_for_company {
            if acf.user_ != read_input.user_uuid {
                continue;
            }

            let company_uuid = acf.data_group.clone();

            // If the company exists in pending transaction, use its data
            if let Some(company) = db.state_of_pending_txn.company.get(&company_uuid) {
                // Get or create the company entry in the map
                let company_entry = company_map.entry(company_uuid.clone()).or_insert_with(|| {
                    cases::list_company_and_branch::AllCompaniesThatUserInWithRoles {
                        company_uuid:     company_uuid.clone(),
                        company_name:     company.name.clone(),
                        company_currancy: company.currency.clone(),
                        user_roles:       Vec::new(),
                        branches:         Vec::new(),
                    }
                });

                // Overwrite with pending data (pending is more recent)
                company_entry.company_name = company.name.clone();
                company_entry.company_currancy = company.currency.clone();

                // Add the role from this access control (avoid duplicates)
                if !company_entry.user_roles.contains(&acf.role) {
                    company_entry.user_roles.push(acf.role.clone());
                }

                // Add pending branches that belong to this company
                for (branch_uuid, branch) in &db.state_of_pending_txn.company_branch {
                    if branch.company_belong == company_uuid {
                        // Check if branch already exists (by UUID)
                        let exists =
                            company_entry.branches.iter().any(|b| b.branch_uuid == *branch_uuid);
                        if !exists {
                            company_entry.branches.push(
                                cases::list_company_and_branch::AllBranchesThatUserInWithRoles {
                                    branch_uuid:     branch_uuid.clone(),
                                    branch_name:     branch.name.clone(),
                                    branch_currancy: branch.currency.clone(),
                                    user_roles:      Vec::new(), // No branch roles in pending transaction
                                },
                            );
                        }
                    }
                }
            }
            // else: no pending company – this shouldn't happen if there's an access control,
            // but we ignore it to avoid incomplete data.
        }

        // Convert the map back into a vector
        read_output.data = company_map.into_values().collect();

        Ok(read_output)
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

    fn wrap_input(data: Self::Type1) -> request_response::push_data::OperationsInput {
        request_response::push_data::OperationsInput::ListCompanyAndBranch(data)
    }

    fn user_uuid(data: &Self::Type2) -> Option<&types::UuidType> {
        Some(&data.user_uuid)
    }

    async fn state_full_operation<Id: types::RowId>(
        data: &Self::Type2,
        state: &mut cache::State<Ch>,
    ) -> Self::Type3 {
        let result = data.state_full_operation::<Cache<Ch, LongCache>>(state).await.unwrap();

        Ok(result)
    }

    fn extract_resource(data: &Self::Type3) -> Vec<resource_utils::ResourceInfo> {
        match data {
            Ok(ok) => {
                let mut resources = Vec::new();
                let user_uuid = &ok.user_uuid;

                for company in &ok.data {
                    let company_uuid = &company.company_uuid;

                    // ---- Company fields ----
                    resources.push(resource_utils::ResourceInfo {
                        row_uuid: company_uuid.clone(),
                        resource: resource_utils::Resource::TableCompanyFieldName(
                            company.company_name.clone(),
                        ),
                    });
                    resources.push(resource_utils::ResourceInfo {
                        row_uuid: company_uuid.clone(),
                        resource: resource_utils::Resource::TableCompanyFieldCurrency(
                            company.company_currancy.clone(),
                        ),
                    });

                    // ---- Company access control ----
                    // One resource per role (multiple roles possible)
                    for role in &company.user_roles {
                        resources.push(resource_utils::ResourceInfo {
                            row_uuid: company_uuid.clone(),
                            resource:
                                resource_utils::Resource::TableAccessControlForCompanyFieldRole(
                                    role.clone(),
                                ),
                        });
                    }
                    // Always add the user and data_group (self) once per company
                    resources.push(resource_utils::ResourceInfo {
                        row_uuid: company_uuid.clone(),
                        resource: resource_utils::Resource::TableAccessControlForCompanyFieldUser(
                            user_uuid.clone(),
                        ),
                    });
                    resources.push(resource_utils::ResourceInfo {
                        row_uuid: company_uuid.clone(),
                        resource:
                            resource_utils::Resource::TableAccessControlForCompanyFieldDataGroup(
                                company_uuid.clone(),
                            ),
                    });

                    // ---- Branches ----
                    for branch in &company.branches {
                        let branch_uuid = &branch.branch_uuid;

                        resources.push(resource_utils::ResourceInfo {
                            row_uuid: branch_uuid.clone(),
                            resource: resource_utils::Resource::TableCompanyBranchFieldName(
                                branch.branch_name.clone(),
                            ),
                        });
                        resources.push(resource_utils::ResourceInfo {
                            row_uuid: branch_uuid.clone(),
                            resource: resource_utils::Resource::TableCompanyBranchFieldCurrency(
                                branch.branch_currancy.clone(),
                            ),
                        });
                        resources.push(resource_utils::ResourceInfo {
                            row_uuid: branch_uuid.clone(),
                            resource:
                                resource_utils::Resource::TableCompanyBranchFieldCompanyBelong(
                                    company_uuid.clone(),
                                ),
                        });

                        // Branch access control (roles)
                        for role in &branch.user_roles {
                            resources.push(resource_utils::ResourceInfo {
                                row_uuid: branch_uuid.clone(),
                                resource: resource_utils::Resource::TableAccessControlForCompanyBranchFieldRole(
                                    role.clone(),
                                ),
                            });
                        }
                        // Add user and data_group for each branch
                        resources.push(resource_utils::ResourceInfo {
                            row_uuid: branch_uuid.clone(),
                            resource: resource_utils::Resource::TableAccessControlForCompanyBranchFieldUser(
                                user_uuid.clone(),
                            ),
                        });
                        resources.push(resource_utils::ResourceInfo {
                            row_uuid: branch_uuid.clone(),
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

    fn unwrap_output(output: request_response::push_data::OperationsResult) -> Self::Type4 {
        if let request_response::push_data::OperationsResult::ListCompanyAndBranch(res) = output {
            match res {
                Ok(ok) => {
                    let mut companies = Vec::with_capacity(ok.data.len());

                    for company_entry in ok.data {
                        // Convert branches for this company
                        let branches = company_entry
                            .branches
                            .into_iter()
                            .map(|branch_entry| {
                                types::Branch {
                                    uuid: branch_entry.branch_uuid,
                                    name: branch_entry.branch_name,
                                }
                            })
                            .collect();

                        // Pick a single role (e.g., the first one, or highest privilege)
                        // If no role, provide a sensible default (adjust as needed)
                        let role = company_entry.user_roles.first().cloned().unwrap_or_default();

                        companies.push(types::Company {
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
    pub(crate) async fn update<
        Rn: traits::RandomNumber,
        Rt: traits::Runtime,
        Id: types::RowId,
        Mpsc: traits::MultiProducerSingleConsumer,
        Rg: traits::Regex,
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

                handle_list_company_and_branch::<Rn, Rt, Id, Mpsc, Rg, As, Ch, LongCache>(
                    model,
                    cache.clone(),
                    commander_local_state.clone(),
                )
                .await;

                let listener_aborter =
                    handle_list_company_and_branch_listener::<
                        Rn,
                        Rt,
                        Id,
                        Mpsc,
                        Rg,
                        As,
                        Ch,
                        LongCache,
                    >(model, cache, commander_local_state.clone());

                commander_local_state
                    .aborter_to_company_and_branch_listener
                    .set(Box::new(listener_aborter));
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
                            selected_company.set(None)
                        } else {
                            selected_company.set(Some(i))
                        }
                    }
                    None => selected_company.set(Some(i)),
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

fn handle_list_company_and_branch_listener<
    Rn: traits::RandomNumber,
    Rt: traits::Runtime,
    Id: types::RowId,
    Mpsc: traits::MultiProducerSingleConsumer,
    Rg: traits::Regex,
    As: ui_model::AllSignalTypes,
    Ch: cache::Cache,
    LongCache: for<'a> cases::list_company_and_branch::DatabaseRead<Db<'a> = Ch>,
>(
    model: &'static ui_model::Model<As>,
    mut cache: client_traits::CacheActorStruct<Mpsc>,
    _: Arc<commander::CommanderLocalState<Mpsc, As>>,
) -> impl FnOnce() {
    let component_id = Rn::generate() as u16;
    let mut cache1 = cache.clone();

    let mut handle = Rt::abortable_spawn_local(async move {
        let mut receiver_to_poke = cache
            .send_subs_to_cache_actor(
                component_id,
                <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::subs(),
            )
            .await;

        let data: types::UuidType = model.user_uuid.read().clone().unwrap();

        loop {
            if let Err(_) = receiver_to_poke.recv().await {
                break;
            }

            let txn_number = Rn::generate();

            let value = cache
                .send_to_cache_actor(
                    cache_actor::CachingStrategy::ReadCacheOnly,
                    txn_number,
                    <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::wrap_input(Type1 {
                        user_uuid: data.clone(),
                    }),
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
                } => <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::unwrap_output(data),
            };

            <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::apply_on_the_model(&value, model);

            if value.is_err() {
                break;
            }
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
    Ch: cache::Cache,
    LongCache: for<'a> cases::list_company_and_branch::DatabaseRead<Db<'a> = Ch>,
>(
    model: &'static ui_model::Model<As>,
    mut cache: client_traits::CacheActorStruct<Mpsc>,
    _: Arc<commander::CommanderLocalState<Mpsc, As>>,
) {
    let user_uuid = model.user_uuid.read().clone().unwrap();

    let txn_number = Rn::generate();

    let mut receiver_to_response = cache
        .send_to_cache_actor(
            cache_actor::CachingStrategy::ReadCacheAndServer,
            txn_number,
            <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::wrap_input(Type1 {
                user_uuid,
            }),
        )
        .await;

    loop {
        let value = match receiver_to_response.recv().await.unwrap() {
            cache_actor::Response::CloseTheChannel => break,
            cache_actor::Response::ServerCannotBeReached => break,
            cache_actor::Response::Data {
                is_response_from_server: _,
                data,
            } => <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::unwrap_output(data),
        };

        <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::apply_on_the_model(&value, model);

        if value.is_err() {
            break;
        }
    }
}
