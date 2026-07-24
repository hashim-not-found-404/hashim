use crate::accounting_client::client_domain::cache;
use crate::accounting_client::client_domain::cache_actor;
use crate::accounting_client::client_domain::client_traits;
use crate::accounting_client::client_domain::client_traits::ViewAndCache;
use crate::accounting_client::client_domain::commander;
use crate::accounting_client::client_domain::process_manager;
use crate::accounting_client::client_domain::ui_model;
use crate::accounting_client::client_domain::ui_model::HashimSignal;
use crate::accounting_domain::cases;
use crate::accounting_domain::request_response;
use crate::accounting_domain::utility::resource_utils;
use crate::accounting_domain::utility::types;
use crate::accounting_domain::utility::types::MyErrorTrait;
use crate::mbg;
use crate::utility::traits;
use crate::utility::traits::JoinHandle;
use crate::utility::traits::Receiver;
use crate::utility::traits::Sender;
use crate::utility::utils::ReadAndSet;
use std::marker::PhantomData;
use std::str::FromStr;
use std::sync::Arc;

type Type1 = cases::create_company_branch::Input;
type Type2 = cases::create_company_branch::Input;
type Type3 = cases::create_company_branch::MyResult;
type Type4 = cases::create_company_branch::MyResult;

struct Cache<Ch, LongCache>
where
    Ch: cache::Cache,
    LongCache: for<'a> cases::create_company_branch::DatabaseRead<Db<'a> = Ch>,
{
    _ph: PhantomData<(Ch, LongCache)>,
}

impl<Ch, LongCache> cases::create_company_branch::DatabaseRead for Cache<Ch, LongCache>
where
    Ch: cache::Cache,
    LongCache: for<'a> cases::create_company_branch::DatabaseRead<Db<'a> = Ch>,
{
    type Db<'a> = cache::State<Ch>;

    async fn read(
        db: &mut Self::Db<'_>,
        read_input: &cases::create_company_branch::ReadInput,
    ) -> Result<cases::create_company_branch::ReadOutput, traits::DynamicError> {
        let mut read_output = LongCache::read(&mut db.cache, read_input).await.unwrap();

        // 2. Check pending transactions (uncommitted changes)
        // Check pending company access control for roles
        for (_, acf) in &db.state_of_pending_txn.access_control_for_company {
            if acf.data_group == read_input.company_belong && acf.user_ == read_input.user_uuid {
                read_output.user_roles.push(acf.role.clone());
            }
        }

        // Check pending company existence
        if db.state_of_pending_txn.company.contains_key(&read_input.company_belong) {
            read_output.is_company_exist = true;
        }

        // Check pending branch name usage
        for (_, branch) in &db.state_of_pending_txn.company_branch {
            if branch.company_belong == read_input.company_belong
                && branch.name == *read_input.branch_name
            {
                read_output.is_branch_name_used = true;
                break;
            }
        }

        Ok(read_output)
    }
}

pub(crate) struct ViewAndCacheType;

