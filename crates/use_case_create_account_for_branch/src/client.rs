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
use utility::ui_effect::Replier;
use utility::ui_effect::UiContext;
use utility::ui_orchestration::GenricAsyncState;
use utility::ui_orchestration::UseCaseClient;
use utility::ui_orchestration::UserConsent;
use utility::ui_orchestration::handle_submit;
use utility::ui_orchestration::handle_subscribe;
use utility::ui_orchestration::handle_unsubscribe;
use utility_ui::domain::Dialog;

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

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct LocalModel {
    process_id: ProcessId,

    pub show_dialog: Dialog,
    pub async_state: AsyncState,

    pub account_name: String,
    pub outflow_type: OutFlowType,
    pub inflow_type: InFlowType,
}

impl LocalModel {
    pub fn clone_user_input(&self) -> Self {
        Self {
            account_name: self.account_name.clone(),
            outflow_type: self.outflow_type.clone(),
            inflow_type: self.inflow_type.clone(),
            ..Default::default()
        }
    }

    pub fn resolved_account_uuid(&self, global_model: &impl GlobalModel) -> Option<AccountUuid> {
        let typed = self.account_name.clone();
        if typed.is_empty() {
            return None;
        }
        global_model
            .list_of_accounts()
            .into_iter()
            .find_map(|(uuid, name)| (name == typed).then_some(uuid))
    }

    pub fn list_of_accounts_to_display(
        &self,
        global_model: &impl GlobalModel,
    ) -> Vec<(AccountUuid, String)> {
        let typed = self.account_name.clone();
        let list = global_model.list_of_accounts();
        if typed.is_empty() {
            return list;
        }
        select_strings(list, typed, |a| a.1.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct AsyncInput {
    pub user_uuid: UserUuid,
    pub belong_to_account: AccountUuid,
    pub belong_to_company_branch: BranchUuid,
    pub outflow_type: OutFlowType,
    pub inflow_type: InFlowType,
}

#[derive(Debug, Clone)]
pub enum LocalChange {
    ProcessId(ProcessId),
    ShowDialog(Dialog),
    AccountName(String),
    OutflowType(OutFlowType),
    InflowType(InFlowType),
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
    Subscribe(ProcessId),
    UnSubscribe(ProcessId),
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
    local_model: &LocalModel,
    global_model: &impl GlobalModel,
) -> (Vec<LocalChange>, Vec<GlobalChange>, Vec<Effect>) {
    if let Message::Intent(ref a) = msg {
        match a {
            Intent::Consent(_) | Intent::Subscribe | Intent::UnSubscribe => {}
            _ => {
                if local_model.async_state.clone().is_loading() {
                    return (vec![], vec![], vec![]);
                }
            }
        }
    }

    match msg {
        Message::Intent(intent) => match intent {
            Intent::Submit => {
                let Some(user_uuid) = global_model.user_uuid() else {
                    return (vec![], vec![], vec![]);
                };

                let Some(belong_to_company_branch) = global_model.selected_company_branch() else {
                    return (vec![], vec![], vec![]);
                };

                let Some(belong_to_account) = local_model.resolved_account_uuid(global_model)
                else {
                    return (vec![], vec![], vec![]);
                };

                let async_input = AsyncInput {
                    user_uuid,
                    belong_to_account,
                    belong_to_company_branch,
                    outflow_type: local_model.outflow_type,
                    inflow_type: local_model.inflow_type,
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
                        let Some(belong_to_company_branch) = global_model.selected_company_branch()
                        else {
                            return (vec![], vec![], vec![]);
                        };
                        let Some(belong_to_account) =
                            local_model.resolved_account_uuid(global_model)
                        else {
                            return (vec![], vec![], vec![]);
                        };
                        let async_input = AsyncInput {
                            user_uuid,
                            belong_to_account,
                            belong_to_company_branch,
                            outflow_type: local_model.outflow_type,
                            inflow_type: local_model.inflow_type,
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
            Intent::Clean => {
                let local_change = vec![
                    LocalChange::ShowDialog(Default::default()),
                    LocalChange::AccountName(Default::default()),
                    LocalChange::OutflowType(Default::default()),
                    LocalChange::InflowType(Default::default()),
                    LocalChange::AsyncState(AsyncState::Idle),
                ];
                (local_change, vec![], vec![])
            }
            Intent::AccountName(v) => {
                let local_change = vec![LocalChange::AccountName(v)];
                (local_change, vec![], vec![])
            }
            Intent::SelectedAccount(idx) => {
                let list = local_model.list_of_accounts_to_display(global_model);
                match list.get(idx) {
                    Some((_, name)) => {
                        (vec![LocalChange::AccountName(name.clone())], vec![], vec![])
                    }
                    None => (vec![], vec![], vec![]),
                }
            }
            Intent::OutflowType(v) => {
                let local_change = vec![LocalChange::OutflowType(v)];
                (local_change, vec![], vec![])
            }
            Intent::InflowType(v) => {
                let local_change = vec![LocalChange::InflowType(v)];
                (local_change, vec![], vec![])
            }
            Intent::Subscribe => {
                let process_id = ProcessId::default();

                let local_change = vec![LocalChange::ProcessId(process_id)];
                let effect = vec![Effect::Subscribe(process_id)];
                (local_change, vec![], effect)
            }
            Intent::UnSubscribe => {
                let effect = vec![Effect::UnSubscribe(local_model.process_id)];
                (vec![], vec![], effect)
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
                    AsyncState::Success { .. } => vec![
                        LocalChange::ShowDialog(Default::default()),
                        LocalChange::AccountName(Default::default()),
                        LocalChange::OutflowType(Default::default()),
                        LocalChange::InflowType(Default::default()),
                        LocalChange::AsyncState(result),
                    ],
                    _ => vec![
                        LocalChange::ShowDialog(Default::default()),
                        LocalChange::AsyncState(result),
                    ],
                };
                (local_change, vec![], vec![])
            }
            Observe::Refresh => (vec![], vec![], vec![]),
        },
    }
}

pub fn local_apply(msg: LocalChange, local_model: &mut LocalModel) {
    match msg {
        LocalChange::ShowDialog(i) => local_model.show_dialog = i,
        LocalChange::AccountName(i) => local_model.account_name = i,
        LocalChange::OutflowType(i) => local_model.outflow_type = i,
        LocalChange::InflowType(i) => local_model.inflow_type = i,
        LocalChange::AsyncState(i) => local_model.async_state = i,
        LocalChange::ProcessId(i) => local_model.process_id = i,
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
        Effect::Subscribe(process_id) => {
            handle_subscribe::<Wire>(replier, process_id, context, RESOURCES_NAME_TO_LISTEN)
                .await?;
        }
        Effect::UnSubscribe(process_id) => {
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

    fn msg_failure_submit(input: Self::AsyncInput, error: Self::Error) -> Self::Message {
        Message::Observe(Observe::Result(AsyncState::Failure { input, error }))
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
