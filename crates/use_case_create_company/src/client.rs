use crate::domain::CompanyNameError;
use crate::domain::Error;
use crate::domain::Input;
use crate::domain::Ok;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use anyhow::Result;
use infrastructure::row_id::Id;
use infrastructure::row_id::RowId;
use infrastructure::runtime::Rt;
use infrastructure::runtime::Runtime;
use kernel::client::Cache;
use kernel::new_types::CompanyUuid;
use kernel::new_types::UserUuid;
use kernel::new_types::UuidType;
use kernel::types::Currency;
use kernel::types::DatabaseRead;
use kernel::types::MyErrorTrait;
use serde::Deserialize;
use serde::Serialize;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;
use utility::cache::ResourceName;
use utility::cache::TraitOperationClientError;
use utility::cache::TraitOperationClientInput;
use utility::cache::TraitOperationClientOk;
use utility::cache::TypeOperationClientError;
use utility::cache::TypeOperationClientResult;
use utility::cache::new_resource_name;
use utility::ui_effect::MessageTrait;
use utility::ui_effect::Replier;
use utility::ui_effect::UiContext;
use utility::ui_orchestration::GenricAsyncState;
use utility::ui_orchestration::UseCaseClient;
use utility::ui_orchestration::UserConsent;
use utility::ui_orchestration::handle_check;
use utility::ui_orchestration::handle_submit;
use utility_ui::domain::Dialog;

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

pub trait GlobalModel {
    fn user_uuid(&self) -> Option<UserUuid>;
}

