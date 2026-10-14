use crate::domain::AccountNameError;
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
use kernel::new_types::AccountUuid;
use kernel::new_types::CompanyUuid;
use kernel::new_types::UserUuid;
use kernel::new_types::UuidType;
use kernel::types::DatabaseRead;
use kernel::types::MyErrorTrait;
use kernel::types::RowIdError;
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
use utility::ui_orchestration::CheckState;
use utility::ui_orchestration::GenricAsyncState;
use utility::ui_orchestration::UseCaseClient;
use utility::ui_orchestration::UserConsent;
use utility::ui_orchestration::handle_check;
use utility::ui_orchestration::handle_submit;
use utility::ui_orchestration::handle_subscribe;
use utility::ui_orchestration::handle_unsubscribe;
use utility_ui::domain::Dialog;

const RESOURCES_NAME_TO_POKE: &[ResourceName] = &[new_resource_name("accounts")];
const RESOURCES_NAME_TO_LISTEN: &[ResourceName] = &[new_resource_name("accounts")];

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
pub(crate) type CheckStateLocal = CheckState<AsyncInput, Ok, Error>;

pub trait GlobalModel {
    fn user_uuid(&self) -> Option<UserUuid>;
    fn selected_company(&self) -> Option<CompanyUuid>;
    fn list_of_companies(&self) -> Vec<(CompanyUuid, String)>;
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct LocalModel {
    process_id: ProcessId,

    pub show_dialog: Dialog,
    pub check_state: CheckStateLocal,
    pub async_state: AsyncState,

    pub is_debit: bool,
    pub is_permanent_account: bool,
    pub account_name: String,
    pub unit_of_measurement_of_quantity: String,
    pub selected_company_name: String,
}

impl LocalModel {
    pub fn clone_user_input(&self) -> Self {
        Self {
            is_debit: self.is_debit,
            is_permanent_account: self.is_permanent_account,
            account_name: self.account_name.clone(),
            unit_of_measurement_of_quantity: self.unit_of_measurement_of_quantity.clone(),
            selected_company_name: self.selected_company_name.clone(),
            ..Default::default()
        }
    }

    pub fn is_submit_ok(&self) -> bool {
        matches!(self.async_state, AsyncState::Success { .. })
    }

    pub fn error_account_name(&self) -> Option<AccountNameError> {
        if let CheckState::Failure { error, .. } = &self.check_state
            && let Some(e) = &error.account_name
        {
            return Some(e.clone());
        }
        if let AsyncState::Failure { error, .. } = &self.async_state
            && let Some(e) = &error.account_name
        {
            return Some(e.clone());
        }
        None
    }

    pub fn resolved_company_uuid(&self, global_model: &impl GlobalModel) -> Option<CompanyUuid> {
        let typed = self.selected_company_name.clone();
        if typed.is_empty() {
            return global_model.selected_company();
        }
        global_model
            .list_of_companies()
            .into_iter()
            .find_map(|(uuid, name)| (name == typed).then_some(uuid))
    }

    pub fn error_company_name(&self, global_model: &impl GlobalModel) -> Option<CompanyNameError> {
        let belong_to_company_err = match &self.check_state {
            CheckState::Failure { error, .. } => error.belong_to_company.clone(),
            _ => match &self.async_state {
                AsyncState::Failure { error, .. } => error.belong_to_company.clone(),
                _ => None,
            },
        };

        if let Some(e) = belong_to_company_err {
            return match e {
                RowIdError::Invalid | RowIdError::NotExist => Some(CompanyNameError::NotExist),
                RowIdError::Duplicated => None,
            };
        }

        let typed = self.selected_company_name.clone();
        if typed.is_empty() {
            if global_model.selected_company().is_none() {
                return Some(CompanyNameError::NotSelected);
            }
            return None;
        }
        if self.resolved_company_uuid(global_model).is_some() {
            None
        } else {
            Some(CompanyNameError::NotExist)
        }
    }

