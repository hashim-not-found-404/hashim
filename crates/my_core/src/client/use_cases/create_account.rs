use crate::client::fetches;
use crate::client::utility::cache::Cache;
use crate::client::utility::cache_actor::CacheStruct;
use crate::client::utility::cache_actor::CachingStrategy;
use crate::client::utility::cache_actor::Response;
use crate::client::utility::client_traits;
use crate::client::utility::client_traits::CacheActorStruct;
use crate::client::utility::commander::CommanderLocalState;
use crate::client::utility::process_manager::MessageToProcessManager;
use crate::client::utility::process_manager::ProcessName;
use crate::client::utility::ui_model::AllSignalTypes;
use crate::client::utility::ui_model::CreateAccount;
use crate::client::utility::ui_model::HashimSignal;
use crate::client::utility::ui_model::Model;
use crate::domain::request_response::OperationsInput;
use crate::domain::request_response::OperationsResult;
use crate::domain::use_cases::create_account::DatabaseRead;
use crate::domain::use_cases::create_account::Input;
use crate::domain::use_cases::create_account::MyResult;
use crate::domain::utility::resource_utils::Subscribe;
use crate::domain::utility::types::MyErrorTrait;
use crate::domain::utility::types::RowId;
use crate::domain::utility::uuid::Account;
use crate::domain::utility::uuid::User;
use crate::make_user_uuid;
use crate::make_wrap_unwrap;
use crate::utility::traits::MultiProducerSingleConsumer;
use crate::utility::traits::RandomNumber;
use crate::utility::traits::Receiver;
use crate::utility::traits::Runtime;
use crate::utility::traits::Sender;
use crate::utility::utils::MakeOptionIfEmpty;
use crate::utility::utils::ReadAndSet;
use std::sync::Arc;

type Type1 = Input;
type Type2 = Input;
type Type3 = MyResult;
type Type4 = MyResult;

make_wrap_unwrap!(create_account, CreateAccount);
make_user_uuid!(create_account);

pub(crate) async fn state_full_operation<
    Ch: Cache,
    LongCache: for<'a> DatabaseRead<Db<'a> = Ch>,
>(
    data: &Type2,
    state: &mut Ch,
) -> Type3 {
    let errr = data.state_full_check::<LongCache>(state).await.unwrap();

    if errr.is_there_error() {
        return Err(errr);
    }

    Ok(data.state_less_operation())
}

pub(crate) fn extract_resource(data: &Type3) -> Vec<resource_utils::ResourceInfo> {
    match data {
        Ok(ok) => {
            vec![
                resource_utils::ResourceInfo {
                    row_uuid: ok.new_uuid.0.clone(),
                    resource: resource_utils::Resource::TableAccountFieldCompanyBelong(
                        ok.belong_to_company.clone(),
                    ),
                },
                resource_utils::ResourceInfo {
                    row_uuid: ok.new_uuid.0.clone(),
                    resource: resource_utils::Resource::TableAccountFieldIsDebit(ok.is_debit),
                },
                resource_utils::ResourceInfo {
                    row_uuid: ok.new_uuid.0.clone(),
                    resource: resource_utils::Resource::TableAccountFieldIsPermanentAccount(
                        ok.is_permanent_account,
                    ),
                },
                resource_utils::ResourceInfo {
                    row_uuid: ok.new_uuid.0.clone(),
                    resource: resource_utils::Resource::TableAccountFieldName(
                        ok.account_name.clone(),
                    ),
                },
                resource_utils::ResourceInfo {
                    row_uuid: ok.new_uuid.0.clone(),
                    resource: resource_utils::Resource::TableAccountFieldNotes(ok.notes.clone()),
                },
                resource_utils::ResourceInfo {
                    row_uuid: ok.new_uuid.0.clone(),
                    resource:
                        resource_utils::Resource::TableAccountFieldUnitOfMeasurementOfQuantity(
                            ok.unit_of_measurement_of_quantity.clone(),
                        ),
                },
            ]
        }
        Err(_) => Vec::new(),
    }
}

fn apply_on_the_model<As: AllSignalTypes>(output: &Type4, model: &Model<As>) {
    let local_state = &model.page_create_account;

    match output {
        Ok(_) => {
            local_state.account_name_error.reset();
        }
        Err(business_error) => {
            local_state
                .account_name_error
                .set(business_error.account_name.as_ref().map(|_| String::from("duplicated")));
        }
    }
}

