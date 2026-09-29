use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use kernel::new_types::BranchUuid;
use kernel::new_types::CompanyUuid;
use kernel::new_types::UserUuid;
use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;
use use_case_get_companies_and_branches::client::USE_CASE_NAME;
use use_case_get_companies_and_branches::domain::CompanyWithBranches;
use use_case_get_companies_and_branches::domain::Error;
use use_case_get_companies_and_branches::domain::Input;
use use_case_get_companies_and_branches::domain::MyResult;
use use_case_get_companies_and_branches::domain::Ok;
use utility::types::ReadAndSet;
use utility::ui_effect::MessageTrait;
use utility::ui_effect::Model;
use utility::ui_effect::PageId;
use utility::ui_effect::UiContext;
use utility::ui_orchestration::spawn_listener;
use utility_ui::domain::HashimSignal;

#[derive(Debug, Clone)]
pub enum Message {
    Subscribe,
    UnSubscribe,
    SelectCompany(CompanyUuid),
    SelectBranch(BranchUuid),
}

impl MessageTrait for Message {}

pub trait GlobalModel: Model + 'static {
    fn user_uuid(&self) -> impl HashimSignal<Option<UserUuid>>;
    fn selected_company_uuid(&self) -> impl HashimSignal<Option<CompanyUuid>>;
    fn selected_company_name(&self) -> impl HashimSignal<Option<String>>;
    fn selected_company_branch_uuid(&self) -> impl HashimSignal<Option<BranchUuid>>;
    fn selected_company_branch_name(&self) -> impl HashimSignal<Option<String>>;
}

pub trait LocalModel: 'static {
    fn page_id(&self) -> impl ReadAndSet<Option<PageId>>;
    fn list_of_companies_and_branches(&self) -> impl HashimSignal<Vec<CompanyWithBranches>>;
    fn list_of_companies(&self) -> impl HashimSignal<Vec<(CompanyUuid, String)>>;
    fn list_of_branches(&self) -> impl HashimSignal<Vec<(BranchUuid, String)>>;
    fn is_loading(&self) -> impl HashimSignal<bool>;
}

fn apply_on_the_model(
    output: &MyResult,
    local_model: Arc<impl LocalModel>,
    global_model: Arc<impl GlobalModel>,
) {
    local_model.is_loading().set(false);

    match output {
        Ok(ok) => {
            let companies = ok.companies.clone();
            local_model
                .list_of_companies_and_branches()
                .set(companies.clone());
            local_model
                .list_of_companies()
                .set(flat_companies(&companies));
            if let Some(company_uuid) = global_model.selected_company_uuid().read() {
                local_model
                    .list_of_branches()
                    .set(flat_branches_for(&companies, &company_uuid));
            }
        }
        Err(_) => {
            local_model.list_of_companies_and_branches().reset();
            local_model.list_of_companies().reset();
            local_model.list_of_branches().reset();
        }
    }
}

pub async fn update_generic(
    message: Message,
    local_model: Arc<impl LocalModel>,
    context: UiContext<impl GlobalModel>,
) -> Result<()> {
    match message {
        Message::Subscribe => handle_subscribe(local_model, context)?,
        Message::UnSubscribe => {
            context.aborters.abort(
                local_model
                    .page_id()
                    .read()
                    .context("there is no page id to abort")?,
            );
        }
        Message::SelectCompany(uuid) => {
            if context.model.selected_company_uuid().read().as_ref() == Some(&uuid) {
                return Ok(());
            }

            let name = local_model
                .list_of_companies()
                .read()
                .into_iter()
                .find_map(|(c_uuid, c_name)| (c_uuid == uuid).then_some(c_name))
                .context("selected company is not in the list")?;

            context
                .model
                .selected_company_uuid()
                .set(Some(uuid.clone()));
            context.model.selected_company_name().set(Some(name));

            context.model.selected_company_branch_uuid().reset();
            context.model.selected_company_branch_name().reset();

            let branches =
                flat_branches_for(&local_model.list_of_companies_and_branches().read(), &uuid);
            local_model.list_of_branches().set(branches);
        }

        Message::SelectBranch(uuid) => {
            if context.model.selected_company_branch_uuid().read().as_ref() == Some(&uuid) {
                return Ok(());
            }

            let name = local_model
                .list_of_branches()
                .read()
                .into_iter()
                .find_map(|(b_uuid, b_name)| (b_uuid == uuid).then_some(b_name))
                .context("selected branch is not in the list")?;

            context.model.selected_company_branch_uuid().set(Some(uuid));
            context.model.selected_company_branch_name().set(Some(name));
        }
    }

    Ok(())
}

fn handle_subscribe(
    local_model: Arc<impl LocalModel>,
    context: UiContext<impl GlobalModel>,
) -> Result<()> {
    let local_model1 = local_model.clone();

    let aborter = spawn_listener(
        context.sender_to_error,
        context.cache,
        &[USE_CASE_NAME],
        Arc::new(Input {
            user_uuid: context
                .model
                .user_uuid()
                .read()
                .context("user uuid not found")?,
        }),
        move |data| {
            let result = match data {
                Ok(ok) => {
                    let a = ok;
                    let a: Arc<dyn Any> = a;
                    let a: &Ok = a.downcast_ref().context("downcast error")?;
                    let a: Ok = a.clone();
                    Ok(a)
                }
                Err(err) => {
                    let a = err;
                    let a: Box<dyn Any> = a;
                    let a: Box<Error> = a.downcast().map_err(|_| anyhow!("downcast error"))?;
                    let a: Error = a.as_ref().clone();
                    Err(a)
                }
            };

            apply_on_the_model(&result, local_model1.clone(), context.model.clone());

            Ok(())
        },
    );
    let a = context.aborters.register(aborter);
    local_model.page_id().put(Some(a));
    Ok(())
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
