use crate::domain::CompanyNameError;
use crate::domain::Error;
use crate::domain::Input;
use crate::domain::MyResult;
use crate::domain::Ok;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use infrastructure::actors::Sender;
use infrastructure::row_id::Id;
use infrastructure::row_id::RowId;
use kernel::client::Cache;
use kernel::new_types::CompanyUuid;
use kernel::new_types::UserUuid;
use kernel::new_types::UuidType;
use kernel::types::Currency;
use kernel::types::DatabaseRead;
use kernel::types::MyErrorTrait;
use std::any::Any;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::Mutex;
use utility::cache::ResourceName;
use utility::cache::TraitOperationClientError;
use utility::cache::TraitOperationClientInput;
use utility::cache::TraitOperationClientOk;
use utility::cache::TypeOperationClientInput;
use utility::cache::TypeOperationClientResult;
use utility::cache::new_resource_name;
use utility::process_manager::MessageToProcessManager;
use utility::process_manager::ProcessDialog;
use utility::process_manager::ProcessId;
use utility::process_manager::UserConsent;
use utility::ui_effect::Commander;
use utility::ui_effect::MessageTrait;
use utility::ui_effect::Model;
use utility::ui_effect::UiContext;
use utility::ui_orchestration::handle_fall_back;
use utility_ui::domain::Dialog;
use utility_ui::domain::HashimSignal;

const RESOURCES_NAME_TO_POKE: &[ResourceName] = &[new_resource_name("companies")];

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

pub trait GlobalModel: Model + 'static {
    fn user_uuid(&self) -> impl HashimSignal<Option<UserUuid>>;
}

pub trait LocalModel: 'static {
    fn show_dialog(&self) -> impl HashimSignal<Dialog>;
    fn is_loading(&self) -> impl HashimSignal<bool>;
    fn company_name(&self) -> impl HashimSignal<String>;
    fn currency(&self) -> impl HashimSignal<Currency>;
    fn company_name_error(&self) -> impl HashimSignal<Option<CompanyNameError>>;
}

#[derive(Debug, Clone)]
pub enum Change {
    ShowDialog(Dialog),
    IsLoading(bool),
    CompanyName(String),
    Currency(Currency),
    CompanyNameError(Option<CompanyNameError>),
}

#[derive(Debug, Clone)]
pub enum Effect {
    Submit {
        process_id: ProcessId,
        user_uuid: UserUuid,
        company_name: String,
        currency: Currency,
    },
    Consent {
        process_id: ProcessId,
        user_consent: UserConsent,
    },
}

#[derive(Debug, Clone)]
pub enum Intent {
    Clean,
    CompanyName(String),
    Consent(UserConsent),
    Currency(Currency),
    Submit,
}

#[derive(Debug, Clone)]
pub enum Observe {
    ShowDialog,
    HideDialog,
    Result(MyResult),
}

#[derive(Debug, Clone)]
pub enum Message {
    Intent(Intent),
    Observe(Observe),
}

impl MessageTrait for Message {}

pub fn reduce(
    msg: Message,
    process_id: ProcessId,
    local_model: &impl LocalModel,
    global_model: &impl GlobalModel,
) -> Result<(Vec<Change>, Vec<Effect>)> {
    match msg {
        Message::Intent(intent) => match intent {
            Intent::Clean => {
                let change = vec![
                    Change::ShowDialog(Default::default()),
                    Change::IsLoading(Default::default()),
                    Change::CompanyName(Default::default()),
                    Change::Currency(Default::default()),
                    Change::CompanyNameError(Default::default()),
                ];
                let effect = vec![];
                Ok((change, effect))
            }
            Intent::CompanyName(v) => {
                let change = vec![Change::CompanyName(v)];
                let effect = vec![];
                Ok((change, effect))
            }
            Intent::Consent(v) => {
                let change = vec![Change::ShowDialog(Default::default())];
                let effect = vec![Effect::Consent {
                    process_id,
                    user_consent: v,
                }];
                Ok((change, effect))
            }
            Intent::Currency(v) => {
                let change = vec![Change::Currency(v)];
                let effect = vec![];
                Ok((change, effect))
            }
            Intent::Submit => {
                let change = vec![Change::IsLoading(true)];
                let effect = vec![Effect::Submit {
                    process_id,
                    user_uuid: global_model
                        .user_uuid()
                        .read()
                        .context("user uuid not found")?,
                    company_name: local_model.company_name().read(),
                    currency: local_model.currency().read(),
                }];
                Ok((change, effect))
            }
        },
        Message::Observe(observe) => match observe {
            Observe::ShowDialog => {
                let change = vec![Change::ShowDialog(Dialog::Show)];
                let effect = vec![];
                Ok((change, effect))
            }
            Observe::HideDialog => {
                let change = vec![Change::ShowDialog(Dialog::Hide)];
                let effect = vec![];
                Ok((change, effect))
            }
            Observe::Result(v) => {
                let change = match v {
                    Ok(_) => {
                        vec![
                            Change::ShowDialog(Default::default()),
                            Change::IsLoading(Default::default()),
                            Change::CompanyName(Default::default()),
                            Change::Currency(Default::default()),
                            Change::CompanyNameError(Default::default()),
                        ]
                    }
                    Err(a) => {
                        vec![
                            Change::ShowDialog(Default::default()),
                            Change::IsLoading(Default::default()),
                            Change::CompanyNameError(a.company_name),
                        ]
                    }
                };
                let effect = vec![];
                Ok((change, effect))
            }
        },
    }
}

pub fn update(msg: Change, local_model: &impl LocalModel, global_model: &impl GlobalModel) {
    match msg {
        Change::ShowDialog(i) => local_model.show_dialog().set(i),
        Change::IsLoading(i) => local_model.is_loading().set(i),
        Change::CompanyName(i) => local_model.company_name().set(i),
        Change::Currency(i) => local_model.currency().set(i),
        Change::CompanyNameError(i) => local_model.company_name_error().set(i),
    }
}

pub async fn effect(msg: Effect, mut context: UiContext) -> Result<()> {
    match msg {
        Effect::Submit {
            process_id,
            user_uuid,
            company_name,
            currency,
        } => {
            handle_submit(
                process_id,
                &Input {
                    user_uuid,
                    new_uuid: CompanyUuid::from(UuidType::from(Id::generate())),
                    company_name,
                    currency,
                },
                context,
            )
            .await?;
        }
        Effect::Consent {
            process_id,
            user_consent,
        } => {
            context
                .sender_to_process_manager
                .send(MessageToProcessManager::FromUser {
                    process_id: process_id,
                    consent: user_consent,
                })
                .await?;
        }
    }

    Ok(())
}

struct A {
    process_id: ProcessId,
    sender: Commander,
}

impl ProcessDialog for A {
    fn show(&self) {
        self.sender
            .send(self.process_id, Message::Observe(Observe::ShowDialog));
    }

    fn hide(&self) {
        self.sender
            .send(self.process_id, Message::Observe(Observe::HideDialog));
    }
}

async fn handle_submit(process_id: ProcessId, input: &Input, context: UiContext) -> Result<()> {
    let context1 = context.clone();

    let dialog_signal_adapter = Arc::new(A {
        process_id,
        sender: context.sender_to_commander,
    });

    let data: TypeOperationClientInput = Arc::new(input.clone());

    handle_fall_back(
        context.sender_to_error,
        context.cache,
        context.sender_to_process_manager,
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

            let is_ok = result.is_ok();

            context1
                .sender_to_commander
                .send(process_id, Message::Observe(Observe::Result(result)));

            Ok(is_ok)
        },
    )
    .await?;

    Ok(())
}