impl<Ch, LongCache> ViewAndCache<Ch, LongCache> for ViewAndCacheType
where
    Ch: cache::Cache,
    LongCache: for<'a> cases::create_company_branch::DatabaseRead<Db<'a> = Ch>,
{
    type Type1 = Type1;
    type Type2 = Type2;
    type Type3 = Type3;
    type Type4 = Type4;

    fn wrap_input(data: Self::Type1) -> request_response::push_data::OperationsInput {
        request_response::push_data::OperationsInput::CreateCompanyBranch(data)
    }

    fn user_uuid(data: &Self::Type2) -> Option<&types::UuidType> {
        Some(&data.user_uuid)
    }

    async fn state_full_operation<Id: types::RowId>(
        data: &Self::Type2,
        state: &mut cache::State<Ch>,
    ) -> Self::Type3 {
        let errr = data.state_full_check::<Id, Cache<Ch, LongCache>>(state).await.unwrap();

        if errr.is_there_error() {
            return Err(errr);
        }

        let result = data.state_less_operation();

        return Ok(result);
    }

    fn extract_resource(data: &Self::Type3) -> Vec<resource_utils::ResourceInfo> {
        match data {
            Ok(ok) => {
                let this = ok;
                let branch_uuid = this.new_uuid.clone();
                vec![
                        // Branch fields
                        resource_utils::ResourceInfo {
                            row_uuid: branch_uuid.clone(),
                            resource: resource_utils::Resource::TableCompanyBranchFieldName(
                                this.branch_name.clone(),
                            ),
                        },
                        resource_utils::ResourceInfo {
                            row_uuid: branch_uuid.clone(),
                            resource: resource_utils::Resource::TableCompanyBranchFieldCompanyBelong(
                                this.company_belong.clone(),
                            ),
                        },
                        resource_utils::ResourceInfo {
                            row_uuid: branch_uuid.clone(),
                            resource: resource_utils::Resource::TableCompanyBranchFieldLocation(
                                this.location.clone(),
                            ),
                        },
                        resource_utils::ResourceInfo {
                            row_uuid: branch_uuid.clone(),
                            resource: resource_utils::Resource::TableCompanyBranchFieldCurrency(
                                this.currency.clone(),
                            ),
                        },
                        // Access control for this branch (row_uuid is the branch UUID)
                        resource_utils::ResourceInfo {
                            row_uuid: branch_uuid.clone(),
                            resource: resource_utils::Resource::TableAccessControlForCompanyBranchFieldRole(
                                this.role.clone(),
                            ),
                        },
                        resource_utils::ResourceInfo {
                            row_uuid: branch_uuid.clone(),
                            resource: resource_utils::Resource::TableAccessControlForCompanyBranchFieldUser(
                                this.user_uuid.clone(),
                            ),
                        },
                        resource_utils::ResourceInfo {
                            row_uuid: branch_uuid,
                            resource:
                                resource_utils::Resource::TableAccessControlForCompanyBranchFieldDataGroup(
                                    this.new_uuid.clone(),
                                ),
                        },
                    ]
            }
            Err(_) => Vec::new(),
        }
    }

    fn unwrap_output(output: request_response::push_data::OperationsResult) -> Self::Type4 {
        if let request_response::push_data::OperationsResult::CreateCompanyBranch(result) = output {
            return result;
        }
        unreachable!("{:?}", output)
    }

    fn apply_on_the_model<As: ui_model::AllSignalTypes>(
        output: &Self::Type4,
        model: &ui_model::Model<As>,
    ) {
        let local_state = &model.page_create_company_branch;

        match output {
            Ok(_) => {
                local_state.branch_name_error.reset();
                local_state.location_error.reset();
            }
            Err(business_error) => {
                mbg!(business_error);
                local_state.branch_name_error.set(todo!());
                local_state.location_error.set(todo!());
            }
        }
    }
}

impl ui_model::CreateCompanyBranch {
    pub(crate) async fn update<
        Rn: traits::RandomNumber,
        Rt: traits::Runtime,
        Id: types::RowId,
        Mpsc: traits::MultiProducerSingleConsumer,
        Rg: traits::Regex,
        As: ui_model::AllSignalTypes,
        Ch: cache::Cache,
        LongCache: for<'a> cases::create_company_branch::DatabaseRead<Db<'a> = Ch>,
    >(
        self,
        model: &'static ui_model::Model<As>,
        cache: client_traits::CacheActorStruct<Mpsc>,
        commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
    ) {
        match self {
            Self::Submit => {
                handle_submit::<Rn, Rt, Id, Mpsc, Rg, As, Ch, LongCache>(
                    model,
                    cache,
                    commander_local_state,
                )
                .await
            }
            Self::Consent(i) => {
                commander_local_state
                    .sender_to_process_manager
                    .read()
                    .send(process_manager::MessageToProcessManager::FromUser {
                        process_name: process_manager::ProcessName::CreateCompanyBranch,
                        consent:      i,
                    })
                    .await
                    .unwrap();
            }
            Self::Close => handle_close::<As>(model),
            Self::Name(i) => {
                model.page_create_company_branch.branch_name.set(i);

                handle_check::<Rn, Rt, Id, Mpsc, Rg, As, Ch, LongCache>(
                    model,
                    cache,
                    commander_local_state,
                )
                .await;
            }
            Self::Currency(i) => {
                model
                    .page_create_company_branch
                    .currency
                    .set(types::Currency::from_str(i.as_str()).unwrap())
            }
        }
    }
}

async fn handle_submit<
    Rn: traits::RandomNumber,
    Rt: traits::Runtime,
    Id: types::RowId,
    Mpsc: traits::MultiProducerSingleConsumer,
    Rg: traits::Regex,
    As: ui_model::AllSignalTypes,
    Ch: cache::Cache,
    LongCache: for<'a> cases::create_company_branch::DatabaseRead<Db<'a> = Ch>,
