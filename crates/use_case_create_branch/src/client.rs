use crate::domain::Error;
use crate::domain::Input;
use crate::domain::MyResult;
use crate::domain::Ok;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use infrastructure::actors::MpscSender;
use infrastructure::actors::Sender;
use infrastructure::row_id::Id;
use infrastructure::row_id::RowId;
use kernel::client::Cache;
use kernel::client::DialogSignalAdapter;
use kernel::new_types::BranchUuid;
use kernel::new_types::CompanyUuid;
use kernel::new_types::UserUuid;
use kernel::new_types::UuidType;
use kernel::types::Currency;
use kernel::types::DatabaseRead;
use kernel::types::Location;
use kernel::types::MyErrorTrait;
use std::any::Any;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;
use utility::cache::CacheStruct;
use utility::cache::ResourceName;
use utility::cache::TraitOperationClientError;
use utility::cache::TraitOperationClientInput;
use utility::cache::TraitOperationClientOk;
use utility::cache::TypeOperationClientInput;
use utility::cache::TypeOperationClientResult;
use utility::cache::new_resource_name;
use utility::process_manager::MessageToProcessManager;
use utility::process_manager::ProcessId;
use utility::process_manager::UserConsent;
use utility::tools::select_strings;
use utility::ui_effect::MessageTrait;
use utility::ui_effect::Model;
use utility::ui_effect::UiContext;
use utility::ui_orchestration::handle_fall_back;
use utility_ui::domain::Dialog;
use utility_ui::domain::HashimSignal;

const RESOURCES_NAME_TO_POKE: &[ResourceName] = &[new_resource_name("branches")];

impl TraitOperationClientOk for Ok {
    fn subs_to_poke(&self) -> &'static [ResourceName] {
        RESOURCES_NAME_TO_POKE
    }
}

impl TraitOperationClientError for Error {
    fn subs_to_poke(&self) -> &'static [ResourceName] {
        RESOURCES_NAME_TO_POKE
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
) -> Result<TypeOperationClientResult> {
    let errr = input.state_full_check::<DBReader>(cache).await?;

    if errr.is_there_error() {
        return Ok(Err(Box::new(errr)));
    }

    let state_less_operation = input.state_less_operation();
    Ok(Ok(Arc::new(state_less_operation)))
}

#[derive(Debug, Clone)]
pub enum Message {
    Submit,
    Consent(UserConsent),
    Clean,
    CompanyName(String),
    SelectedCompany(usize),
    BranchName(String),
    Currency(Currency),
    Latitude(String),
    Longitude(String),
}

impl MessageTrait for Message {}

type Type1 = Input;
type Type2 = Input;
type Type3 = MyResult;
type Type4 = MyResult;

pub trait GlobalModel: Model + 'static {
    fn list_of_companies(&self) -> impl HashimSignal<Vec<(CompanyUuid, String)>>;
    fn user_uuid(&self) -> impl HashimSignal<Option<UserUuid>>;
    fn selected_company(&self) -> impl HashimSignal<Option<CompanyUuid>>;
}

pub trait LocalModel: 'static {
    fn process_id(&self) -> impl HashimSignal<Option<ProcessId>>;
    fn show_dialog(&self) -> impl HashimSignal<Dialog>;
    fn is_loading(&self) -> impl HashimSignal<bool>;
    fn list_of_companies_to_display(&self) -> impl HashimSignal<Vec<(CompanyUuid, String)>>;
    fn company_name_error(&self) -> impl HashimSignal<String>;
    fn selected_company_name(&self) -> impl HashimSignal<String>;
    fn selected_company_uuid(&self) -> impl HashimSignal<Option<CompanyUuid>>;
    fn branch_name(&self) -> impl HashimSignal<String>;
    fn currency(&self) -> impl HashimSignal<Currency>;
    fn location(&self) -> impl HashimSignal<Location>;
    fn branch_name_error(&self) -> impl HashimSignal<Option<String>>;
    fn location_error(&self) -> impl HashimSignal<Option<String>>;
}

fn apply_on_the_model_for_submit(output: &Type4, local_model: Arc<impl LocalModel>) {
    match output {
        Ok(_) => {
            handle_clean(local_model);
        }
        Err(business_error) => {
            local_model.branch_name_error().set(
                business_error
                    .branch_name
                    .as_ref()
                    .map(|_| String::from("invalid branch name")),
            );
            local_model.location_error().set(
                business_error
                    .location
                    .as_ref()
                    .map(|_| String::from("invalid location")),
            );
        }
    }
}

