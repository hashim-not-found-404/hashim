use crate::domain::BranchNameError;
use crate::domain::Error;
use crate::domain::Input;
use crate::domain::LocationError;
use crate::domain::Ok;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use infrastructure::actors::Receiver;
use infrastructure::actors::Sender;
use infrastructure::row_id::Id;
use infrastructure::row_id::RowId;
use kernel::client::Cache;
use kernel::new_types::BranchUuid;
use kernel::new_types::CompanyUuid;
use kernel::new_types::UserUuid;
use kernel::new_types::UuidType;
use kernel::types::Currency;
use kernel::types::DatabaseRead;
use kernel::types::Location;
use kernel::types::MyErrorTrait;
use serde::Deserialize;
use serde::Serialize;
use std::any::Any;
use std::ops::Deref;
use std::sync::Arc;
use utility::cache::CachingStrategy;
use utility::cache::ResourceName;
use utility::cache::Response;
use utility::cache::TraitOperationClientError;
use utility::cache::TraitOperationClientInput;
use utility::cache::TraitOperationClientOk;
use utility::cache::TypeOperationClientInput;
use utility::cache::TypeOperationClientResult;
use utility::cache::new_resource_name;
use utility::dtos::TxnNumber;
use utility::process_manager::MessageToProcessManager;
use utility::process_manager::ProcessDialog;
use utility::process_manager::ProcessId;
use utility::process_manager::UserConsent;
use utility::tools::select_strings;
use utility::ui_effect::Commander;
use utility::ui_effect::MessageTrait;
use utility::ui_effect::UiContext;
use utility::ui_orchestration::GenricAsyncState;
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

pub(crate) type AsyncState = GenricAsyncState<AsyncInput, Ok, Error>;

pub trait GlobalModel {
    fn user_uuid(&self) -> Option<UserUuid>;
    fn selected_company(&self) -> Option<CompanyUuid>;
    fn list_of_companies(&self) -> Vec<(CompanyUuid, String)>;
}

