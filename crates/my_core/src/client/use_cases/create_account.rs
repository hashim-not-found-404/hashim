use crate::client::fetches;
use crate::client::utility::cache::Cache;
use crate::client::utility::cache_actor;
use crate::client::utility::client_traits;
use crate::client::utility::client_traits::ViewAndCache;
use crate::client::utility::commander;
use crate::client::utility::process_manager;
use crate::client::utility::ui_model;
use crate::client::utility::ui_model::HashimSignal;
use crate::domain::use_cases;
use crate::domain::utility::resource_utils;
use crate::domain::utility::types::MyErrorTrait;
use crate::domain::utility::types::RowId;
use crate::domain::utility::uuid::Account;
use crate::domain::utility::uuid::User;
use crate::make_user_uuid;
use crate::make_wrap_unwrap;
use crate::utility::traits;
use crate::utility::traits::Receiver;
use crate::utility::traits::Sender;
use crate::utility::utils::MakeOptionIfEmpty;
use crate::utility::utils::ReadAndSet;
use std::sync::Arc;

type Type1 = use_cases::create_account::Input;
type Type2 = use_cases::create_account::Input;
type Type3 = use_cases::create_account::MyResult;
type Type4 = use_cases::create_account::MyResult;

make_wrap_unwrap!(create_account, CreateAccount);
make_user_uuid!(create_account);

pub(crate) struct ViewAndCacheType;

impl<Ch, LongCache> ViewAndCache<Ch, LongCache> for ViewAndCacheType
where
    Ch: Cache,
    LongCache: for<'a> use_cases::create_account::DatabaseRead<Db<'a> = Ch>,
{
    type Type1 = Type1;
    type Type2 = Type2;
    type Type3 = Type3;
    type Type4 = Type4;

    async fn state_full_operation<Id: RowId>(data: &Self::Type2, state: &mut Ch) -> Self::Type3 {
        let errr = data.state_full_check::<LongCache>(state).await.unwrap();

        if errr.is_there_error() {
            return Err(errr);
        }

        Ok(data.state_less_operation())
    }

    fn extract_resource(data: &Self::Type3) -> Vec<resource_utils::ResourceInfo> {
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
                        resource: resource_utils::Resource::TableAccountFieldNotes(
                            ok.notes.clone(),
                        ),
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
}

fn apply_on_the_model<As: ui_model::AllSignalTypes>(output: &Type4, model: &ui_model::Model<As>) {
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

impl ui_model::CreateAccount {
    pub(crate) async fn update<
        Rn: traits::RandomNumber,
        Rt: traits::Runtime,
        Id: RowId,
        Mpsc: traits::MultiProducerSingleConsumer,
        As: ui_model::AllSignalTypes,
        Ch: Cache,
        LongCache: for<'a> use_cases::create_account::DatabaseRead<Db<'a> = Ch>,
    >(
        self,
        model: &'static ui_model::Model<As>,
        cache: client_traits::CacheActorStruct<Mpsc>,
        commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
    ) {
        let local_state = &model.page_create_account;

        match self {
            ui_model::CreateAccount::Submit => {
                handle_submit::<Rn, Rt, Id, Mpsc, As, Ch, LongCache>(
                    model,
                    cache,
                    commander_local_state,
                )
                .await
            }
            ui_model::CreateAccount::Consent(i) => {
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
                handle_check::<Rn, Id, Mpsc, As, Ch, LongCache>(model, cache).await;
            }
            ui_model::CreateAccount::Notes(v) => local_state.notes.set(v),
            ui_model::CreateAccount::UnitOfMeasurementOfQuantity(v) => {
                local_state.unit_of_measurement_of_quantity.set(v)
            }
            ui_model::CreateAccount::Subscribe => {
                fetches::get_all_accounts::fetch::<Rn, Mpsc, As>(model, cache).await
            }
        }
    }
}

fn build_input<Id: RowId, As: ui_model::AllSignalTypes>(model: &ui_model::Model<As>) -> Type1 {
    let local_state = &model.page_create_account;

    use_cases::create_account::Input {
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
    Id: RowId,
    Mpsc: traits::MultiProducerSingleConsumer,
    As: ui_model::AllSignalTypes,
    Ch: Cache,
    LongCache: for<'a> use_cases::create_account::DatabaseRead<Db<'a> = Ch>,
>(
    model: &'static ui_model::Model<As>,
    cache: client_traits::CacheActorStruct<Mpsc>,
    commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
) {
    let input = build_input::<Id, As>(model);
    let data = wrap_input(input);

    client_traits::handle_fall_back::<Rn, Rt, Mpsc, As>(
        cache,
        commander_local_state,
        &model.page_create_account.show_dialog,
        process_manager::ProcessName::CreateAccount,
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
    Rn: traits::RandomNumber,
    Id: RowId,
    Mpsc: traits::MultiProducerSingleConsumer,
    As: ui_model::AllSignalTypes,
    Ch: Cache,
    LongCache: for<'a> use_cases::create_account::DatabaseRead<Db<'a> = Ch>,
>(
    model: &'static ui_model::Model<As>,
    mut cache: client_traits::CacheActorStruct<Mpsc>,
) {
    let input = build_input::<Id, As>(model);
    let data = wrap_input(input);

    let mut receiver_to_response = cache
        .send_to_cache_actor(cache_actor::CachingStrategy::ReadCacheOnly, Rn::generate(), data)
        .await;

    match receiver_to_response.recv().await.unwrap() {
        cache_actor::Response::CloseTheChannel => {}
        cache_actor::Response::ServerCannotBeReached => {}
        cache_actor::Response::Data {
            is_response_from_server: _,
            data,
        } => {
            let result = unwrap_output(data);

            apply_on_the_model(&result, model);
        }
    }
}
