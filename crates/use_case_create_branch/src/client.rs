use crate::domain::BranchNameError;
use crate::domain::Error;
use crate::domain::Input;
use crate::domain::LocationError;
use crate::domain::Ok;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use anyhow::Result;
use infrastructure::row_id::Id;
use infrastructure::row_id::RowId;
use infrastructure::runtime::Rt;
use infrastructure::runtime::Runtime;
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
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;
use utility::cache::ProcessId;
use utility::cache::ResourceName;
use utility::cache::TraitOperationClientError;
use utility::cache::TraitOperationClientInput;
use utility::cache::TraitOperationClientOk;
use utility::cache::TypeOperationClientError;
use utility::cache::TypeOperationClientResult;
use utility::cache::new_resource_name;
use utility::tools::select_strings;
use utility::ui_effect::MessageTrait;
use utility::ui_effect::UiContext;
use utility::ui_orchestration::GenricAsyncState;
use utility::ui_orchestration::UseCaseClient;
use utility::ui_orchestration::UserConsent;
use utility::ui_orchestration::handle_check;
use utility::ui_orchestration::handle_submit;
use utility::ui_orchestration::handle_subscribe;
use utility::ui_orchestration::handle_unsubscribe;
use utility_ui::domain::Dialog;
use utility_ui::domain::HashimSignal;

const RESOURCES_NAME_TO_POKE: &[ResourceName] = &[new_resource_name("branches")];
const RESOURCES_NAME_TO_LISTEN: &[ResourceName] = &[new_resource_name("branches")];

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

    fn state_less_check(&self) -> Option<TypeOperationClientError> {
        let errr = self.state_less_check();
        if errr.is_there_error() {
            Some(Box::new(errr))
        } else {
            None
        }
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
    SpawnTimer,
    Submit {
        async_input: AsyncInput,
        is_to_server: bool,
    },
    Check {
        async_input: AsyncInput,
    },
    Subscribe,
    UnSubscribe,
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
    Subscribe,
    UnSubscribe,
}

#[derive(Debug, Clone)]
pub enum Observe {
    Timeout,
    Result(AsyncState),
    Refresh,
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
    vec![Effect::Check { async_input: a }]
}

pub fn update(
    msg: Message,
    local_model: &impl LocalModel,
    global_model: &impl GlobalModel,
) -> (Vec<Change>, Vec<Effect>) {
    if let Message::Intent(ref a) = msg {
        match a {
            Intent::Consent(_) | Intent::Subscribe | Intent::UnSubscribe => {}
            _ => {
                if local_model.async_state().read().is_loading() {
                    return (vec![], vec![]);
                }
            }
        }
    }

    match msg {
        Message::Intent(intent) => match intent {
            Intent::Submit => {
                let Some(user_uuid) = global_model.user_uuid() else {
                    return (vec![], vec![]);
                };

                let Some(company_belong) = resolved_company_uuid(local_model, global_model) else {
                    return (vec![], vec![]);
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

                let effect = vec![
                    Effect::SpawnTimer,
                    Effect::Submit {
                        async_input,
                        is_to_server: true,
                    },
                ];

                (change, effect)
            }
            Intent::Consent(v) => {
                let (change, effect) = match v {
                    UserConsent::CancelOperation => (
                        vec![
                            Change::ShowDialog(Dialog::Hide),
                            Change::AsyncState(AsyncState::Idle),
                        ],
                        vec![],
                    ),
                    UserConsent::WaitForServerResponse => (
                        vec![Change::ShowDialog(Dialog::Hide)],
                        vec![Effect::SpawnTimer],
                    ),
                    UserConsent::DontWaitForServerResponse => {
                        let Some(user_uuid) = global_model.user_uuid() else {
                            return (vec![], vec![]);
                        };
                        let Some(company_belong) = resolved_company_uuid(local_model, global_model)
                        else {
                            return (vec![], vec![]);
                        };
                        let async_input = AsyncInput {
                            user_uuid,
                            company_belong,
                            branch_name: local_model.branch_name().read(),
                            currency: local_model.currency().read(),
                            location: local_model.location().read(),
                        };

                        (
                            vec![Change::ShowDialog(Dialog::Hide)],
                            vec![Effect::Submit {
                                async_input,
                                is_to_server: false,
                            }],
                        )
                    }
                };

                (change, effect)
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
                (change, vec![])
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
                let effect = build_check_effect(local_model, global_model, resolved, |_| {});
                (change, effect)
            }
            Intent::SelectedCompany(idx) => {
                let list = list_of_companies_to_display(local_model, global_model);
                match list.get(idx) {
                    Some((company_uuid, company_name)) => {
                        let change = vec![Change::CompanyName(company_name.clone())];
                        let effect = build_check_effect(
                            local_model,
                            global_model,
                            Some(company_uuid.clone()),
                            |_| {},
                        );
                        (change, effect)
                    }
                    None => (vec![], vec![]),
                }
            }
            Intent::BranchName(v) => {
                let change = vec![Change::BranchName(v.clone())];
                let effect = build_check_effect(local_model, global_model, None, |a| {
                    a.branch_name = v;
                });
                (change, effect)
            }
            Intent::Currency(v) => {
                let change = vec![Change::Currency(v.clone())];
                let effect = build_check_effect(local_model, global_model, None, |a| {
                    a.currency = v;
                });
                (change, effect)
            }
            Intent::Latitude(v) => {
                let mut loc = local_model.location().read();
                loc.latitude = v;
                let loc_for_effect = loc.clone();
                let change = vec![Change::Location(loc)];
                let effect = build_check_effect(local_model, global_model, None, |a| {
                    a.location = loc_for_effect;
                });
                (change, effect)
            }
            Intent::Longitude(v) => {
                let mut loc = local_model.location().read();
                loc.longitude = v;
                let loc_for_effect = loc.clone();
                let change = vec![Change::Location(loc)];
                let effect = build_check_effect(local_model, global_model, None, |a| {
                    a.location = loc_for_effect;
                });
                (change, effect)
            }
            Intent::Subscribe => {
                let effect = vec![Effect::Subscribe];
                (vec![], effect)
            }
            Intent::UnSubscribe => {
                let effect = vec![Effect::UnSubscribe];
                (vec![], effect)
            }
        },
        Message::Observe(observe) => match observe {
            Observe::Timeout => {
                let change = match local_model.async_state().read() {
                    AsyncState::Loading { .. } => vec![Change::ShowDialog(Dialog::Show)],
                    _ => vec![],
                };
                (change, vec![])
            }
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
                (change, vec![])
            }
            Observe::Refresh => {
                let effect = build_check_effect(local_model, global_model, None, |_| {});
                (vec![], effect)
            }
        },
    }
}