pub trait LocalModel {
    fn show_dialog(&self) -> impl HashimSignal<Dialog>;
    fn company_name(&self) -> impl HashimSignal<String>;
    fn branch_name(&self) -> impl HashimSignal<String>;
    fn currency(&self) -> impl HashimSignal<Currency>;
    fn location(&self) -> impl HashimSignal<Location>;
    fn async_state(&self) -> impl HashimSignal<AsyncState>;
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum CompanyNameError {
    NotSelected,
    NotExist,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AsyncInput {
    pub user_uuid: UserUuid,
    pub company_belong: CompanyUuid,
    pub branch_name: String,
    pub currency: Currency,
    pub location: Location,
}

pub fn error_branch_name(local_model: &impl LocalModel) -> Option<BranchNameError> {
    match local_model.async_state().read() {
        AsyncState::Failure { error, .. } => error.branch_name,
        _ => None,
    }
}

pub fn error_location(local_model: &impl LocalModel) -> Option<LocationError> {
    match local_model.async_state().read() {
        AsyncState::Failure { error, .. } => error.location,
        _ => None,
    }
}

pub fn resolved_company_uuid(
    local_model: &impl LocalModel,
    global_model: &impl GlobalModel,
) -> Option<CompanyUuid> {
    let typed = local_model.company_name().read();
    if typed.is_empty() {
        return global_model.selected_company();
    }
    global_model
        .list_of_companies()
        .into_iter()
        .find_map(|(uuid, name)| (name == typed).then_some(uuid))
}

pub fn company_name_error(
    local_model: &impl LocalModel,
    global_model: &impl GlobalModel,
) -> Option<CompanyNameError> {
    let typed = local_model.company_name().read();
    if typed.is_empty() {
        if global_model.selected_company().is_none() {
            return Some(CompanyNameError::NotSelected);
        }
        return None;
    }
    if resolved_company_uuid(local_model, global_model).is_some() {
        None
    } else {
        Some(CompanyNameError::NotExist)
    }
}

pub fn list_of_companies_to_display(
    local_model: &impl LocalModel,
    global_model: &impl GlobalModel,
) -> Vec<(CompanyUuid, String)> {
    let typed = local_model.company_name().read();
    let list = global_model.list_of_companies();
    if typed.is_empty() {
        return list;
    }
    select_strings(list, typed, |a| a.1.as_str())
}

#[derive(Debug, Clone)]
pub enum Change {
    ShowDialog(Dialog),
    CompanyName(String),
    BranchName(String),
    Currency(Currency),
    Location(Location),
    AsyncState(AsyncState),
}

#[derive(Debug, Clone)]
pub enum Effect {
    Submit {
        process_id: ProcessId,
        async_input: AsyncInput,
    },
    Consent {
        process_id: ProcessId,
        user_consent: UserConsent,
    },
    Check {
        process_id: ProcessId,
        async_input: AsyncInput,
    },
}

#[derive(Debug, Clone)]
pub enum Intent {
    Submit,
    Consent(UserConsent),
    Clean,
    CompanyName(String),
    SelectedCompany(usize),
    BranchName(String),
    Currency(Currency),
    Latitude(f64),
    Longitude(f64),
}

#[derive(Debug, Clone)]
pub enum Observe {
    ShowDialog,
    HideDialog,
    Result(AsyncState),
}

#[derive(Debug, Clone)]
pub enum Message {
    Intent(Intent),
    Observe(Observe),
}

impl MessageTrait for Message {}

fn build_check_effect(
    local_model: &impl LocalModel,
    global_model: &impl GlobalModel,
    process_id: ProcessId,
    override_company: Option<CompanyUuid>,
    mutate: impl FnOnce(&mut AsyncInput),
) -> Vec<Effect> {
    let Some(user_uuid) = global_model.user_uuid() else {
        return vec![];
    };
    let company_belong =
        override_company.or_else(|| resolved_company_uuid(local_model, global_model));
    let Some(company_belong) = company_belong else {
        return vec![];
    };
    let mut a = AsyncInput {
        user_uuid,
        company_belong,
        branch_name: local_model.branch_name().read(),
        currency: local_model.currency().read(),
        location: local_model.location().read(),
    };
    mutate(&mut a);
    vec![Effect::Check {
        process_id,
        async_input: a,
    }]
}

pub fn reduce(
    msg: Message,
    process_id: ProcessId,
    local_model: &impl LocalModel,
    global_model: &impl GlobalModel,
) -> Result<(Vec<Change>, Vec<Effect>)> {
    if let Message::Intent(ref a) = msg {
        if let Intent::Consent(_) = a {
        } else {
            if local_model.async_state().read().is_loading() {
                return Ok((vec![], vec![]));
            }
        }
    }

    match msg {
        Message::Intent(intent) => match intent {
            Intent::Submit => {
                let Some(user_uuid) = global_model.user_uuid() else {
                    return Ok((vec![], vec![]));
                };

                let Some(company_belong) = resolved_company_uuid(local_model, global_model) else {
                    return Ok((vec![], vec![]));
                };

                let async_input = AsyncInput {
                    user_uuid,
                    company_belong,
                    branch_name: local_model.branch_name().read(),
                    currency: local_model.currency().read(),
                    location: local_model.location().read(),
                };

                let change = vec![
                    Change::AsyncState(AsyncState::Loading {
                        input: async_input.clone(),
                    }),
                    Change::ShowDialog(Dialog::Hide),
                ];

                let effect = vec![Effect::Submit {
                    process_id,
                    async_input,
                }];

                Ok((change, effect))
            }
            Intent::Consent(v) => {
                let change = match v {
                    UserConsent::CancelOperation => {
                        vec![
                            Change::ShowDialog(Dialog::Hide),
                            Change::AsyncState(AsyncState::Idle),
                        ]
                    }
                    _ => vec![Change::ShowDialog(Dialog::Hide)],
                };
                let effect = vec![Effect::Consent {
                    process_id,
                    user_consent: v,
                }];
                Ok((change, effect))
            }
            Intent::Clean => {
                let change = vec![
                    Change::ShowDialog(Default::default()),
                    Change::CompanyName(Default::default()),
                    Change::BranchName(Default::default()),
                    Change::Currency(Default::default()),
                    Change::Location(Default::default()),
                    Change::AsyncState(AsyncState::Idle),
                ];
                Ok((change, vec![]))
            }
            Intent::CompanyName(v) => {
                let change = vec![Change::CompanyName(v.clone())];
                let resolved = if v.is_empty() {
                    global_model.selected_company()
                } else {
                    global_model
                        .list_of_companies()
                        .into_iter()
                        .find_map(|(uuid, name)| (name == v).then_some(uuid))
                };
                let effect =
                    build_check_effect(local_model, global_model, process_id, resolved, |_| {});
                Ok((change, effect))
            }
            Intent::SelectedCompany(idx) => {
                let list = list_of_companies_to_display(local_model, global_model);
                match list.get(idx) {
                    Some((company_uuid, company_name)) => {
                        let change = vec![Change::CompanyName(company_name.clone())];
                        let effect = build_check_effect(
                            local_model,
                            global_model,
                            process_id,
                            Some(company_uuid.clone()),
                            |_| {},
                        );
                        Ok((change, effect))
                    }
                    None => Ok((vec![], vec![])),
                }
            }
            Intent::BranchName(v) => {
                let change = vec![Change::BranchName(v.clone())];
                let effect = build_check_effect(local_model, global_model, process_id, None, |a| {
                    a.branch_name = v;
                });
                Ok((change, effect))
            }
            Intent::Currency(v) => {
                let change = vec![Change::Currency(v.clone())];
                let effect = build_check_effect(local_model, global_model, process_id, None, |a| {
                    a.currency = v;
                });
                Ok((change, effect))
            }
            Intent::Latitude(v) => {
                let mut loc = local_model.location().read();
                loc.latitude = v;
                let loc_for_effect = loc.clone();
                let change = vec![Change::Location(loc)];
                let effect = build_check_effect(local_model, global_model, process_id, None, |a| {
                    a.location = loc_for_effect;
                });
                Ok((change, effect))
            }
            Intent::Longitude(v) => {
                let mut loc = local_model.location().read();
                loc.longitude = v;
                let loc_for_effect = loc.clone();
                let change = vec![Change::Location(loc)];
                let effect = build_check_effect(local_model, global_model, process_id, None, |a| {
                    a.location = loc_for_effect;
                });
                Ok((change, effect))
            }
        },
        Message::Observe(observe) => match observe {
            Observe::ShowDialog => Ok((vec![Change::ShowDialog(Dialog::Show)], vec![])),
            Observe::HideDialog => Ok((vec![Change::ShowDialog(Dialog::Hide)], vec![])),
            Observe::Result(result) => {
                let change = match result {
                    AsyncState::Success { .. } => {
                        vec![
                            Change::ShowDialog(Default::default()),
                            Change::CompanyName(Default::default()),
                            Change::BranchName(Default::default()),
                            Change::Currency(Default::default()),
                            Change::Location(Default::default()),
                            Change::AsyncState(result),
                        ]
                    }
                    _ => vec![
                        Change::ShowDialog(Default::default()),
                        Change::AsyncState(result),
                    ],
                };
                Ok((change, vec![]))
            }
        },
    }
}

pub fn update(msg: Change, local_model: &impl LocalModel, _global_model: &impl GlobalModel) {
    match msg {
        Change::ShowDialog(i) => local_model.show_dialog().set(i),
        Change::CompanyName(i) => local_model.company_name().set(i),
        Change::BranchName(i) => local_model.branch_name().set(i),
        Change::Currency(i) => local_model.currency().set(i),
        Change::Location(i) => local_model.location().set(i),
        Change::AsyncState(i) => local_model.async_state().set(i),
    }
}

pub async fn effect(msg: Effect, mut context: UiContext) -> Result<()> {
    match msg {
        Effect::Submit {
            process_id,
            async_input,
        } => {
            handle_submit(process_id, async_input, context).await?;
        }
        Effect::Consent {
            process_id,
            user_consent,
        } => {
            context
                .sender_to_process_manager
                .send(MessageToProcessManager::FromUser {
                    process_id,
                    consent: user_consent,
                })
                .await?;
        }
        Effect::Check {
            process_id,
            async_input,
        } => {
            handle_check(process_id, async_input, context).await?;
        }
    }
    Ok(())
}

struct DialogDispatchAdapter {
    process_id: ProcessId,
    sender: Commander,
}

impl ProcessDialog for DialogDispatchAdapter {
    fn show(&self) {
        self.sender
            .send(self.process_id, Message::Observe(Observe::ShowDialog));
    }

    fn hide(&self) {
        self.sender
            .send(self.process_id, Message::Observe(Observe::HideDialog));
    }
}

async fn handle_submit(process_id: ProcessId, input: AsyncInput, context: UiContext) -> Result<()> {
    let context1 = context.clone();

    let dialog_signal_adapter = Arc::new(DialogDispatchAdapter {
        process_id,
        sender: context.sender_to_commander.clone(),
    });

    let new_uuid = BranchUuid::from(UuidType::from(Id::generate()));

    let data: TypeOperationClientInput = Arc::new(Input {
        user_uuid: input.user_uuid.clone(),
        new_uuid,
        company_belong: input.company_belong.clone(),
        branch_name: input.branch_name.clone(),
        currency: input.currency.clone(),
        location: input.location.clone(),
    });

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

            let async_state = match result {
                Ok(a) => AsyncState::Success {
                    input: input.clone(),
                    ok: a,
                },
                Err(a) => AsyncState::Failure {
                    input: input.clone(),
                    error: a,
                },
            };

            context1
                .sender_to_commander
                .send(process_id, Message::Observe(Observe::Result(async_state)));

            Ok(is_ok)
        },
    )
    .await?;