pub async fn update_generic(
    message: Message,
    local_model: Arc<impl LocalModel>,
    mut context: UiContext<impl GlobalModel>,
) -> Result<()> {
    match message {
        Message::Submit => {
            handle_submit(
                context.sender_to_error,
                context.model,
                local_model,
                context.cache,
                context.sender_to_process_manager,
            )
            .await?;
        }
        Message::Consent(i) => {
            context
                .sender_to_process_manager
                .send(MessageToProcessManager::FromUser {
                    process_id: local_model
                        .process_id()
                        .read()
                        .context("process id not found")?,
                    consent: i,
                })
                .await?;
        }
        Message::Clean => handle_clean(local_model),
        Message::BranchName(v) => local_model.branch_name().set(v),
        Message::Currency(v) => local_model.currency().set(v),
        Message::Latitude(v) => {
            let mut loc = local_model.location().read();
            loc.latitude = v.parse().unwrap_or_default();
            local_model.location().set(loc);
        }
        Message::Longitude(v) => {
            let mut loc = local_model.location().read();
            loc.longitude = v.parse().unwrap_or_default();
            local_model.location().set(loc);
        }
        Message::CompanyName(v) => {
            let list_of_companies = context.model.list_of_companies().read();
            let a = select_strings(list_of_companies, v.clone(), |a| a.1.as_str());

            let list_of_companies_to_display = local_model.list_of_companies_to_display();
            list_of_companies_to_display.set(a);
            local_model.selected_company_name().set(v.clone());
            let selected_company_uuid = local_model.selected_company_uuid();

            match list_of_companies_to_display.read().get(0) {
                Some(a) => {
                    if v == a.1 {
                        selected_company_uuid.set(Some(a.0.clone()));
                    } else {
                        selected_company_uuid.set(None);
                    }
                }
                None => {
                    selected_company_uuid.set(None);
                }
            }
        }
        Message::SelectedCompany(v) => {
            let a = local_model.list_of_companies_to_display().read();

            if let Some(a) = a.get(v) {
                local_model.selected_company_name().set(a.1.clone());
                local_model.selected_company_uuid().set(Some(a.0.clone()));
            };
        }
    }

    Ok(())
}

fn build_input(
    global_model: Arc<impl GlobalModel>,
    local_model: Arc<impl LocalModel>,
) -> Result<Option<Type1>> {
    Ok(Some(Input {
        user_uuid: global_model
            .user_uuid()
            .read()
            .context("user uuid not found")?,
        new_uuid: BranchUuid::from(UuidType::from(Id::generate())),
        company_belong: {
            let company_name = local_model.selected_company_name().read();
            let read = local_model.list_of_companies_to_display().read();

            let first_company_name = match read.get(0) {
                Some(a) => a.1.clone(),
                None => return Ok(None),
            };

            if !company_name.is_empty() && company_name != first_company_name {
                local_model
                    .company_name_error()
                    .set("please select company".to_string());
                return Ok(None);
            } else {
                local_model.company_name_error().reset();
            }

            match local_model.selected_company_uuid().read() {
                Some(a) => a,
                None => global_model
                    .selected_company()
                    .read()
                    .context("company uuid not found")?,
            }
        },
        branch_name: local_model.branch_name().read(),
        currency: local_model.currency().read(),
        location: local_model.location().read(),
    }))
}

fn handle_clean(local_model: Arc<impl LocalModel>) {
    local_model.branch_name().reset();
    local_model.currency().reset();
    local_model.location().reset();
    local_model.is_loading().reset();
    local_model.branch_name_error().reset();
    local_model.location_error().reset();
}

async fn handle_submit(
    sender_to_error: MpscSender<anyhow::Error>,
    global_model: Arc<impl GlobalModel>,
    local_model: Arc<impl LocalModel>,
    cache: CacheStruct,
    sender_to_process_manager: MpscSender<MessageToProcessManager>,
) -> Result<()> {
    let process_id = ProcessId::default();
    local_model.process_id().set(Some(process_id));

    let dialog_signal_adapter = Arc::new(DialogSignalAdapter(local_model.show_dialog()));

    let data = build_input(global_model, local_model.clone())?;
    let data = match data {
        Some(a) => a,
        None => return Ok(()),
    };
    let data: TypeOperationClientInput = Arc::new(data);

    let local_model1 = local_model.clone();
    handle_fall_back(
        sender_to_error,
        cache,
        sender_to_process_manager,
        dialog_signal_adapter,
        process_id,
        data,
        move |data| {
            let result = match data {
                Ok(ok) => {
                    let a: Arc<dyn Any> = ok;
                    let a: &Ok = a.downcast_ref().context("downcast error")?;
                    Ok(a.clone())
                }
                Err(err) => {
                    let a: Box<dyn Any> = err;
                    let a: Box<Error> = a.downcast().map_err(|_| anyhow!("downcast error"))?;
                    Err(*a)
                }
            };
            apply_on_the_model_for_submit(&result, local_model.clone());

            Ok(result.is_ok())
        },
    )
    .await?;

    local_model1.clone().is_loading().reset();

    Ok(())
}