>(
    model: &'static ui_model::Model<As>,
    cache: client_traits::CacheActorStruct<Mpsc>,
    commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
) {
    let local_state = &model.page_create_company_branch;

    if local_state.is_loading.read() == true {
        return;
    }
    local_state.is_loading.set(true);

    let data = commander_local_state.user_uuid.read().clone().unwrap();

    let input = cases::create_company_branch::Input {
        user_uuid:      data,
        new_uuid:       Id::generate(),
        company_belong: model.selected_company.read().unwrap(),
        currency:       local_state.currency.read(),
        branch_name:    local_state.branch_name.read(),
        location:       local_state.location.read(),
    };

    let data = <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::wrap_input(input);
    let txn_number = Rn::generate();

    {
        let dialog: &'static As::Dialog = &local_state.show_dialog;
        let mut cache = cache;
        let data1 = data.clone();
        let mut cache1 = cache.clone();
        let commander_local_state1 = commander_local_state.clone();
        let mut handle = <Rt>::abortable_spawn_local(async move {
            let mut receiver_to_response = cache1
                .send_to_cache_actor(
                    cache_actor::CachingStrategy::WriteServerOnly,
                    txn_number,
                    data1,
                )
                .await;

            match receiver_to_response.recv().await.unwrap() {
                cache_actor::Response::CloseTheChannel => return,
                cache_actor::Response::ServerCannotBeReached => return,
                cache_actor::Response::Data {
                    is_response_from_server,
                    data,
                } => {
                    let result =
                        <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::unwrap_output(data);
                    let is_ok = result.is_ok();
                    <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::apply_on_the_model(
                        &result, model,
                    );

                    if is_ok {
                        handle_close::<As>(model);
                    }

                    commander_local_state1
                        .sender_to_process_manager
                        .read()
                        .send(process_manager::MessageToProcessManager::FromProcess {
                            process_name: process_manager::ProcessName::CreateCompanyBranch,
                            message:      process_manager::MessageFromProcess::Response {
                                is_response_from_server,
                                is_response_ok: is_ok,
                            },
                        })
                        .await
                        .unwrap();
                }
            }
        });
        let (sender_to_process, mut receiver_to_process) = <Mpsc>::channel();
        commander_local_state
            .sender_to_process_manager
            .read()
            .send(process_manager::MessageToProcessManager::FromProcess {
                process_name: process_manager::ProcessName::CreateCompanyBranch,
                message:      process_manager::MessageFromProcess::Subscribe {
                    sender: sender_to_process,
                    dialog: &dialog,
                },
            })
            .await
            .unwrap();
        match receiver_to_process.recv().await.unwrap() {
            process_manager::MessageToProcess::FallBackToCache => {
                let mut receiver_to_response = cache
                    .send_to_cache_actor(
                        cache_actor::CachingStrategy::WriteCacheOnly,
                        txn_number,
                        data,
                    )
                    .await;

                match receiver_to_response.recv().await.unwrap() {
                    cache_actor::Response::CloseTheChannel => return,
                    cache_actor::Response::ServerCannotBeReached => return,
                    cache_actor::Response::Data {
                        is_response_from_server: _,
                        data,
                    } => {
                        let result =
                            <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::unwrap_output(data);
                        <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::apply_on_the_model(
                            &result, model,
                        );
                    }
                }
            }
            process_manager::MessageToProcess::CancelOperation => {}
        }
        handle.abort().await;
    }

    local_state.is_loading.reset();
}

async fn handle_check<
    Rn: traits::RandomNumber,
    Rt: traits::Runtime,
    Id: types::RowId,
    Mpsc: traits::MultiProducerSingleConsumer,
    Rg: traits::Regex,
    As: ui_model::AllSignalTypes,
    Ch: cache::Cache,
    LongCache: for<'a> cases::create_company_branch::DatabaseRead<Db<'a> = Ch>,
>(
    model: &'static ui_model::Model<As>,
    mut cache: client_traits::CacheActorStruct<Mpsc>,
    commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
) {
    let local_state = &model.page_create_company_branch;

    let data = commander_local_state.user_uuid.read().clone().unwrap();

    let input = cases::create_company_branch::Input {
        user_uuid:      data,
        new_uuid:       Id::generate(),
        company_belong: model.selected_company.read().unwrap(),
        currency:       local_state.currency.read(),
        branch_name:    local_state.branch_name.read(),
        location:       local_state.location.read(),
    };

    let txn_number = Rn::generate();

    let mut receiver_to_response = cache
        .send_to_cache_actor(
            cache_actor::CachingStrategy::ReadCacheOnly,
            txn_number,
            <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::wrap_input(input),
        )
        .await;

    match receiver_to_response.recv().await.unwrap() {
        cache_actor::Response::CloseTheChannel => {}
        cache_actor::Response::ServerCannotBeReached => {}
        cache_actor::Response::Data {
            is_response_from_server: _,
            data,
        } => {
            let result: Type4 =
                <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::unwrap_output(data);

            <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::apply_on_the_model(&result, model);
        }
    }
}

fn handle_close<As: ui_model::AllSignalTypes>(model: &'static ui_model::Model<As>) {
    let page_create_company_branch = &model.page_create_company_branch;

    if page_create_company_branch.show_dialog.read() == ui_model::Dialog::Show {
        return;
    }

    if page_create_company_branch.is_loading.read() {
        return;
    }

    page_create_company_branch.branch_name.reset();
    page_create_company_branch.currency.reset();
    page_create_company_branch.location.reset();

    model
        .navigator
        .set(ui_model::Navigator::ListCompanyAndBranch(ui_model::ListCompanyAndBranch::None));
}