impl CreateAccount {
    pub(crate) async fn update<
        Rn: RandomNumber,
        Rt: Runtime,
        Id: RowId,
        Mpsc: MultiProducerSingleConsumer,
        As: AllSignalTypes,
        Ch: Cache,
        LongCache: for<'a> DatabaseRead<Db<'a> = Ch>,
    >(
        self,
        model: &'static Model<As>,
        cache: CacheStruct<Mpsc, Subscribe, OperationsInput, OperationsResult>,
        commander_local_state: Arc<CommanderLocalState<Mpsc, As>>,
    ) {
        let local_state = &model.page_create_account;

        match self {
            CreateAccount::Submit => {
                handle_submit::<Rn, Rt, Id, Mpsc, As, Ch, LongCache>(
                    model,
                    cache,
                    commander_local_state,
                )
                .await
            }
            CreateAccount::Consent(i) => {
                commander_local_state
                    .sender_to_process_manager
                    .read()
                    .send(MessageToProcessManager::FromUser {
                        process_name: ProcessName::CreateAccount,
                        consent:      i,
                    })
                    .await
                    .unwrap();
            }
            CreateAccount::Clean => handle_clean::<As>(model),
            CreateAccount::IsDebit(v) => local_state.is_debit.set(v),
            CreateAccount::IsPermanentAccount(v) => local_state.is_permanent_account.set(v),
            CreateAccount::AccountName(v) => {
                local_state.account_name.set(v);
                handle_check::<Rn, Id, Mpsc, As, Ch, LongCache>(model, cache).await;
            }
            CreateAccount::Notes(v) => local_state.notes.set(v),
            CreateAccount::UnitOfMeasurementOfQuantity(v) => {
                local_state.unit_of_measurement_of_quantity.set(v)
            }
            CreateAccount::Subscribe => {
                fetches::get_all_accounts::fetch::<Rn, Mpsc, As>(model, cache).await
            }
        }
    }
}

fn build_input<Id: RowId, As: AllSignalTypes>(model: &Model<As>) -> Type1 {
    let local_state = &model.page_create_account;

    Input {
        user_uuid:                       model.user_uuid.read().clone().unwrap(),
        new_uuid:                        Account(Id::generate()),
        is_debit:                        local_state.is_debit.read(),
        is_permanent_account:            local_state.is_permanent_account.read(),
        account_name:                    local_state.account_name.read(),
        notes:                           local_state.notes.read().none_if_empty(),
        unit_of_measurement_of_quantity: local_state.unit_of_measurement_of_quantity.read(),
        belong_to_company:               model.selected_company.read().unwrap(),
    }
}

fn handle_clean<As: AllSignalTypes>(model: &Model<As>) {
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
    Rn: RandomNumber,
    Rt: Runtime,
    Id: RowId,
    Mpsc: MultiProducerSingleConsumer,
    As: AllSignalTypes,
    Ch: Cache,
    LongCache: for<'a> DatabaseRead<Db<'a> = Ch>,
>(
    model: &'static Model<As>,
    cache: CacheActorStruct<Mpsc>,
    commander_local_state: Arc<CommanderLocalState<Mpsc, As>>,
) {
    let input = build_input::<Id, As>(model);
    let data = wrap_input(input);

    client_traits::handle_fall_back::<Rn, Rt, Mpsc, As>(
        cache,
        commander_local_state,
        &model.page_create_account.show_dialog,
        ProcessName::CreateAccount,
        data,
        move |data| {
            let result = unwrap_output(data);
            apply_on_the_model(&result, model);

            let is_ok = result.is_ok();
            if is_ok {
                handle_clean(model);
            }

            is_ok
        },
    )
    .await;

    model.page_create_account.is_loading.reset();
}

async fn handle_check<
    Rn: RandomNumber,
    Id: RowId,
    Mpsc: MultiProducerSingleConsumer,
    As: AllSignalTypes,
    Ch: Cache,
    LongCache: for<'a> DatabaseRead<Db<'a> = Ch>,
>(
    model: &'static Model<As>,
    mut cache: CacheActorStruct<Mpsc>,
) {
    let input = build_input::<Id, As>(model);
    let data = wrap_input(input);

    let mut receiver_to_response =
        cache.send_to_cache_actor(CachingStrategy::ReadCacheOnly, Rn::generate(), data).await;

    match receiver_to_response.recv().await.unwrap() {
        Response::CloseTheChannel => {}
        Response::ServerCannotBeReached => {}
        Response::Data {
            is_response_from_server: _,
            data,
        } => {
            let result = unwrap_output(data);

            apply_on_the_model(&result, model);
        }
    }
}
