use crate::client::fetches;
use crate::domain::DatabaseRead;
use crate::domain::Input;
use crate::domain::MyResult;
use kernel::cache::Cache;
use kernel::new_types::AccountUuid;
use kernel::new_types::CompanyUuid;
use kernel::new_types::UserUuid;
use kernel::new_types::UuidType;
use kernel::request_response::TypeOperationsInput;
use kernel::request_response::TypeOperationsResult;
use kernel::types::MyErrorTrait;
use utility::actors::MultiProducerSingleConsumer;
use utility::actors::Receiver;
use utility::actors::Sender;
use utility::cache::CacheStruct;
use utility::cache::CachingStrategy;
use utility::cache::Response;
use utility::process_manager::Dialog;
use utility::process_manager::MessageToProcessManager;
use utility::process_manager::ProcessId;
use utility::process_manager::UserConsent;
use utility::random_number::RandomNumber;
use utility::row_id::RowId;
use utility::runtime::Runtime;
use utility::types::MakeOptionIfEmpty;
use utility::types::ReadAndSet;
use utility::ui_orchestration::Subscribe;
use utility::ui_orchestration::handle_fall_back;
use utility_ui::domain::HashimSignal;

type Type1 = Input;
type Type2 = Input;
type Type3 = MyResult;
type Type4 = MyResult;

pub trait GlobalModel {
    fn user_uuid(&self) -> impl ReadAndSet<UserUuid>;
    fn selected_company(&self) -> impl ReadAndSet<CompanyUuid>;
}

pub trait LocalModel {
    fn process_id(&self) -> impl ReadAndSet<Option<ProcessId>>;
    fn show_dialog(&self) -> impl Dialog;
    fn is_loading(&self) -> impl HashimSignal<bool>;
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

fn apply_on_the_model<As: LocalModel>(output: &Type4, model: &As) {
    match output {
        Ok(_) => {
            model.account_name_error().reset();
        }
        Err(business_error) => {
            model
                .account_name_error()
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
        As: LocalModel,
        Ch: Cache,
        LongCache: for<'a> DatabaseRead<Db<'a> = Ch>,
    >(
        self,
        model: &'static As,
        cache: CacheStruct<Mpsc, Subscribe, TypeOperationsInput, TypeOperationsResult>,
        commander_local_state: CommanderLocalState<Mpsc, As>,
    ) {
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
            CreateAccount::IsDebit(v) => model.is_debit().set(v),
            CreateAccount::IsPermanentAccount(v) => model.is_permanent_account().set(v),
            CreateAccount::AccountName(v) => {
                model.account_name().set(v);
                handle_check::<Rn, Id, Mpsc, As, Ch, LongCache>(model, cache).await;
            }
            CreateAccount::Notes(v) => model.notes().set(v),
            CreateAccount::UnitOfMeasurementOfQuantity(v) => {
                model.unit_of_measurement_of_quantity().set(v)
            }
            CreateAccount::Subscribe => {
                fetches::get_all_accounts::fetch::<Rn, Mpsc, As>(model, cache).await
            }
        }
    }
}

fn build_input<Id: RowId>(global_model: &impl GlobalModel, local_model: &impl LocalModel) -> Type1 {
    Input {
        user_uuid:                       global_model.user_uuid().read(),
        new_uuid:                        AccountUuid::from(UuidType::from(Id::generate())),
        is_debit:                        local_model.is_debit().read(),
        is_permanent_account:            local_model.is_permanent_account().read(),
        account_name:                    local_model.account_name().read(),
        notes:                           local_model.notes().read().none_if_empty(),
        unit_of_measurement_of_quantity: local_model.unit_of_measurement_of_quantity().read(),
        belong_to_company:               global_model.selected_company().read(),
    }
}

fn handle_clean<As: LocalModel>(local_model: &As) {
    local_model.process_id().put(None);
    local_model.account_name().reset();
    local_model.is_debit().reset();
    local_model.is_permanent_account().reset();
    local_model.notes().reset();
    local_model.unit_of_measurement_of_quantity().reset();
    local_model.is_loading().reset();
    local_model.account_name_error().reset();
}

async fn handle_submit<
    Rn: RandomNumber,
    Rt: Runtime,
    Id: RowId,
    Mpsc: MultiProducerSingleConsumer,
    As: LocalModel,
    // Di:
    Ch: Cache,
    LongCache: for<'a> DatabaseRead<Db<'a> = Ch>,
>(
    global_model: &impl GlobalModel,
    local_model: &'static As,
    cache: CacheStruct<Mpsc, Subscribe, TypeOperationsInput, TypeOperationsResult>,
    commander_local_state: CommanderLocalState<Mpsc, As>,
) {
    let process_id = ProcessId::new();
    local_model.process_id().put(Some(process_id));

    let input = build_input::<Id>(global_model, local_model).into();

    handle_fall_back::<Rn, Rt, Mpsc, Di, TypeOperationsInput, TypeOperationsResult>(
        cache,
        commander_local_state,
        &local_model.show_dialog(),
        process_id,
        input,
        move |data| {
            data.downcast::<MyResult>().unwrap();
            let result = unwrap_output(data);
            apply_on_the_model(&result, local_model);

            let is_ok = result.is_ok();
            if is_ok {
                handle_clean(local_model);
            }

            is_ok
        },
    )
    .await;

    local_model.is_loading().reset();
}

async fn handle_check<
    Rn: RandomNumber,
    Id: RowId,
    Mpsc: MultiProducerSingleConsumer,
    As: LocalModel,
    Ch: Cache,
    LongCache: for<'a> DatabaseRead<Db<'a> = Ch>,
>(
    global_model: &impl GlobalModel,
    local_model: &'static As,
    mut cache: CacheStruct<Mpsc, Subscribe, TypeOperationsInput, TypeOperationsResult>,
) {
    let input = build_input::<Id>(global_model, local_model);

    let mut receiver_to_response =
        cache.send_to_cache_actor(CachingStrategy::ReadCacheOnly, Rn::generate(), input).await;

    match receiver_to_response.recv().await.unwrap() {
        Response::CloseTheChannel => {}
        Response::ServerCannotBeReached => {}
        Response::Data {
            is_response_from_server: _,
            data,
        } => {
            let result = unwrap_output(data);

            apply_on_the_model(&result, local_model);
        }
    }
}