    Ok(())
}

async fn handle_check(process_id: ProcessId, input: AsyncInput, context: UiContext) -> Result<()> {
    let mut cache = context.cache.clone();
    let sender_to_commander = context.sender_to_commander.clone();

    let new_uuid = BranchUuid::from(UuidType::from(Id::generate()));

    let data: TypeOperationClientInput = Arc::new(Input {
        user_uuid: input.user_uuid.clone(),
        new_uuid,
        company_belong: input.company_belong.clone(),
        branch_name: input.branch_name.clone(),
        currency: input.currency.clone(),
        location: input.location.clone(),
    });

    let mut receiver_to_response = cache
        .send_to_cache_actor(CachingStrategy::ReadCacheOnly, TxnNumber::default(), data)
        .await?;

    match receiver_to_response.recv().await? {
        Response::CloseTheChannel | Response::ServerCannotBeReached => {}
        Response::Data { data, .. } => {
            let new_state = match data {
                Ok(_) => AsyncState::Idle,
                Err(err) => {
                    let a: Box<dyn Any> = err;
                    let a: Box<Error> = a.downcast().map_err(|_| anyhow!("downcast error"))?;
                    AsyncState::Failure { input, error: *a }
                }
            };
            sender_to_commander.send(process_id, Message::Observe(Observe::Result(new_state)));
        }
    }

    Ok(())
}
