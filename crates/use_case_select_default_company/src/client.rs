use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;
use use_case_get_companies_and_branches::domain::CompanyWithBranches;
use use_case_get_companies_and_branches::domain::Error;
use use_case_get_companies_and_branches::domain::Input;
use use_case_get_companies_and_branches::domain::Ok;
use use_case_prelude_client::*;

const RESOURCES_NAME_TO_LISTEN: &[ResourceName] = &[
    new_resource_name("companies"),
    new_resource_name("branches"),
];

pub(crate) type AsyncState = GenricAsyncState<AsyncInput, Ok, Error>;

pub trait GlobalModel {
    fn user_uuid(&self) -> Option<UserUuid>;
    fn selected_company_uuid(&self) -> Option<CompanyUuid>;
    fn selected_company_branch_uuid(&self) -> Option<BranchUuid>;

    fn set_selected_company_uuid(&mut self, v: Option<CompanyUuid>);
    fn set_selected_company_branch_uuid(&mut self, v: Option<BranchUuid>);
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct LocalModel {
    process_id: ProcessId,

    pub async_state: AsyncState,
}

impl LocalModel {
    pub fn list_of_companies_and_branches(&self) -> Vec<CompanyWithBranches> {
        match self.async_state.clone() {
            AsyncState::Success { ok, .. } => ok.companies,
            _ => Vec::new(),
        }
    }

    pub fn list_of_companies(&self) -> Vec<(CompanyUuid, String)> {
        match self.async_state.clone() {
            AsyncState::Success { ok, .. } => flat_companies(&ok.companies),
            _ => Vec::new(),
        }
    }

    pub fn list_of_branches(&self, global_model: &impl GlobalModel) -> Vec<(BranchUuid, String)> {
        let Some(company_uuid) = global_model.selected_company_uuid() else {
            return Vec::new();
        };
        match self.async_state.clone() {
            AsyncState::Success { ok, .. } => flat_branches_for(&ok.companies, &company_uuid),
            _ => Vec::new(),
        }
    }

    pub fn selected_company_name(&self, global_model: &impl GlobalModel) -> Option<String> {
        let uuid = global_model.selected_company_uuid()?;
        self.list_of_companies()
            .into_iter()
            .find_map(|(c_uuid, c_name)| (c_uuid == uuid).then_some(c_name))
    }

    pub fn selected_company_branch_name(&self, global_model: &impl GlobalModel) -> Option<String> {
        let selected_company_branch_uuid = global_model.selected_company_branch_uuid();
        let uuid = selected_company_branch_uuid?;
        self.list_of_branches(global_model)
            .into_iter()
            .find_map(|(b_uuid, b_name)| (b_uuid == uuid).then_some(b_name))
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct AsyncInput {
    pub user_uuid: UserUuid,
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
pub enum LocalChange {
    AsyncState(AsyncState),
}

#[derive(Debug, Clone)]
pub enum GlobalChange {
    SelectedCompanyUuid(Option<CompanyUuid>),
    SelectedCompanyBranchUuid(Option<BranchUuid>),
}

#[derive(Debug, Clone)]
pub enum Effect {
    Subscribe(ProcessId),
    UnSubscribe(ProcessId),
    Refresh { async_input: AsyncInput },
    FetchFromServer { async_input: AsyncInput },
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
    local_model: &LocalModel,
    global_model: &impl GlobalModel,
) -> (Vec<LocalChange>, Vec<GlobalChange>, Vec<Effect>) {
    match msg {
        Message::Intent(intent) => match intent {
            Intent::Subscribe => {
                let Some(user_uuid) = global_model.user_uuid() else {
                    return (vec![], vec![], vec![]);
                };
                let async_input = AsyncInput { user_uuid };

                let local_change = vec![LocalChange::AsyncState(AsyncState::Loading {
                    input: async_input.clone(),
                })];

                let effect = vec![
                    Effect::Subscribe(local_model.process_id),
                    Effect::FetchFromServer { async_input },
                ];
                (local_change, vec![], effect)
            }
            Intent::UnSubscribe => {
                let effect = vec![Effect::UnSubscribe(local_model.process_id)];
                (vec![], vec![], effect)
            }
            Intent::SelectCompany(uuid) => {
                if global_model.selected_company_uuid().as_ref() == Some(&uuid) {
                    return (vec![], vec![], vec![]);
                }

                let global_change = vec![
                    GlobalChange::SelectedCompanyUuid(Some(uuid)),
                    GlobalChange::SelectedCompanyBranchUuid(None),
                ];
                (vec![], global_change, vec![])
            }
            Intent::SelectBranch(uuid) => {
                if global_model.selected_company_branch_uuid().as_ref() == Some(&uuid) {
                    return (vec![], vec![], vec![]);
                }

                let global_change = vec![GlobalChange::SelectedCompanyBranchUuid(Some(uuid))];
                (vec![], global_change, vec![])
            }
        },
        Message::Observe(observe) => match observe {
            Observe::Refresh => {
                let Some(user_uuid) = global_model.user_uuid() else {
                    return (vec![], vec![], vec![]);
                };
                let async_input = AsyncInput { user_uuid };

                let local_change = vec![LocalChange::AsyncState(AsyncState::Loading {
                    input: async_input.clone(),
                })];
                let effect = vec![Effect::Refresh { async_input }];
                (local_change, vec![], effect)
            }
            Observe::Result(result) => {
                let local_change = vec![LocalChange::AsyncState(result)];
                (local_change, vec![], vec![])
            }
        },
    }
}

pub fn local_apply(msg: LocalChange, local_model: &mut LocalModel) {
    match msg {
        LocalChange::AsyncState(i) => local_model.async_state = i,
    }
}

pub fn global_apply(msg: GlobalChange, global_model: &mut impl GlobalModel) {
    match msg {
        GlobalChange::SelectedCompanyUuid(i) => global_model.set_selected_company_uuid(i),
        GlobalChange::SelectedCompanyBranchUuid(i) => {
            global_model.set_selected_company_branch_uuid(i)
        }
    }
}

pub async fn effect(msg: Effect, replier: Replier, context: UiContext) -> Result<()> {
    match msg {
        Effect::Subscribe(process_id) => {
            handle_subscribe::<Wire>(replier, process_id, context, RESOURCES_NAME_TO_LISTEN)
                .await?;
        }
        Effect::UnSubscribe(process_id) => {
            handle_unsubscribe::<Wire>(process_id, context).await?;
        }
        Effect::Refresh { async_input } => {
            handle_refresh::<Wire>(replier, async_input, context).await?;
        }
        Effect::FetchFromServer { async_input } => {
            use_case_get_companies_and_branches::client::fetch(
                context.cache.clone(),
                async_input.user_uuid,
            )
            .await?;
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

    fn msg_refresh() -> Self::Message {
        Message::Observe(Observe::Refresh)
    }

    fn build_input(input: &Self::AsyncInput) -> Self::Input {
        Input {
            user_uuid: input.user_uuid.clone(),
        }
    }
}
