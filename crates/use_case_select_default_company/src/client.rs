use anyhow::Result;
use kernel::new_types::BranchUuid;
use kernel::new_types::CompanyUuid;
use kernel::new_types::UserUuid;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;
use use_case_get_companies_and_branches::domain::CompanyWithBranches;
use use_case_get_companies_and_branches::domain::Error;
use use_case_get_companies_and_branches::domain::Input;
use use_case_get_companies_and_branches::domain::Ok;
use utility::cache::ResourceName;
use utility::cache::new_resource_name;
use utility::process_manager::ProcessId;
use utility::ui_effect::MessageTrait;
use utility::ui_effect::UiContext;
use utility::ui_orchestration::GenricAsyncState;
use utility::ui_orchestration::UseCaseClient;
use utility::ui_orchestration::handle_refresh;
use utility::ui_orchestration::spawn_listener;
use utility_ui::domain::HashimSignal;

const RESOURCES_NAME_TO_LISTEN: &[ResourceName] = &[
    new_resource_name("companies"),
    new_resource_name("branches"),
];

pub(crate) type AsyncState = GenricAsyncState<AsyncInput, Ok, Error>;

pub trait GlobalModel {
    fn user_uuid(&self) -> Option<UserUuid>;
    fn selected_company_uuid(&self) -> impl HashimSignal<Option<CompanyUuid>>;
    fn selected_company_branch_uuid(&self) -> impl HashimSignal<Option<BranchUuid>>;
}

