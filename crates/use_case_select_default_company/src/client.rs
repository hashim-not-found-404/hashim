use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use infrastructure::actors::Receiver;
use kernel::new_types::BranchUuid;
use kernel::new_types::CompanyUuid;
use kernel::new_types::UserUuid;
use serde::Deserialize;
use serde::Serialize;
use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;
use use_case_get_companies_and_branches::domain::CompanyWithBranches;
use use_case_get_companies_and_branches::domain::Error;
use use_case_get_companies_and_branches::domain::Input;
use use_case_get_companies_and_branches::domain::Ok;
use utility::cache::CachingStrategy;
use utility::cache::ResourceName;
use utility::cache::Response;
use utility::cache::TypeOperationClientInput;
use utility::cache::new_resource_name;
use utility::dtos::TxnNumber;
use utility::process_manager::ProcessId;
use utility::ui_effect::MessageTrait;
use utility::ui_effect::UiContext;
use utility::ui_orchestration::spawn_listener;
use utility_ui::domain::HashimSignal;

const RESOURCES_NAME_TO_LISTEN: &[ResourceName] = &[
    new_resource_name("companies"),
    new_resource_name("branches"),
];

pub trait GlobalModel {
    fn user_uuid(&self) -> Option<UserUuid>;
    fn selected_company_uuid(&self) -> impl HashimSignal<Option<CompanyUuid>>;
    fn selected_company_branch_uuid(&self) -> impl HashimSignal<Option<BranchUuid>>;
}

pub trait LocalModel {
    fn async_state(&self) -> impl HashimSignal<AsyncState>;
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub enum AsyncState {
    #[default]
    Idle,
    Loading {
        input: AsyncInput,
    },
    Success {
        input: AsyncInput,
        ok: Ok,
    },
    Failure {
        input: AsyncInput,
        error: Error,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AsyncInput {
    pub user_uuid: UserUuid,
}

pub fn is_loading(local_model: &impl LocalModel) -> bool {
    matches!(local_model.async_state().read(), AsyncState::Loading { .. })
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
    Subscribe {
        process_id: ProcessId,
    },
    UnSubscribe {
        process_id: ProcessId,
    },
    Refresh {
        process_id: ProcessId,
        async_input: AsyncInput,
    },
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

pub fn reduce(
    msg: Message,
    process_id: ProcessId,
    local_model: &impl LocalModel,
    global_model: &impl GlobalModel,
) -> Result<(Vec<Change>, Vec<Effect>)> {
    match msg {
        Message::Intent(intent) => match intent {
            Intent::Subscribe => {
                let Some(user_uuid) = global_model.user_uuid() else {
                    return Ok((vec![], vec![]));
                };
                let async_input = AsyncInput { user_uuid };

                let change = vec![Change::AsyncState(AsyncState::Loading {
                    input: async_input.clone(),
                })];
                let effect = vec![
                    Effect::Subscribe { process_id },
                    Effect::Refresh {
                        process_id,
                        async_input,
                    },
                ];
                Ok((change, effect))
            }
            Intent::UnSubscribe => {
                let effect = vec![Effect::UnSubscribe { process_id }];
                Ok((vec![], effect))
            }
            Intent::SelectCompany(uuid) => {
                if global_model.selected_company_uuid().read().as_ref() == Some(&uuid) {
                    return Ok((vec![], vec![]));
                }

                let change = vec![
                    Change::SelectedCompanyUuid(Some(uuid)),
                    Change::SelectedCompanyBranchUuid(None),
                ];
                Ok((change, vec![]))
            }
            Intent::SelectBranch(uuid) => {
                if global_model.selected_company_branch_uuid().read().as_ref() == Some(&uuid) {
                    return Ok((vec![], vec![]));
                }

                let change = vec![Change::SelectedCompanyBranchUuid(Some(uuid))];
                Ok((change, vec![]))
            }
        },
        Message::Observe(observe) => match observe {
            Observe::Refresh => {
                let Some(user_uuid) = global_model.user_uuid() else {
                    return Ok((vec![], vec![]));
                };
                let async_input = AsyncInput { user_uuid };

                let change = vec![Change::AsyncState(AsyncState::Loading {
                    input: async_input.clone(),
                })];
                let effect = vec![Effect::Refresh {
                    process_id,
                    async_input,
                }];
                Ok((change, effect))
            }
            Observe::Result(result) => {
                let change = vec![Change::AsyncState(result)];
                Ok((change, vec![]))
            }
        },
    }
}

pub fn update(msg: Change, local_model: &impl LocalModel, global_model: &impl GlobalModel) {
    match msg {
        Change::AsyncState(i) => local_model.async_state().set(i),
        Change::SelectedCompanyUuid(i) => global_model.selected_company_uuid().set(i),
        Change::SelectedCompanyBranchUuid(i) => global_model.selected_company_branch_uuid().set(i),
    }
}

pub async fn effect(msg: Effect, context: UiContext) -> Result<()> {
    match msg {
        Effect::Subscribe { process_id } => {
            handle_subscribe(process_id, context).await?;
        }
        Effect::UnSubscribe { process_id } => {
            context.aborters.abort(process_id);
        }
        Effect::Refresh {
            process_id,
            async_input,
        } => {
            handle_refresh(process_id, async_input, context).await?;
        }
    }
    Ok(())
}

async fn handle_refresh(
    process_id: ProcessId,
    async_input: AsyncInput,
    context: UiContext,
) -> Result<()> {
    let sender_to_commander = context.sender_to_commander.clone();
    let mut cache = context.cache.clone();

    let input: TypeOperationClientInput = Arc::new(Input {
        user_uuid: async_input.user_uuid.clone(),
    });

    let mut receiver_to_response = cache
        .send_to_cache_actor(CachingStrategy::ReadCacheOnly, TxnNumber::default(), input)
        .await?;

    match receiver_to_response.recv().await? {
        Response::CloseTheChannel => {}
        Response::ServerCannotBeReached => {}
        Response::Data { data, .. } => {
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

            let async_state = match result {
                Ok(a) => AsyncState::Success {
                    input: async_input,
                    ok: a,
                },
                Err(a) => AsyncState::Failure {
                    input: async_input,
                    error: a,
                },
            };

            sender_to_commander.send(process_id, Message::Observe(Observe::Result(async_state)));
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
