use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use infrastructure::actors::Receiver;
use kernel::new_types::BranchUuid;
use kernel::new_types::CompanyUuid;
use kernel::new_types::UserUuid;
use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;
use use_case_get_companies_and_branches::domain::CompanyWithBranches;
use use_case_get_companies_and_branches::domain::Error;
use use_case_get_companies_and_branches::domain::Input;
use use_case_get_companies_and_branches::domain::MyResult;
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
    fn selected_company_name(&self) -> impl HashimSignal<Option<String>>;
    fn selected_company_branch_uuid(&self) -> impl HashimSignal<Option<BranchUuid>>;
    fn selected_company_branch_name(&self) -> impl HashimSignal<Option<String>>;
}

pub trait LocalModel {
    fn list_of_companies_and_branches(&self) -> impl HashimSignal<Vec<CompanyWithBranches>>;
    fn list_of_companies(&self) -> impl HashimSignal<Vec<(CompanyUuid, String)>>;
    fn list_of_branches(&self) -> impl HashimSignal<Vec<(BranchUuid, String)>>;
    fn is_loading(&self) -> impl HashimSignal<bool>;
}

#[derive(Debug, Clone)]
pub enum Change {
    IsLoading(bool),
    ListOfCompaniesAndBranches(Vec<CompanyWithBranches>),
    ListOfCompanies(Vec<(CompanyUuid, String)>),
    ListOfBranches(Vec<(BranchUuid, String)>),
    SelectedCompanyUuid(Option<CompanyUuid>),
    SelectedCompanyName(Option<String>),
    SelectedCompanyBranchUuid(Option<BranchUuid>),
    SelectedCompanyBranchName(Option<String>),
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
        user_uuid: UserUuid,
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
    Result(MyResult),
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
                let change = vec![Change::IsLoading(true)];
                let effect = vec![
                    Effect::Subscribe { process_id },
                    Effect::Refresh {
                        process_id,
                        user_uuid,
                    },
                ];
                Ok((change, effect))
            }
            Intent::UnSubscribe => {
                let change = vec![];
                let effect = vec![Effect::UnSubscribe { process_id }];
                Ok((change, effect))
            }
            Intent::SelectCompany(uuid) => {
                if global_model.selected_company_uuid().read().as_ref() == Some(&uuid) {
                    return Ok((vec![], vec![]));
                }

                let Some(name) = local_model
                    .list_of_companies()
                    .read()
                    .into_iter()
                    .find_map(|(c_uuid, c_name)| (c_uuid == uuid).then_some(c_name))
                else {
                    return Ok((vec![], vec![]));
                };

                let branches =
                    flat_branches_for(&local_model.list_of_companies_and_branches().read(), &uuid);

                let change = vec![
                    Change::SelectedCompanyUuid(Some(uuid)),
                    Change::SelectedCompanyName(Some(name)),
                    Change::SelectedCompanyBranchUuid(None),
                    Change::SelectedCompanyBranchName(None),
                    Change::ListOfBranches(branches),
                ];
                Ok((change, vec![]))
            }
            Intent::SelectBranch(uuid) => {
                if global_model.selected_company_branch_uuid().read().as_ref() == Some(&uuid) {
                    return Ok((vec![], vec![]));
                }

                let Some(name) = local_model
                    .list_of_branches()
                    .read()
                    .into_iter()
                    .find_map(|(b_uuid, b_name)| (b_uuid == uuid).then_some(b_name))
                else {
                    return Ok((vec![], vec![]));
                };

                let change = vec![
                    Change::SelectedCompanyBranchUuid(Some(uuid)),
                    Change::SelectedCompanyBranchName(Some(name)),
                ];
                Ok((change, vec![]))
            }
        },
        Message::Observe(observe) => match observe {
            Observe::Result(result) => {
                let mut change = vec![Change::IsLoading(false)];
                match result {
                    Ok(ok) => {
                        let companies = ok.companies.clone();
                        change.push(Change::ListOfCompaniesAndBranches(companies.clone()));
                        change.push(Change::ListOfCompanies(flat_companies(&companies)));

                        if let Some(company_uuid) = global_model.selected_company_uuid().read() {
                            change.push(Change::ListOfBranches(flat_branches_for(
                                &companies,
                                &company_uuid,
                            )));
                        }
                    }
                    Err(_) => {
                        change.push(Change::ListOfCompaniesAndBranches(Vec::new()));
                        change.push(Change::ListOfCompanies(Vec::new()));
                        change.push(Change::ListOfBranches(Vec::new()));
                    }
                }
                Ok((change, vec![]))
            }
            Observe::Refresh => {
                let Some(user_uuid) = global_model.user_uuid() else {
                    return Ok((vec![], vec![]));
                };
                let change = vec![Change::IsLoading(true)];
                let effect = vec![Effect::Refresh {
                    process_id,
                    user_uuid,
                }];
                Ok((change, effect))
            }
        },
    }
}

pub fn update(msg: Change, local_model: &impl LocalModel, global_model: &impl GlobalModel) {
    match msg {
        Change::IsLoading(i) => local_model.is_loading().set(i),
        Change::ListOfCompaniesAndBranches(i) => {
            local_model.list_of_companies_and_branches().set(i)
        }
        Change::ListOfCompanies(i) => local_model.list_of_companies().set(i),
        Change::ListOfBranches(i) => local_model.list_of_branches().set(i),
        Change::SelectedCompanyUuid(i) => global_model.selected_company_uuid().set(i),
        Change::SelectedCompanyName(i) => global_model.selected_company_name().set(i),
        Change::SelectedCompanyBranchUuid(i) => global_model.selected_company_branch_uuid().set(i),
        Change::SelectedCompanyBranchName(i) => global_model.selected_company_branch_name().set(i),
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
            user_uuid,
        } => {
            handle_refresh(process_id, user_uuid, context).await?;
        }
    }
    Ok(())
}

async fn handle_refresh(
    process_id: ProcessId,
    user_uuid: UserUuid,
    context: UiContext,
) -> Result<()> {
    let sender_to_commander = context.sender_to_commander.clone();
    let mut cache = context.cache.clone();

    let input: TypeOperationClientInput = Arc::new(Input { user_uuid });

    let mut receiver_to_response = cache
        .send_to_cache_actor(CachingStrategy::ReadCacheOnly, TxnNumber::default(), input)
        .await?;

    match receiver_to_response.recv().await? {
        Response::CloseTheChannel => {}
        Response::ServerCannotBeReached => {
            sender_to_commander.send(
                process_id,
                Message::Observe(Observe::Result(Err(Error::default()))),
            );
        }
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

            sender_to_commander.send(process_id, Message::Observe(Observe::Result(result)));
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
