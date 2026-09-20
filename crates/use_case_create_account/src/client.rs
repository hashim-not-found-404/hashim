use crate::domain::Error;
use crate::domain::Input;
use crate::domain::MyResult;
use crate::domain::Ok;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use infrastructure::actors::MpscSender;
use infrastructure::actors::Receiver;
use infrastructure::actors::Sender;
use infrastructure::row_id::Id;
use infrastructure::row_id::RowId;
use kernel::client::Cache;
use kernel::client::DialogSignalAdapter;
use kernel::new_types::AccountUuid;
use kernel::new_types::CompanyUuid;
use kernel::new_types::UserUuid;
use kernel::new_types::UuidType;
use kernel::types::DatabaseRead;
use kernel::types::MyErrorTrait;
use std::any::Any;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;
use use_case_get_all_accounts::client::fetch;
use utility::cache::CacheStruct;
use utility::cache::CachingStrategy;
use utility::cache::Response;
use utility::cache::Subscribe;
use utility::cache::TraitOperationClientError;
use utility::cache::TraitOperationClientInput;
use utility::cache::TraitOperationClientOk;
use utility::cache::TypeOperationClientError;
use utility::cache::TypeOperationClientInput;
use utility::cache::TypeOperationClientOk;
use utility::cache::TypeOperationClientResult;
use utility::dtos::TxnNumber;
use utility::process_manager::MessageToProcessManager;
use utility::process_manager::ProcessId;
use utility::process_manager::UserConsent;
use utility::types::MakeOptionIfEmpty;
use utility::ui_effect::MessageTrait;
use utility::ui_orchestration::handle_fall_back;
use utility_ui::domain::Dialog;
use utility_ui::domain::HashimSignal;

impl TraitOperationClientOk for Ok {
    fn subs_to_poke(&self) -> &'static [Subscribe] {
        todo!()
    }
}

impl TraitOperationClientError for Error {
    fn subs_to_poke(&self) -> &'static [Subscribe] {
        todo!()
    }
}

impl TraitOperationClientInput for Input {
    fn user_uuid(&self) -> Option<[u8; 16]> {
        Some(*self.user_uuid.deref().deref())
    }
}

pub async fn check_input<
    Ch: Cache,
    DBReader: for<'a> DatabaseRead<Db<'a> = Ch, Input = ReadInput, Output = ReadOutput>,
>(
    input: &Input,
    cache: &mut Ch,
) -> TypeOperationClientResult {
    let errr = input.state_full_check::<DBReader>(cache).await.unwrap();

    if errr.is_there_error() {
        return Err(TypeOperationClientError(Box::new(errr)));
    }

    let state_less_operation = input.state_less_operation();
    Ok(TypeOperationClientOk(Box::new(state_less_operation)))
}

#[derive(Debug)]
pub enum Message {
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

impl MessageTrait for Message {}

type Type1 = Input;
type Type2 = Input;
type Type3 = MyResult;
type Type4 = MyResult;

pub trait GlobalModel: 'static {
    fn user_uuid(&self) -> impl HashimSignal<Option<UserUuid>>;
    fn selected_company(&self) -> impl HashimSignal<Option<CompanyUuid>>;
}

pub trait LocalModel: 'static {
    fn process_id(&self) -> impl HashimSignal<Option<ProcessId>>;
    fn show_dialog(&self) -> impl HashimSignal<Dialog>;
    fn is_loading(&self) -> impl HashimSignal<bool>;
    fn is_debit(&self) -> impl HashimSignal<bool>;
    fn is_permanent_account(&self) -> impl HashimSignal<bool>;
    fn account_name(&self) -> impl HashimSignal<String>;
    fn notes(&self) -> impl HashimSignal<String>;
    fn unit_of_measurement_of_quantity(&self) -> impl HashimSignal<String>;
    fn account_name_error(&self) -> impl HashimSignal<Option<String>>;
}

fn apply_on_the_model(output: &Type4, local_model: Arc<impl LocalModel>) {
    match output {
        Ok(_) => {
            local_model.account_name_error().reset();
        }
        Err(business_error) => {
            local_model
                .account_name_error()
                .set(business_error.account_name.as_ref().map(|_| String::from("duplicated")));
        }
    }
}

