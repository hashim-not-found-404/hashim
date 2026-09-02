use crate::client::fetches;
use crate::client::utility::cache::Cache;
use crate::domain::DatabaseRead;
use crate::domain::Input;
use crate::domain::MyResult;
use crate::make_wrap_unwrap;
use kernel::new_types::AccountUuid;
use kernel::types::MyErrorTrait;
use serde::Deserialize;
use serde::Serialize;
use utility::actors::MultiProducerSingleConsumer;
use utility::actors::Receiver;
use utility::actors::Sender;
use utility::cache::CacheStruct;
use utility::cache::CachingStrategy;
use utility::cache::Response;
use utility::process_manager::MessageToProcessManager;
use utility::process_manager::ProcessId as ProcessName;
use utility::process_manager::UserConsent;
use utility::random_number::RandomNumber;
use utility::row_id::RowId;
use utility::runtime::Runtime;
use utility::types::MakeOptionIfEmpty;
use utility::types::ReadAndSet;
use utility::ui_orchestration::OperationName;
use utility::ui_orchestration::handle_fall_back;

pub trait HashimSignal<T: Clone + Default>: Default {
    fn reset(&self) {
        self.set(T::default());
    }
    fn read(&self) -> T;
    fn set(&self, v: T);
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub enum Dialog {
    #[default]
    Hide,
    Show,
    Error,
}

type Type1 = Input;
type Type2 = Input;
type Type3 = MyResult;
type Type4 = MyResult;

make_wrap_unwrap!(create_account, CreateAccount);

pub trait Model {
    fn is_loading(&self) -> impl HashimSignal<bool>;
    fn show_dialog(&self) -> impl HashimSignal<Dialog>;
    fn is_debit(&self) -> impl HashimSignal<bool>;
    fn is_permanent_account(&self) -> impl HashimSignal<bool>;
    fn account_name(&self) -> impl HashimSignal<String>;
    fn notes(&self) -> impl HashimSignal<String>;
    fn unit_of_measurement_of_quantity(&self) -> impl HashimSignal<String>;
    fn account_name_error(&self) -> impl HashimSignal<Option<String>>;
}

#[derive(Debug)]
pub enum CreateAccount {
    Subscribe,
    Submit,
    Consent(UserConsent),
    Clean,
    IsDebit(bool),
    IsPermanentAccount(bool),
    AccountName(String),
    Notes(String),
    UnitOfMeasurementOfQuantity(String),
}

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

fn apply_on_the_model<As: Model>(output: &Type4, model: &As) {
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
        As: Model,
        OperationsInput,
        OperationsResult,
        Ch: Cache,
        LongCache: for<'a> DatabaseRead<Db<'a> = Ch>,
    >(
        self,
        model: &'static As,
        cache: CacheStruct<Mpsc, OperationName, OperationsInput, OperationsResult>,
        commander_local_state: CommanderLocalState<Mpsc, As>,
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
                        process_id: ProcessName::CreateAccount,
                        consent:    i,
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

fn build_input<Id: RowId, As: Model>(model: &As) -> Type1 {
    let local_state = &model.page_create_account;

    Input {
        user_uuid:                       model.user_uuid.read().clone().unwrap(),
        new_uuid:                        AccountUuid(Id::generate().into()),
        is_debit:                        local_state.is_debit.read(),
        is_permanent_account:            local_state.is_permanent_account.read(),
        account_name:                    local_state.account_name.read(),
        notes:                           local_state.notes.read().none_if_empty(),
        unit_of_measurement_of_quantity: local_state.unit_of_measurement_of_quantity.read(),
        belong_to_company:               model.selected_company.read().unwrap(),
    }
}

fn handle_clean<As: Model>(model: &As) {
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
    As: Model,
    Ch: Cache,
    LongCache: for<'a> DatabaseRead<Db<'a> = Ch>,
>(
    model: &'static As,
    cache: CacheActorStruct<Mpsc>,
    commander_local_state: CommanderLocalState<Mpsc, As>,
) {
    let input = build_input::<Id, As>(model);
    let data = wrap_input(input);

    handle_fall_back::<Rn, Rt, Mpsc, As>(
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
    As: Model,
    Ch: Cache,
    LongCache: for<'a> DatabaseRead<Db<'a> = Ch>,
>(
    model: &'static As,
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