    pub fn list_of_companies_to_display(
        &self,
        global_model: &impl GlobalModel,
    ) -> Vec<(CompanyUuid, String)> {
        let typed = self.selected_company_name.clone();
        let list = global_model.list_of_companies();
        if typed.is_empty() {
            return list;
        }
        select_strings(list, typed, |a| a.1.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum CompanyNameError {
    NotSelected,
    NotExist,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct AsyncInput {
    pub user_uuid: UserUuid,
    pub is_debit: bool,
    pub is_permanent_account: bool,
    pub account_name: String,
    pub unit_of_measurement_of_quantity: String,
    pub belong_to_company: CompanyUuid,
}

#[derive(Debug, Clone)]
pub enum LocalChange {
    ShowDialog(Dialog),
    IsDebit(bool),
    IsPermanentAccount(bool),
    AccountName(String),
    UnitOfMeasurementOfQuantity(String),
    SelectedCompanyName(String),
    CheckState(CheckStateLocal),
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
    Subscribe(ProcessId),
    UnSubscribe(ProcessId),
}

#[derive(Debug, Clone)]
pub enum Intent {
    Submit,
    Consent(UserConsent),
    Clean,
    IsDebit(bool),
    IsPermanentAccount(bool),
    AccountName(String),
    UnitOfMeasurementOfQuantity(String),
    CompanyName(String),
    SelectedCompany(usize),
    Subscribe,
    UnSubscribe,
}

#[derive(Debug, Clone)]
pub enum Observe {
    Timeout,
    Result(AsyncState),
    CheckResult(CheckStateLocal),
    Refresh,
}

#[derive(Debug, Clone)]
pub enum Message {
    Intent(Intent),
    Observe(Observe),
}

impl MessageTrait for Message {}

fn current_check_input(
    local_model: &LocalModel,
    global_model: &impl GlobalModel,
) -> Option<AsyncInput> {
    let user_uuid = global_model.user_uuid()?;
    let belong_to_company = local_model.resolved_company_uuid(global_model)?;
    Some(AsyncInput {
        user_uuid,
        is_debit: local_model.is_debit,
        is_permanent_account: local_model.is_permanent_account,
        account_name: local_model.account_name.clone(),
        unit_of_measurement_of_quantity: local_model.unit_of_measurement_of_quantity.clone(),
        belong_to_company,
    })
}

fn build_check_effect(
    local_model: &LocalModel,
    global_model: &impl GlobalModel,
    override_company: Option<CompanyUuid>,
    mutate: impl FnOnce(&mut AsyncInput),
) -> Vec<Effect> {
    let Some(user_uuid) = global_model.user_uuid() else {
        return vec![];
    };
    let belong_to_company =
        override_company.or_else(|| local_model.resolved_company_uuid(global_model));
    let Some(belong_to_company) = belong_to_company else {
        return vec![];
    };
    let mut a = AsyncInput {
        user_uuid,
        is_debit: local_model.is_debit,
        is_permanent_account: local_model.is_permanent_account,
        account_name: local_model.account_name.clone(),
        unit_of_measurement_of_quantity: local_model.unit_of_measurement_of_quantity.clone(),
        belong_to_company,
    };
    mutate(&mut a);
    vec![Effect::Check { async_input: a }]
}

fn is_check_stale(
    result: &CheckStateLocal,
    local_model: &LocalModel,
    global_model: &impl GlobalModel,
) -> bool {
    let input = match result {
        CheckState::Idle => return false,
        CheckState::Success { input, .. } | CheckState::Failure { input, .. } => input,
    };
    current_check_input(local_model, global_model).as_ref() != Some(input)
}

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

                let Some(belong_to_company) = local_model.resolved_company_uuid(global_model)
                else {
                    return (vec![], vec![], vec![]);
                };

                let async_input = AsyncInput {
                    user_uuid,
                    is_debit: local_model.is_debit,
                    is_permanent_account: local_model.is_permanent_account,
                    account_name: local_model.account_name.clone(),
                    unit_of_measurement_of_quantity: local_model
                        .unit_of_measurement_of_quantity
                        .clone(),
                    belong_to_company,
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
                        let Some(belong_to_company) =
                            local_model.resolved_company_uuid(global_model)
                        else {
                            return (vec![], vec![], vec![]);
                        };
                        let async_input = AsyncInput {
                            user_uuid,
                            is_debit: local_model.is_debit,
                            is_permanent_account: local_model.is_permanent_account,
                            account_name: local_model.account_name.clone(),
                            unit_of_measurement_of_quantity: local_model
                                .unit_of_measurement_of_quantity
                                .clone(),
                            belong_to_company,
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
                    LocalChange::IsDebit(Default::default()),
                    LocalChange::IsPermanentAccount(Default::default()),
                    LocalChange::AccountName(Default::default()),
                    LocalChange::UnitOfMeasurementOfQuantity(Default::default()),
                    LocalChange::SelectedCompanyName(Default::default()),
                    LocalChange::CheckState(CheckState::Idle),
                    LocalChange::AsyncState(AsyncState::Idle),
                ];
                (local_change, vec![], vec![])
            }
            Intent::IsDebit(v) => {
                let local_change = vec![
                    LocalChange::IsDebit(v),
                    LocalChange::CheckState(CheckState::Idle),
                ];
                let effect = build_check_effect(local_model, global_model, None, |a| {
                    a.is_debit = v;
                });
                (local_change, vec![], effect)
            }
            Intent::IsPermanentAccount(v) => {
                let local_change = vec![
                    LocalChange::IsPermanentAccount(v),
                    LocalChange::CheckState(CheckState::Idle),
                ];
                let effect = build_check_effect(local_model, global_model, None, |a| {
                    a.is_permanent_account = v;
                });
                (local_change, vec![], effect)
            }
            Intent::AccountName(v) => {
                let local_change = vec![
                    LocalChange::AccountName(v.clone()),
                    LocalChange::CheckState(CheckState::Idle),
                ];
                let effect = build_check_effect(local_model, global_model, None, |a| {
                    a.account_name = v;
                });
                (local_change, vec![], effect)
            }
            Intent::UnitOfMeasurementOfQuantity(v) => {
                let local_change = vec![
                    LocalChange::UnitOfMeasurementOfQuantity(v.clone()),
                    LocalChange::CheckState(CheckState::Idle),
                ];
                let effect = build_check_effect(local_model, global_model, None, |a| {
                    a.unit_of_measurement_of_quantity = v;
                });
                (local_change, vec![], effect)
            }
            Intent::CompanyName(v) => {
                let local_change = vec![
                    LocalChange::SelectedCompanyName(v.clone()),
                    LocalChange::CheckState(CheckState::Idle),
                ];
                let resolved = if v.is_empty() {
                    global_model.selected_company()
                } else {
                    global_model
                        .list_of_companies()
                        .into_iter()
                        .find_map(|(uuid, name)| (name == v).then_some(uuid))
                };
                let effect = build_check_effect(local_model, global_model, resolved, |_| {});
                (local_change, vec![], effect)
            }
            Intent::SelectedCompany(idx) => {
                let list = local_model.list_of_companies_to_display(global_model);
                match list.get(idx) {
                    Some((company_uuid, company_name)) => {
                        let local_change = vec![
                            LocalChange::SelectedCompanyName(company_name.clone()),
                            LocalChange::CheckState(CheckState::Idle),
                        ];
                        let effect = build_check_effect(
                            local_model,
                            global_model,
                            Some(company_uuid.clone()),
                            |_| {},
                        );
                        (local_change, vec![], effect)
                    }
                    None => (vec![], vec![], vec![]),
                }
            }
            Intent::Subscribe => {
                let effect = vec![Effect::Subscribe(local_model.process_id)];
                (vec![], vec![], effect)
            }
            Intent::UnSubscribe => {
                let effect = vec![Effect::UnSubscribe(local_model.process_id)];
                (vec![], vec![], effect)
            }
        },
        Message::Observe(observe) => match observe {
            Observe::Timeout => {
                let change = match local_model.async_state.clone() {
                    AsyncState::Loading { .. } => vec![LocalChange::ShowDialog(Dialog::Show)],
                    _ => vec![],
                };
                (change, vec![], vec![])
            }
            Observe::Result(result) => match result {
                AsyncState::Success { .. } => {
                    let change = vec![
                        LocalChange::ShowDialog(Default::default()),
                        LocalChange::AccountName(Default::default()),
                        LocalChange::IsDebit(Default::default()),
                        LocalChange::IsPermanentAccount(Default::default()),
                        LocalChange::UnitOfMeasurementOfQuantity(Default::default()),
                        LocalChange::SelectedCompanyName(Default::default()),
                        LocalChange::CheckState(CheckState::Idle),
                        LocalChange::AsyncState(result),
                    ];
                    (change, vec![], vec![])
                }
                _ => {
                    let change = vec![
                        LocalChange::ShowDialog(Default::default()),
                        LocalChange::AsyncState(result),
                    ];
                    (change, vec![], vec![])
                }
            },
            Observe::CheckResult(result) => {
                if is_check_stale(&result, local_model, global_model) {
                    return (vec![], vec![], vec![]);
                }
                (vec![LocalChange::CheckState(result)], vec![], vec![])
            }
            Observe::Refresh => {
                let effect = build_check_effect(local_model, global_model, None, |_| {});
                (vec![], vec![], effect)
            }
        },
    }
}

pub fn local_apply(msg: LocalChange, local_model: &mut LocalModel) {
    match msg {
        LocalChange::ShowDialog(i) => local_model.show_dialog = i,
        LocalChange::IsDebit(i) => local_model.is_debit = i,
        LocalChange::IsPermanentAccount(i) => local_model.is_permanent_account = i,
        LocalChange::AccountName(i) => local_model.account_name = i,
        LocalChange::UnitOfMeasurementOfQuantity(i) => {
            local_model.unit_of_measurement_of_quantity = i
        }
        LocalChange::SelectedCompanyName(i) => local_model.selected_company_name = i,
        LocalChange::CheckState(i) => local_model.check_state = i,
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

    fn msg_success_check(input: Self::AsyncInput, ok: Self::Ok) -> Self::Message {
        Message::Observe(Observe::CheckResult(CheckState::Success { input, ok }))
    }

    fn msg_failure_check(input: Self::AsyncInput, error: Self::Error) -> Self::Message {
        Message::Observe(Observe::CheckResult(CheckState::Failure { input, error }))
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
            new_uuid: AccountUuid::from(UuidType::from(Id::generate())),
            is_debit: input.is_debit,
            is_permanent_account: input.is_permanent_account,
            account_name: input.account_name.clone(),
            unit_of_measurement_of_quantity: input.unit_of_measurement_of_quantity.clone(),
            belong_to_company: input.belong_to_company.clone(),
        }
    }
}
