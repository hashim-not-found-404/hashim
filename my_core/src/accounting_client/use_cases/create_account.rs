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
use std::sync::Arc;

type Type1 = cases::create_account::Input;
type Type2 = cases::create_account::Input;
type Type3 = cases::create_account::MyResult;
type Type4 = cases::create_account::MyResult;

struct Cache<Ch, LongCache>
where
    Ch: cache::Cache,
    LongCache: for<'a> cases::create_account::DatabaseRead<Db<'a> = Ch>,
{
    _ph: PhantomData<(Ch, LongCache)>,
}

impl<Ch, LongCache> cases::create_account::DatabaseRead for Cache<Ch, LongCache>
where
    Ch: cache::Cache,
    LongCache: for<'a> cases::create_account::DatabaseRead<Db<'a> = Ch>,
{
    type Db<'a> = cache::State<Ch>;

    async fn read(
        db: &mut Self::Db<'_>,
        read_input: &cases::create_account::ReadInput,
    ) -> Result<cases::create_account::ReadOutput, traits::DynamicError> {
        let mut read_output = LongCache::read(&mut db.cache, read_input).await.unwrap();
        read_output.is_company_uuid_exist = true;
        read_output.is_new_uuid_used = false;

        for (_, table) in &db.state_of_pending_txn.account {
            if read_input.account_name == table.name
                && read_input.belong_to_company == table.company_belong
            {
                read_output.is_account_name_used = true;
            }
        }

        for (_, table) in &db.state_of_pending_txn.access_control_for_company {
            if read_input.user_uuid == table.user_ {
                read_output.user_roles.push(table.role.clone());
            }
        }

        Ok(read_output)
    }
}

pub(crate) struct ViewAndCacheType;

impl<Ch, LongCache> ViewAndCache<Ch, LongCache> for ViewAndCacheType
where
    Ch: cache::Cache,
    LongCache: for<'a> cases::create_account::DatabaseRead<Db<'a> = Ch>,
{
    type Type1 = Type1;
    type Type2 = Type2;
    type Type3 = Type3;
    type Type4 = Type4;

    fn wrap_input(data: Self::Type1) -> request_response::push_data::OperationsInput {
        request_response::push_data::OperationsInput::CreateAccount(data)
    }

    fn user_uuid(data: &Self::Type2) -> Option<&types::UuidType> {
        Some(&data.user_uuid)
    }

    async fn state_full_operation<Id: types::RowId>(
        data: &Self::Type2,
        state: &mut cache::State<Ch>,
    ) -> Self::Type3 {
        let errr = data.state_full_check::<Cache<Ch, LongCache>>(state).await.unwrap();

        if errr.is_there_error() {
            return Err(errr);
        }

        let ok = data.state_less_operation();

        return Ok(ok);
    }

    fn extract_resource(data: &Self::Type3) -> Vec<resource_utils::ResourceInfo> {
        match data {
            Ok(ok) => {
                let mut resources = Vec::new();

                resources.push(resource_utils::ResourceInfo {
                    row_uuid: ok.new_uuid.clone(),
                    resource: resource_utils::Resource::TableAccountFieldCompanyBelong(
                        ok.belong_to_company.clone(),
                    ),
                });

                resources.push(resource_utils::ResourceInfo {
                    row_uuid: ok.new_uuid.clone(),
                    resource: resource_utils::Resource::TableAccountFieldIsDebit(
                        ok.is_debit.clone(),
                    ),
                });

                resources.push(resource_utils::ResourceInfo {
                    row_uuid: ok.new_uuid.clone(),
                    resource: resource_utils::Resource::TableAccountFieldIsPermanentAccount(
                        ok.is_permanent_account.clone(),
                    ),
                });

                resources.push(resource_utils::ResourceInfo {
                    row_uuid: ok.new_uuid.clone(),
                    resource: resource_utils::Resource::TableAccountFieldName(
                        ok.account_name.clone(),
                    ),
                });

                resources.push(resource_utils::ResourceInfo {
                    row_uuid: ok.new_uuid.clone(),
                    resource: resource_utils::Resource::TableAccountFieldNotes(ok.notes.clone()),
                });

                resources.push(resource_utils::ResourceInfo {
                    row_uuid: ok.new_uuid.clone(),
                    resource:
                        resource_utils::Resource::TableAccountFieldUnitOfMeasurementOfQuantity(
                            ok.unit_of_measurement_of_quantity.clone(),
                        ),
                });

                resources
            }
            Err(_) => Vec::new(),
        }
    }

    fn unwrap_output(output: request_response::push_data::OperationsResult) -> Self::Type4 {
        if let request_response::push_data::OperationsResult::CreateAccount(result) = output {
            return result;
        }
        unreachable!("{:?}", output)
    }

    fn apply_on_the_model<As: ui_model::AllSignalTypes>(
        output: &Self::Type4,
        model: &ui_model::Model<As>,
    ) {
        let local_state = &model.page_create_account;

        match output {
            Ok(_) => {
                local_state.account_name_error.reset();
            }
            Err(business_error) => {
                local_state.account_name_error.set(match business_error.account_name {
                    Some(_) => Some(String::from("duplicated")),
                    None => None,
                });
            }
        }
    }
}