pub trait LocalModel {
    fn async_state(&self) -> impl HashimSignal<AsyncState>;
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AsyncInput {
    pub user_uuid: UserUuid,
}

pub fn list_of_companies_and_branches(local_model: &impl LocalModel) -> Vec<CompanyWithBranches> {
    match local_model.async_state().read() {
        AsyncState::Success { ok, .. } => ok.companies,
        _ => Vec::new(),
    }
}

pub fn list_of_companies(local_model: &impl LocalModel) -> Vec<(CompanyUuid, String)> {
    match local_model.async_state().read() {
        AsyncState::Success { ok, .. } => flat_companies(&ok.companies),
        _ => Vec::new(),
    }
}

pub fn list_of_branches(
    local_model: &impl LocalModel,
    global_model: &impl GlobalModel,
) -> Vec<(BranchUuid, String)> {
    let Some(company_uuid) = global_model.selected_company_uuid().read() else {
        return Vec::new();
    };
    match local_model.async_state().read() {
        AsyncState::Success { ok, .. } => flat_branches_for(&ok.companies, &company_uuid),
        _ => Vec::new(),
    }
}

pub fn selected_company_name(
    local_model: &impl LocalModel,
    global_model: &impl GlobalModel,
) -> Option<String> {
    let uuid = global_model.selected_company_uuid().read()?;
    list_of_companies(local_model)
        .into_iter()
        .find_map(|(c_uuid, c_name)| (c_uuid == uuid).then_some(c_name))
}

pub fn selected_company_branch_name(
    local_model: &impl LocalModel,
    global_model: &impl GlobalModel,
) -> Option<String> {
    let selected_company_branch_uuid = global_model.selected_company_branch_uuid().read();
    let uuid = selected_company_branch_uuid?;
    list_of_branches(local_model, global_model)
        .into_iter()
        .find_map(|(b_uuid, b_name)| (b_uuid == uuid).then_some(b_name))
}

fn flat_companies(companies: &[CompanyWithBranches]) -> Vec<(CompanyUuid, String)> {
    companies
        .iter()
        .map(|c| (c.uuid.clone(), c.name.clone()))
        .collect()
}

fn flat_branches_for(
    companies: &[CompanyWithBranches],
    company_uuid: &CompanyUuid,
) -> Vec<(BranchUuid, String)> {
    companies
        .iter()
        .find(|c| &c.uuid == company_uuid)
        .map(|c| {
            c.branches
                .iter()
                .map(|b| (b.uuid.clone(), b.name.clone()))
                .collect()
        })
        .unwrap_or_default()
}

#[derive(Debug, Clone)]
pub enum Change {
    AsyncState(AsyncState),
    SelectedCompanyUuid(Option<CompanyUuid>),
    SelectedCompanyBranchUuid(Option<BranchUuid>),
}

#[derive(Debug, Clone)]
pub enum Effect {
    Subscribe,
    UnSubscribe,
    Refresh { async_input: AsyncInput },
}

#[derive(Debug, Clone)]
pub enum Intent {
    Subscribe,
    UnSubscribe,
    SelectCompany(CompanyUuid),
    SelectBranch(BranchUuid),
}

#[derive(Debug, Clone)]
pub enum Observe {
    Refresh,
    Result(AsyncState),
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
    match msg {
        Message::Intent(intent) => match intent {
            Intent::Subscribe => {
                let Some(user_uuid) = global_model.user_uuid() else {
                    return (vec![], vec![]);
                };
                let async_input = AsyncInput { user_uuid };

                let change = vec![Change::AsyncState(AsyncState::Loading {
                    input: async_input.clone(),
                })];
                let effect = vec![Effect::Subscribe, Effect::Refresh { async_input }];
                (change, effect)
            }
            Intent::UnSubscribe => {
                let effect = vec![Effect::UnSubscribe];
                (vec![], effect)
            }
            Intent::SelectCompany(uuid) => {
                if global_model.selected_company_uuid().read().as_ref() == Some(&uuid) {
                    return (vec![], vec![]);
                }

                let change = vec![
                    Change::SelectedCompanyUuid(Some(uuid)),
                    Change::SelectedCompanyBranchUuid(None),
                ];
                (change, vec![])
            }
            Intent::SelectBranch(uuid) => {
                if global_model.selected_company_branch_uuid().read().as_ref() == Some(&uuid) {
                    return (vec![], vec![]);
                }

                let change = vec![Change::SelectedCompanyBranchUuid(Some(uuid))];
                (change, vec![])
            }
        },
        Message::Observe(observe) => match observe {
            Observe::Refresh => {
                let Some(user_uuid) = global_model.user_uuid() else {
                    return (vec![], vec![]);
                };
                let async_input = AsyncInput { user_uuid };

                let change = vec![Change::AsyncState(AsyncState::Loading {
                    input: async_input.clone(),
                })];
                let effect = vec![Effect::Refresh { async_input }];
                (change, effect)
            }
            Observe::Result(result) => {
                let change = vec![Change::AsyncState(result)];
                (change, vec![])
            }
        },
    }
}

pub fn apply(msg: Change, local_model: &impl LocalModel, global_model: &impl GlobalModel) {
    match msg {
        Change::AsyncState(i) => local_model.async_state().set(i),
        Change::SelectedCompanyUuid(i) => global_model.selected_company_uuid().set(i),
        Change::SelectedCompanyBranchUuid(i) => global_model.selected_company_branch_uuid().set(i),
    }
}

pub async fn effect(msg: Effect, process_id: ProcessId, context: UiContext) -> Result<()> {
    match msg {
        Effect::Subscribe => {
            handle_subscribe(process_id, context).await?;
        }
        Effect::UnSubscribe => {
            context.aborters.abort(process_id).await?;
        }
        Effect::Refresh { async_input } => {
            handle_refresh::<Wire>(process_id, async_input, context).await?;
        }
    }
    Ok(())
}

async fn handle_subscribe(process_id: ProcessId, context: UiContext) -> Result<()> {
    spawn_listener(
        context,
        RESOURCES_NAME_TO_LISTEN,
        process_id,
        Message::Observe(Observe::Refresh),
    )
    .await
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
        }
    }

    fn msg_success_submit(input: Self::AsyncInput, ok: Self::Ok) -> Self::Message {
        Message::Observe(Observe::Result(AsyncState::Success { input, ok }))
    }

    fn msg_failure(input: Self::AsyncInput, error: Self::Error) -> Self::Message {
        Message::Observe(Observe::Result(AsyncState::Failure { input, error }))
    }

    fn msg_timeout() -> Self::Message {
        unreachable!()
    }

    fn msg_success_check() -> Self::Message {
        Message::Observe(Observe::Result(AsyncState::Idle))
    }
}
