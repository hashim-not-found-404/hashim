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
use utility::ui_effect::Replier;
use utility::ui_effect::UiContext;
use utility::ui_orchestration::GenricAsyncState;
use utility::ui_orchestration::UseCaseClient;
use utility::ui_orchestration::UserConsent;
use utility::ui_orchestration::handle_check;
use utility::ui_orchestration::handle_submit;
use utility::ui_orchestration::handle_subscribe;
use utility::ui_orchestration::handle_unsubscribe;
use utility_ui::domain::Dialog;

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

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct LocalModel {
    process_id: ProcessId,

    pub show_dialog: Dialog,
    pub async_state: AsyncState,

    pub company_name: String,
    pub branch_name: String,
    pub currency: Currency,
    pub location: Location,
}

impl LocalModel {
    pub fn clone_user_input(&self) -> Self {
        Self {
            company_name: self.company_name.clone(),
            branch_name: self.branch_name.clone(),
            currency: self.currency.clone(),
            location: self.location.clone(),
            ..Default::default()
        }
    }

    pub fn error_branch_name(&self) -> Option<BranchNameError> {
        match self.async_state.clone() {
            AsyncState::Failure { error, .. } => error.branch_name,
            _ => None,
        }
    }

    pub fn error_location(&self) -> Option<LocationError> {
        match self.async_state.clone() {
            AsyncState::Failure { error, .. } => error.location,
            _ => None,
        }
    }

    pub fn resolved_company_uuid(&self, global_model: &impl GlobalModel) -> Option<CompanyUuid> {
        let typed = self.company_name.clone();
        if typed.is_empty() {
            return global_model.selected_company();
        }
        global_model
            .list_of_companies()
            .into_iter()
            .find_map(|(uuid, name)| (name == typed).then_some(uuid))
    }

    pub fn company_name_error(&self, global_model: &impl GlobalModel) -> Option<CompanyNameError> {
        let typed = self.company_name.clone();
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
        let typed = self.company_name.clone();
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
    pub company_belong: CompanyUuid,
    pub branch_name: String,
    pub currency: Currency,
    pub location: Location,
}

#[derive(Debug, Clone)]
pub enum LocalChange {
    ProcessId(ProcessId),
    ShowDialog(Dialog),
    CompanyName(String),
    BranchName(String),
    Currency(Currency),
    Location(Location),
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
    local_model: &LocalModel,
    global_model: &impl GlobalModel,
    override_company: Option<CompanyUuid>,
    mutate: impl FnOnce(&mut AsyncInput),
) -> Vec<Effect> {
    let Some(user_uuid) = global_model.user_uuid() else {
        return vec![];
    };
    let company_belong =
        override_company.or_else(|| local_model.resolved_company_uuid(global_model));
    let Some(company_belong) = company_belong else {
        return vec![];
    };
    let mut a = AsyncInput {
        user_uuid,
        company_belong,
        branch_name: local_model.branch_name.clone(),
        currency: local_model.currency.clone(),
        location: local_model.location.clone(),
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

                let Some(company_belong) = local_model.resolved_company_uuid(global_model) else {
                    return (vec![], vec![], vec![]);
                };

                let async_input = AsyncInput {
                    user_uuid,
                    company_belong,
                    branch_name: local_model.branch_name.clone(),
                    currency: local_model.currency.clone(),
                    location: local_model.location.clone(),
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
                        let Some(company_belong) = local_model.resolved_company_uuid(global_model)
                        else {
                            return (vec![], vec![], vec![]);
                        };
                        let async_input = AsyncInput {
                            user_uuid,
                            company_belong,
                            branch_name: local_model.branch_name.clone(),
                            currency: local_model.currency.clone(),
                            location: local_model.location.clone(),
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
                    LocalChange::CompanyName(Default::default()),
                    LocalChange::BranchName(Default::default()),
                    LocalChange::Currency(Default::default()),
                    LocalChange::Location(Default::default()),
                    LocalChange::AsyncState(AsyncState::Idle),
                ];
                (local_change, vec![], vec![])
            }
            Intent::CompanyName(v) => {
                let local_change = vec![LocalChange::CompanyName(v.clone())];
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
                        let local_change = vec![LocalChange::CompanyName(company_name.clone())];
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
            Intent::BranchName(v) => {
                let local_change = vec![LocalChange::BranchName(v.clone())];
                let effect = build_check_effect(local_model, global_model, None, |a| {
                    a.branch_name = v;
                });
                (local_change, vec![], effect)
            }
            Intent::Currency(v) => {
                let local_change = vec![LocalChange::Currency(v.clone())];
                let effect = build_check_effect(local_model, global_model, None, |a| {
                    a.currency = v;
                });
                (local_change, vec![], effect)
            }
            Intent::Latitude(v) => {
                let mut loc = local_model.location.clone();
                loc.latitude = v;
                let loc_for_effect = loc.clone();
                let local_change = vec![LocalChange::Location(loc)];
                let effect = build_check_effect(local_model, global_model, None, |a| {
                    a.location = loc_for_effect;
                });
                (local_change, vec![], effect)
            }
            Intent::Longitude(v) => {
                let mut loc = local_model.location.clone();
                loc.longitude = v;
                let loc_for_effect = loc.clone();
                let local_change = vec![LocalChange::Location(loc)];
                let effect = build_check_effect(local_model, global_model, None, |a| {
                    a.location = loc_for_effect;
                });
                (local_change, vec![], effect)
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
                    AsyncState::Success { .. } => {
                        vec![
                            LocalChange::ShowDialog(Default::default()),
                            LocalChange::CompanyName(Default::default()),
                            LocalChange::BranchName(Default::default()),
                            LocalChange::Currency(Default::default()),
                            LocalChange::Location(Default::default()),
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
        LocalChange::CompanyName(i) => local_model.company_name = i,
        LocalChange::BranchName(i) => local_model.branch_name = i,
        LocalChange::Currency(i) => local_model.currency = i,
        LocalChange::Location(i) => local_model.location = i,
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
            new_uuid: BranchUuid::from(UuidType::from(Id::generate())),
            company_belong: input.company_belong.clone(),
            branch_name: input.branch_name.clone(),
            currency: input.currency.clone(),
            location: input.location.clone(),
        }
    }
}