pub fn apply(msg: Change, local_model: &impl LocalModel, _global_model: &impl GlobalModel) {
    match msg {
        Change::ShowDialog(i) => local_model.show_dialog().set(i),
        Change::CompanyName(i) => local_model.company_name().set(i),
        Change::BranchName(i) => local_model.branch_name().set(i),
        Change::Currency(i) => local_model.currency().set(i),
        Change::Location(i) => local_model.location().set(i),
        Change::AsyncState(i) => local_model.async_state().set(i),
    }
}

pub async fn effect(msg: Effect, process_id: ProcessId, context: UiContext) -> Result<()> {
    match msg {
        Effect::SpawnTimer => {
            Rt::spawn_local(async move {
                Rt::sleep(Duration::from_secs(5)).await;
                let _ = context
                    .sender_to_commander
                    .async_send(process_id, Message::Observe(Observe::Timeout))
                    .await;
            });
        }
        Effect::Submit {
            async_input,
            is_to_server,
        } => {
            handle_submit::<Wire>(process_id, async_input, context, is_to_server).await?;
        }
        Effect::Check { async_input } => {
            handle_check::<Wire>(process_id, async_input, context).await?;
        }
        Effect::Subscribe => {
            handle_subscribe::<Wire>(process_id, context, RESOURCES_NAME_TO_LISTEN).await?;
        }
        Effect::UnSubscribe => {
            handle_unsubscribe::<Wire>(process_id, context).await?;
        }
    }
    Ok(())
}

pub struct Wire;

impl UseCaseClient for Wire {
    type Input = Input;
    type AsyncInput = AsyncInput;
    type Ok = Ok;
    type Error = Error;
    type Message = Message;

    fn build_input(input: &Self::AsyncInput) -> Self::Input {
        Input {
            user_uuid: input.user_uuid.clone(),
            new_uuid: BranchUuid::from(UuidType::from(Id::generate())),
            company_belong: input.company_belong.clone(),
            branch_name: input.branch_name.clone(),
            currency: input.currency.clone(),
            location: input.location.clone(),
        }
    }

    fn msg_success_submit(input: Self::AsyncInput, ok: Self::Ok) -> Self::Message {
        Message::Observe(Observe::Result(AsyncState::Success { input, ok }))
    }

    fn msg_failure(input: Self::AsyncInput, error: Self::Error) -> Self::Message {
        Message::Observe(Observe::Result(AsyncState::Failure { input, error }))
    }

    fn msg_timeout() -> Self::Message {
        Message::Observe(Observe::Timeout)
    }

    fn msg_success_check() -> Self::Message {
        Message::Observe(Observe::Result(AsyncState::Idle))
    }

    fn msg_refresh() -> Self::Message {
        Message::Observe(Observe::Refresh)
    }
}