pub(crate) type AsyncState = GenricAsyncState<AsyncInput, Ok, Error>;

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct LocalModel {
    pub show_dialog: Dialog,
    pub company_name: String,
    pub currency: Currency,
    pub async_state: AsyncState,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct AsyncInput {
    pub user_uuid: UserUuid,
    pub company_name: String,
    pub currency: Currency,
}

pub fn error_company_name(local_model: &LocalModel) -> Option<CompanyNameError> {
    match local_model.async_state.clone() {
        AsyncState::Failure { error, .. } => error.company_name,
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub enum LocalChange {
    ShowDialog(Dialog),
    CompanyName(String),
    Currency(Currency),
    AsyncState(AsyncState),
}

#[derive(Debug, Clone)]
pub enum GlobalChange {}

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
    Timeout,
    Result(AsyncState),
}

#[derive(Debug, Clone)]
pub enum Message {
    Intent(Intent),
    Observe(Observe),
}

impl MessageTrait for Message {}

fn build_check_effect(
    local_model: &LocalModel,
    global_model: &impl GlobalModel,
    mutate: impl FnOnce(&mut AsyncInput),
) -> Vec<Effect> {
    let Some(user_uuid) = global_model.user_uuid() else {
        return vec![];
    };
    let mut a = AsyncInput {
        user_uuid,
        company_name: local_model.company_name.clone(),
        currency: local_model.currency.clone(),
    };
    mutate(&mut a);
    vec![Effect::Check { async_input: a }]
}

pub fn update(
    msg: Message,
    local_model: &LocalModel,
    global_model: &impl GlobalModel,
) -> (Vec<LocalChange>, Vec<GlobalChange>, Vec<Effect>) {
    if let Message::Intent(ref a) = msg {
        if let Intent::Consent(_) = a {
        } else {
            if local_model.async_state.clone().is_loading() {
                return (vec![], vec![], vec![]);
            }
        }
    }

    match msg {
        Message::Intent(intent) => match intent {
            Intent::Clean => {
                let local_change = vec![
                    LocalChange::ShowDialog(Default::default()),
                    LocalChange::CompanyName(Default::default()),
                    LocalChange::Currency(Default::default()),
                    LocalChange::AsyncState(AsyncState::Idle),
                ];
                (local_change, vec![], vec![])
            }
            Intent::CompanyName(v) => {
                let local_change = vec![LocalChange::CompanyName(v.clone())];
                let effect = build_check_effect(local_model, global_model, |a| {
                    a.company_name = v;
                });
                (local_change, vec![], effect)
            }
            Intent::Currency(v) => {
                let local_change = vec![LocalChange::Currency(v.clone())];
                let effect = build_check_effect(local_model, global_model, |a| {
                    a.currency = v;
                });
                (local_change, vec![], effect)
            }
            Intent::Consent(v) => {
                let (local_change, effect) = match v {
                    UserConsent::CancelOperation => (
                        vec![
                            LocalChange::ShowDialog(Dialog::Hide),
                            LocalChange::AsyncState(AsyncState::Idle),
                        ],
                        vec![],
                    ),
                    UserConsent::WaitForServerResponse => (
                        vec![LocalChange::ShowDialog(Dialog::Hide)],
                        vec![Effect::SpawnTimer],
                    ),
                    UserConsent::DontWaitForServerResponse => {
                        let Some(user_uuid) = global_model.user_uuid() else {
                            return (vec![], vec![], vec![]);
                        };
                        let async_input = AsyncInput {
                            user_uuid,
                            company_name: local_model.company_name.clone(),
                            currency: local_model.currency.clone(),
                        };

                        (
                            vec![LocalChange::ShowDialog(Dialog::Hide)],
                            vec![Effect::Submit {
                                async_input,
                                is_to_server: false,
                            }],
                        )
                    }
                };

                (local_change, vec![], effect)
            }
            Intent::Submit => {
                let Some(user_uuid) = global_model.user_uuid() else {
                    return (vec![], vec![], vec![]);
                };

                let async_input = AsyncInput {
                    user_uuid,
                    company_name: local_model.company_name.clone(),
                    currency: local_model.currency.clone(),
                };

                let local_change = vec![
                    LocalChange::AsyncState(AsyncState::Loading {
                        input: async_input.clone(),
                    }),
                    LocalChange::ShowDialog(Dialog::Hide),
                ];

                let effect = vec![
                    Effect::SpawnTimer,
                    Effect::Submit {
                        async_input,
                        is_to_server: true,
                    },
                ];

                (local_change, vec![], effect)
            }
        },
        Message::Observe(observe) => match observe {
            Observe::Timeout => {
                let local_change = match local_model.async_state.clone() {
                    AsyncState::Loading { .. } => vec![LocalChange::ShowDialog(Dialog::Show)],
                    _ => vec![],
                };
                (local_change, vec![], vec![])
            }
            Observe::Result(result) => {
                let local_change = match result {
                    AsyncState::Success { .. } => {
                        vec![
                            LocalChange::ShowDialog(Default::default()),
                            LocalChange::CompanyName(Default::default()),
                            LocalChange::Currency(Default::default()),
                            LocalChange::AsyncState(result),
                        ]
                    }
                    _ => vec![
                        LocalChange::ShowDialog(Default::default()),
                        LocalChange::AsyncState(result),
                    ],
                };
                (local_change, vec![], vec![])
            }
        },
    }
}

pub fn local_apply(msg: LocalChange, local_model: &mut LocalModel) {
    match msg {
        LocalChange::ShowDialog(i) => local_model.show_dialog = i,
        LocalChange::CompanyName(i) => local_model.company_name = i,
        LocalChange::Currency(i) => local_model.currency = i,
        LocalChange::AsyncState(i) => local_model.async_state = i,
    }
}

pub fn global_apply(msg: GlobalChange, global_model: &mut impl GlobalModel) {
    match msg {}
}

pub async fn effect(msg: Effect, replier: Replier, context: UiContext) -> Result<()> {
    match msg {
        Effect::SpawnTimer => {
            Rt::spawn_local(async move {
                Rt::sleep(Duration::from_secs(5)).await;
                let _ = replier(Box::new(Message::Observe(Observe::Timeout))).await;
            });
        }
        Effect::Submit {
            async_input,
            is_to_server,
        } => {
            handle_submit::<Wire>(replier, async_input, context, is_to_server).await?;
        }
        Effect::Check { async_input } => {
            handle_check::<Wire>(replier, async_input, context).await?;
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

    fn msg_success_submit(input: Self::AsyncInput, ok: Self::Ok) -> Self::Message {
        Message::Observe(Observe::Result(AsyncState::Success { input, ok }))
    }

    fn msg_failure(input: Self::AsyncInput, error: Self::Error) -> Self::Message {
        Message::Observe(Observe::Result(AsyncState::Failure { input, error }))
    }

    fn msg_success_check() -> Self::Message {
        Message::Observe(Observe::Result(AsyncState::Idle))
    }

    fn msg_timeout() -> Self::Message {
        Message::Observe(Observe::Timeout)
    }

    fn build_input(input: &Self::AsyncInput) -> Self::Input {
        Input {
            user_uuid: input.user_uuid.clone(),
            new_uuid: CompanyUuid::from(UuidType::from(Id::generate())),
            company_name: input.company_name.clone(),
            currency: input.currency.clone(),
        }
    }
}