impl ui_model::CreateAccount {
    pub(crate) async fn update<
        Rn: traits::RandomNumber,
        Rt: traits::Runtime,
        Id: types::RowId,
        Mpsc: traits::MultiProducerSingleConsumer,
        Rg: traits::Regex,
        As: ui_model::AllSignalTypes,
        Ch: cache::Cache,
        LongCache: for<'a> cases::create_account::DatabaseRead<Db<'a> = Ch>,
    >(
        self,
        model: &'static ui_model::Model<As>,
        cache: client_traits::CacheActorStruct<Mpsc>,
        commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
    ) {
        let local_state = &model.page_create_account;

        match self {
            ui_model::CreateAccount::Submit => {
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
                        process_name: process_manager::ProcessName::CreateAccount,
                        consent:      i,
                    })
                    .await
                    .unwrap();
            }
            ui_model::CreateAccount::Clean => handle_clean::<As>(model),
            ui_model::CreateAccount::IsDebit(v) => local_state.is_debit.set(v),
            ui_model::CreateAccount::IsPermanentAccount(v) => {
                local_state.is_permanent_account.set(v)
            }
            ui_model::CreateAccount::AccountName(v) => {
                local_state.account_name.set(v);
                handle_check::<Rn, Rt, Id, Mpsc, Rg, As, Ch, LongCache>(
                    model,
                    cache,
                    commander_local_state,
                )
                .await;
            }
            ui_model::CreateAccount::Notes(v) => local_state.notes.set(v),
            ui_model::CreateAccount::UnitOfMeasurementOfQuantity(v) => {
                local_state.unit_of_measurement_of_quantity.set(v)
            }
        }
    }
}

fn handle_clean<As: ui_model::AllSignalTypes>(model: &ui_model::Model<As>) {
    let local_state = &model.page_create_account;

    local_state.account_name.reset();
    local_state.is_debit.reset();
    local_state.is_permanent_account.reset();
    local_state.notes.reset();
    local_state.unit_of_measurement_of_quantity.reset();
    local_state.is_loading.reset();
    local_state.account_name_error.reset();
}

async fn handle_submit<
    Rn: traits::RandomNumber,
    Rt: traits::Runtime,
    Id: types::RowId,
    Mpsc: traits::MultiProducerSingleConsumer,
    Rg: traits::Regex,
    As: ui_model::AllSignalTypes,
    Ch: cache::Cache,
    LongCache: for<'a> cases::create_account::DatabaseRead<Db<'a> = Ch>,
>(
    model: &'static ui_model::Model<As>,
    cache: client_traits::CacheActorStruct<Mpsc>,
    commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
) {
    let user_uuid = commander_local_state.user_uuid.read().clone().unwrap();

    let local_state = &model.page_create_account;

    let input = cases::create_account::Input {
        user_uuid,
        new_uuid: Id::generate(),
        is_debit: local_state.is_debit.read(),
        is_permanent_account: local_state.is_permanent_account.read(),
        account_name: local_state.account_name.read(),
        notes: local_state.notes.read(),
        unit_of_measurement_of_quantity: local_state.unit_of_measurement_of_quantity.read(),
        belong_to_company: model.selected_company.read().unwrap(),
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
                        handle_clean(model);
                    }

                    commander_local_state1
                        .sender_to_process_manager
                        .read()
                        .send(process_manager::MessageToProcessManager::FromProcess {
                            process_name: process_manager::ProcessName::CreateAccount,
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
                process_name: process_manager::ProcessName::CreateAccount,
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
                        let is_ok = result.is_ok();

                        <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::apply_on_the_model(
                            &result, model,
                        );

                        if is_ok {
                            handle_clean(model);
                        }
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
    LongCache: for<'a> cases::create_account::DatabaseRead<Db<'a> = Ch>,
>(
    model: &'static ui_model::Model<As>,
    mut cache: client_traits::CacheActorStruct<Mpsc>,
    commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
) {
    let local_state = &model.page_create_account;

    let user_uuid = commander_local_state.user_uuid.read().clone().unwrap();

    let input = cases::create_account::Input {
        user_uuid,
        new_uuid: Id::generate(),
        is_debit: local_state.is_debit.read(),
        is_permanent_account: local_state.is_permanent_account.read(),
        account_name: local_state.account_name.read(),
        notes: local_state.notes.read(),
        unit_of_measurement_of_quantity: local_state.unit_of_measurement_of_quantity.read(),
        belong_to_company: model.selected_company.read().unwrap(),
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
            let result = <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::unwrap_output(data);

            <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::apply_on_the_model(&result, model);
        }
    }
}
