use crate::domain::Error;
use crate::domain::Input;
use crate::domain::Ok;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use accounting_engine::accounting_stuff::InFlowType;
use accounting_engine::accounting_stuff::OutFlowType;
use anyhow::Result;
use infrastructure::row_id::Id;
use infrastructure::row_id::RowId;
use infrastructure::runtime::Rt;
use infrastructure::runtime::Runtime;
use kernel::client::Cache;
use kernel::new_types::AccountForBranchUuid;
use kernel::new_types::AccountUuid;
use kernel::new_types::BranchUuid;
use kernel::new_types::UserUuid;
use kernel::new_types::UuidType;
use kernel::types::DatabaseRead;
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
use utility::ui_orchestration::handle_submit;
use utility::ui_orchestration::handle_subscribe;
use utility::ui_orchestration::handle_unsubscribe;
use utility_ui::domain::Dialog;
use utility_ui::domain::HashimSignal;

const RESOURCES_NAME_TO_POKE: &[ResourceName] = &[new_resource_name("accounts_for_branch")];
const RESOURCES_NAME_TO_LISTEN: &[ResourceName] = &[
    new_resource_name("accounts"),
    new_resource_name("accounts_for_branch"),
];

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
    fn selected_company_branch(&self) -> Option<BranchUuid>;
    fn list_of_accounts(&self) -> Vec<(AccountUuid, String)>; // this should be fetched from this use case , not get it from the model
}

pub trait LocalModel {
    fn show_dialog(&self) -> impl HashimSignal<Dialog>;
    fn account_name(&self) -> impl HashimSignal<String>;
    fn outflow_type(&self) -> impl HashimSignal<OutFlowType>;
    fn inflow_type(&self) -> impl HashimSignal<InFlowType>;
    fn async_state(&self) -> impl HashimSignal<AsyncState>;
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AsyncInput {
    pub user_uuid: UserUuid,
    pub belong_to_account: AccountUuid,
    pub belong_to_company_branch: BranchUuid,
    pub outflow_type: OutFlowType,
    pub inflow_type: InFlowType,
}

pub fn resolved_account_uuid(
    local_model: &impl LocalModel,
    global_model: &impl GlobalModel,
) -> Option<AccountUuid> {
    let typed = local_model.account_name().read();
    if typed.is_empty() {
        return None;
    }
    global_model
        .list_of_accounts()
        .into_iter()
        .find_map(|(uuid, name)| (name == typed).then_some(uuid))
}

pub fn list_of_accounts_to_display(
    local_model: &impl LocalModel,
    global_model: &impl GlobalModel,
) -> Vec<(AccountUuid, String)> {
    let typed = local_model.account_name().read();
    let list = global_model.list_of_accounts();
    if typed.is_empty() {
        return list;
    }
    select_strings(list, typed, |a| a.1.as_str())
}

#[derive(Debug, Clone)]
pub enum Change {
    ShowDialog(Dialog),
    AccountName(String),
    OutflowType(OutFlowType),
    InflowType(InFlowType),
    AsyncState(AsyncState),
}

#[derive(Debug, Clone)]
pub enum Effect {
    SpawnTimer,
    Submit {
        async_input: AsyncInput,
        is_to_server: bool,
    },
    Subscribe,
    UnSubscribe,
}

#[derive(Debug, Clone)]
pub enum Intent {
    Submit,
    Consent(UserConsent),
    Clean,
    AccountName(String),
    SelectedAccount(usize),
    OutflowType(OutFlowType),
    InflowType(InFlowType),
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

                let Some(belong_to_company_branch) = global_model.selected_company_branch() else {
                    return (vec![], vec![]);
                };

                let Some(belong_to_account) = resolved_account_uuid(local_model, global_model)
                else {
                    return (vec![], vec![]);
                };

                let async_input = AsyncInput {
                    user_uuid,
                    belong_to_account,
                    belong_to_company_branch,
                    outflow_type: local_model.outflow_type().read(),
                    inflow_type: local_model.inflow_type().read(),
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
                        let Some(belong_to_company_branch) = global_model.selected_company_branch()
                        else {
                            return (vec![], vec![]);
                        };
                        let Some(belong_to_account) =
                            resolved_account_uuid(local_model, global_model)
                        else {
                            return (vec![], vec![]);
                        };
                        let async_input = AsyncInput {
                            user_uuid,
                            belong_to_account,
                            belong_to_company_branch,
                            outflow_type: local_model.outflow_type().read(),
                            inflow_type: local_model.inflow_type().read(),
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
                    Change::AccountName(Default::default()),
                    Change::OutflowType(Default::default()),
                    Change::InflowType(Default::default()),
                    Change::AsyncState(AsyncState::Idle),
                ];
                (change, vec![])
            }
            Intent::AccountName(v) => {
                let change = vec![Change::AccountName(v)];
                (change, vec![])
            }
            Intent::SelectedAccount(idx) => {
                let list = list_of_accounts_to_display(local_model, global_model);
                match list.get(idx) {
                    Some((_, name)) => (vec![Change::AccountName(name.clone())], vec![]),
                    None => (vec![], vec![]),
                }
            }
            Intent::OutflowType(v) => {
                let change = vec![Change::OutflowType(v)];
                (change, vec![])
            }
            Intent::InflowType(v) => {
                let change = vec![Change::InflowType(v)];
                (change, vec![])
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
                    AsyncState::Success { .. } => vec![
                        Change::ShowDialog(Default::default()),
                        Change::AccountName(Default::default()),
                        Change::OutflowType(Default::default()),
                        Change::InflowType(Default::default()),
                        Change::AsyncState(result),
                    ],
                    _ => vec![
                        Change::ShowDialog(Default::default()),
                        Change::AsyncState(result),
                    ],
                };
                (change, vec![])
            }
            Observe::Refresh => (vec![], vec![]),
        },
    }
}

pub fn apply(msg: Change, local_model: &impl LocalModel, _: &impl GlobalModel) {
    match msg {
        Change::ShowDialog(i) => local_model.show_dialog().set(i),
        Change::AccountName(i) => local_model.account_name().set(i),
        Change::OutflowType(i) => local_model.outflow_type().set(i),
        Change::InflowType(i) => local_model.inflow_type().set(i),
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

    fn msg_refresh() -> Self::Message {
        Message::Observe(Observe::Refresh)
    }

    fn build_input(input: &Self::AsyncInput) -> Self::Input {
        Input {
            user_uuid: input.user_uuid.clone(),
            new_uuid: AccountForBranchUuid::from(UuidType::from(Id::generate())),
            belong_to_account: input.belong_to_account.clone(),
            belong_to_company_branch: input.belong_to_company_branch.clone(),
            outflow_type: input.outflow_type,
            inflow_type: input.inflow_type,
        }
    }
}