impl Message {
    pub async fn update_generic(
        self,
        global_model: Arc<impl GlobalModel>,
        local_model: Arc<impl LocalModel>,
        cache: CacheStruct,
        mut sender_to_process_manager: MpscSender<MessageToProcessManager>,
    ) {
        match self {
            Self::Submit => {
                handle_submit(global_model, local_model, cache, sender_to_process_manager).await
            }
            Self::Consent(i) => {
                sender_to_process_manager
                    .send(MessageToProcessManager::FromUser {
                        process_id: local_model.process_id().read().unwrap(),
                        consent:    i,
                    })
                    .await
                    .unwrap();
            }
            Self::Clean => handle_clean(local_model),
            Self::IsDebit(v) => local_model.is_debit().set(v),
            Self::IsPermanentAccount(v) => local_model.is_permanent_account().set(v),
            Self::AccountName(v) => {
                local_model.account_name().set(v);
                handle_check(global_model, local_model, cache).await;
            }
            Self::Notes(v) => local_model.notes().set(v),
            Self::UnitOfMeasurementOfQuantity(v) => {
                local_model.unit_of_measurement_of_quantity().set(v)
            }
            Self::Subscribe => {
                fetch(
                    global_model.selected_company().read().unwrap(),
                    global_model.user_uuid().read().unwrap(),
                    cache,
                )
                .await
            }
        }
    }
}

fn build_input(global_model: Arc<impl GlobalModel>, local_model: Arc<impl LocalModel>) -> Type1 {
    Input {
        user_uuid:                       global_model.user_uuid().read().unwrap(),
        new_uuid:                        AccountUuid::from(UuidType::from(Id::generate())),
        is_debit:                        local_model.is_debit().read(),
        is_permanent_account:            local_model.is_permanent_account().read(),
        account_name:                    local_model.account_name().read(),
        notes:                           local_model.notes().read().none_if_empty(),
        unit_of_measurement_of_quantity: local_model.unit_of_measurement_of_quantity().read(),
        belong_to_company:               global_model.selected_company().read().unwrap(),
    }
}

fn handle_clean(local_model: Arc<impl LocalModel>) {
    local_model.process_id().reset();
    local_model.account_name().reset();
    local_model.is_debit().reset();
    local_model.is_permanent_account().reset();
    local_model.notes().reset();
    local_model.unit_of_measurement_of_quantity().reset();
    local_model.is_loading().reset();
    local_model.account_name_error().reset();
}

async fn handle_submit(
    global_model: Arc<impl GlobalModel>,
    local_model: Arc<impl LocalModel>,
    cache: CacheStruct,
    sender_to_process_manager: MpscSender<MessageToProcessManager>,
) {
    let process_id = ProcessId::default();
    local_model.process_id().set(Some(process_id));

    let dialog_signal_adapter = Arc::new(DialogSignalAdapter(local_model.show_dialog()));

    let data = build_input(global_model, local_model.clone());

    let data: TypeOperationClientInput = TypeOperationClientInput(Arc::new(data));

    let local_model1 = local_model.clone();
    handle_fall_back(
        cache,
        sender_to_process_manager,
        dialog_signal_adapter,
        process_id,
        data,
        move |data| {
            let result = match data {
                Ok(ok) => {
                    let a = ok.0;
                    let a: Box<dyn Any> = a;
                    let a: Box<Ok> = a.downcast().unwrap();
                    let a: Ok = a.as_ref().clone();
                    Ok(a)
                }
                Err(err) => {
                    let a = err.0;
                    let a: Box<dyn Any> = a;
                    let a: Box<Error> = a.downcast().unwrap();
                    let a: Error = a.as_ref().clone();
                    Err(a)
                }
            };
            apply_on_the_model(&result, local_model.clone());

            let is_ok = result.is_ok();
            if is_ok {
                handle_clean(local_model.clone());
            }

            is_ok
        },
    )
    .await;

    local_model1.clone().is_loading().reset();
}

async fn handle_check(
    global_model: Arc<impl GlobalModel>,
    local_model: Arc<impl LocalModel>,
    mut cache: CacheStruct,
) {
    let data = build_input(global_model, local_model.clone());

    let data: TypeOperationClientInput = TypeOperationClientInput(Arc::new(data));

    let mut receiver_to_response =
        cache.send_to_cache_actor(CachingStrategy::ReadCacheOnly, TxnNumber::default(), data).await;

    match receiver_to_response.recv().await.unwrap() {
        Response::CloseTheChannel => {}
        Response::ServerCannotBeReached => {}
        Response::Data {
            is_response_from_server: _,
            data,
        } => {
            let result = match data {
                Ok(ok) => {
                    let a = ok.0;
                    let a: Box<dyn Any> = a;
                    let a: Box<Ok> = a.downcast().unwrap();
                    let a: Ok = a.as_ref().clone();
                    Ok(a)
                }
                Err(err) => {
                    let a = err.0;
                    let a: Box<dyn Any> = a;
                    let a: Box<Error> = a.downcast().unwrap();
                    let a: Error = a.as_ref().clone();
                    Err(a)
                }
            };
            apply_on_the_model(&result, local_model);
        }
    }
}
