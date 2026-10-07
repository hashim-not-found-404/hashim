use crate::domain::BranchNameError;
use crate::domain::Error;
use crate::domain::Input;
use crate::domain::LocationError;
use crate::domain::MyResult;
use crate::domain::Ok;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use infrastructure::actors::Sender;
use infrastructure::row_id::Id;
use infrastructure::row_id::RowId;
use kernel::client::Cache;
use kernel::new_types::BranchUuid;
use kernel::new_types::CompanyUuid;
use kernel::new_types::UserUuid;
use kernel::new_types::UuidType;
use kernel::types::Currency;
use kernel::types::DatabaseRead;
use kernel::types::Location;
use kernel::types::MyErrorTrait;
use std::any::Any;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;
use utility::cache::ResourceName;
use utility::cache::TraitOperationClientError;
use utility::cache::TraitOperationClientInput;
use utility::cache::TraitOperationClientOk;
use utility::cache::TypeOperationClientInput;
use utility::cache::TypeOperationClientResult;
use utility::cache::new_resource_name;
use utility::process_manager::MessageToProcessManager;
use utility::process_manager::ProcessDialog;
use utility::process_manager::ProcessId;
use utility::process_manager::UserConsent;
use utility::tools::select_strings;
use utility::ui_effect::Commander;
use utility::ui_effect::MessageTrait;
use utility::ui_effect::UiContext;
use utility::ui_orchestration::handle_fall_back;
use utility_ui::domain::Dialog;
use utility_ui::domain::HashimSignal;

const RESOURCES_NAME_TO_POKE: &[ResourceName] = &[new_resource_name("branches")];

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
    fn list_of_companies(&self) -> Vec<(CompanyUuid, String)>;
    fn user_uuid(&self) -> Option<UserUuid>;
    fn selected_company(&self) -> impl HashimSignal<Option<CompanyUuid>>;
}

pub trait LocalModel {
    fn show_dialog(&self) -> impl HashimSignal<Dialog>;
    fn is_loading(&self) -> impl HashimSignal<bool>;
    fn list_of_companies_to_display(&self) -> impl HashimSignal<Vec<(CompanyUuid, String)>>;
    fn company_name_error(&self) -> impl HashimSignal<String>;
    fn selected_company_name(&self) -> impl HashimSignal<String>;
    fn selected_company_uuid(&self) -> impl HashimSignal<Option<CompanyUuid>>;
    fn branch_name(&self) -> impl HashimSignal<String>;
    fn currency(&self) -> impl HashimSignal<Currency>;
    fn location(&self) -> impl HashimSignal<Location>;
    fn branch_name_error(&self) -> impl HashimSignal<Option<BranchNameError>>;
    fn location_error(&self) -> impl HashimSignal<Option<LocationError>>;
}

#[derive(Debug, Clone)]
pub enum Change {
    ShowDialog(Dialog),
    IsLoading(bool),
    ListOfCompaniesToDisplay(Vec<(CompanyUuid, String)>),
    CompanyNameError(String),
    SelectedCompanyName(String),
    SelectedCompanyUuid(Option<CompanyUuid>),
    BranchName(String),
    Currency(Currency),
    Location(Location),
    BranchNameError(Option<BranchNameError>),
    LocationError(Option<LocationError>),
}

#[derive(Debug, Clone)]
pub enum Effect {
    Submit {
        process_id: ProcessId,
        user_uuid: UserUuid,
        company_belong: CompanyUuid,
        branch_name: String,
        currency: Currency,
        location: Location,
    },
    Consent {
        process_id: ProcessId,
        user_consent: UserConsent,
    },
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
}

#[derive(Debug, Clone)]
pub enum Observe {
    ShowDialog,
    HideDialog,
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
            Intent::Submit => {
                // Check the typed company name against the filtered list.
                let company_name = local_model.selected_company_name().read();
                let list = local_model.list_of_companies_to_display().read();

                let Some(first) = list.get(0) else {
                    // No company to submit against; nothing to do.
                    return Ok((vec![], vec![]));
                };

                if !company_name.is_empty() && company_name != first.1 {
                    let change = vec![Change::CompanyNameError(
                        "please select company".to_string(),
                    )];
                    return Ok((change, vec![]));
                }

                // Resolve the company: local pick first, else global default.
                let Some(company_belong) = local_model
                    .selected_company_uuid()
                    .read()
                    .or_else(|| global_model.selected_company().read())
                else {
                    return Ok((vec![], vec![]));
                };

                let Some(user_uuid) = global_model.user_uuid() else {
                    return Ok((vec![], vec![]));
                };

                let change = vec![
                    Change::IsLoading(true),
                    Change::CompanyNameError(String::new()),
                    Change::ShowDialog(Default::default()),
                    Change::BranchNameError(None),
                    Change::LocationError(None),
                ];
                let effect = vec![Effect::Submit {
                    process_id,
                    user_uuid,
                    company_belong,
                    branch_name: local_model.branch_name().read(),
                    currency: local_model.currency().read(),
                    location: local_model.location().read(),
                }];
                Ok((change, effect))
            }
            Intent::Consent(v) => {
                let change = match v {
                    UserConsent::CancelOperation => {
                        vec![Change::ShowDialog(Dialog::Hide), Change::IsLoading(false)]
                    }
                    _ => vec![Change::ShowDialog(Dialog::Hide)],
                };
                let effect = vec![Effect::Consent {
                    process_id,
                    user_consent: v,
                }];
                Ok((change, effect))
            }
            Intent::Clean => {
                let change = vec![
                    Change::ShowDialog(Default::default()),
                    Change::IsLoading(Default::default()),
                    Change::ListOfCompaniesToDisplay(Default::default()),
                    Change::CompanyNameError(Default::default()),
                    Change::SelectedCompanyName(Default::default()),
                    Change::SelectedCompanyUuid(Default::default()),
                    Change::BranchName(Default::default()),
                    Change::Currency(Default::default()),
                    Change::Location(Default::default()),
                    Change::BranchNameError(Default::default()),
                    Change::LocationError(Default::default()),
                ];
                Ok((change, vec![]))
            }
            Intent::CompanyName(v) => {
                let list_of_companies = global_model.list_of_companies();
                let filtered = select_strings(list_of_companies, v.clone(), |a| a.1.as_str());

                let uuid = match filtered.get(0) {
                    Some(a) if v == a.1 => Some(a.0.clone()),
                    _ => None,
                };

                let change = vec![
                    Change::SelectedCompanyName(v),
                    Change::ListOfCompaniesToDisplay(filtered),
                    Change::SelectedCompanyUuid(uuid),
                ];
                Ok((change, vec![]))
            }
            Intent::SelectedCompany(idx) => {
                let list = local_model.list_of_companies_to_display().read();
                match list.get(idx) {
                    Some(a) => {
                        let change = vec![
                            Change::SelectedCompanyName(a.1.clone()),
                            Change::SelectedCompanyUuid(Some(a.0.clone())),
                        ];
                        Ok((change, vec![]))
                    }
                    None => Ok((vec![], vec![])),
                }
            }
            Intent::BranchName(v) => Ok((vec![Change::BranchName(v)], vec![])),
            Intent::Currency(v) => Ok((vec![Change::Currency(v)], vec![])),
            Intent::Latitude(v) => {
                let mut loc = local_model.location().read();
                loc.latitude = v;
                Ok((vec![Change::Location(loc)], vec![]))
            }
            Intent::Longitude(v) => {
                let mut loc = local_model.location().read();
                loc.longitude = v;
                Ok((vec![Change::Location(loc)], vec![]))
            }
        },
        Message::Observe(observe) => match observe {
            Observe::ShowDialog => Ok((vec![Change::ShowDialog(Dialog::Show)], vec![])),
            Observe::HideDialog => Ok((vec![Change::ShowDialog(Dialog::Hide)], vec![])),
            Observe::Result(result) => {
                let change = match result {
                    Ok(_) => vec![
                        Change::ShowDialog(Default::default()),
                        Change::IsLoading(Default::default()),
                        Change::BranchName(Default::default()),
                        Change::Currency(Default::default()),
                        Change::Location(Default::default()),
                        Change::BranchNameError(Default::default()),
                        Change::LocationError(Default::default()),
                    ],
                    Err(a) => vec![
                        Change::ShowDialog(Default::default()),
                        Change::IsLoading(Default::default()),
                        Change::BranchNameError(a.branch_name),
                        Change::LocationError(a.location),
                    ],
                };
                Ok((change, vec![]))
            }
        },
    }
}

pub fn update(msg: Change, local_model: &impl LocalModel, global_model: &impl GlobalModel) {
    match msg {
        Change::ShowDialog(i) => local_model.show_dialog().set(i),
        Change::IsLoading(i) => local_model.is_loading().set(i),
        Change::ListOfCompaniesToDisplay(i) => local_model.list_of_companies_to_display().set(i),
        Change::CompanyNameError(i) => local_model.company_name_error().set(i),
        Change::SelectedCompanyName(i) => local_model.selected_company_name().set(i),
        Change::SelectedCompanyUuid(i) => local_model.selected_company_uuid().set(i),
        Change::BranchName(i) => local_model.branch_name().set(i),
        Change::Currency(i) => local_model.currency().set(i),
        Change::Location(i) => local_model.location().set(i),
        Change::BranchNameError(i) => local_model.branch_name_error().set(i),
        Change::LocationError(i) => local_model.location_error().set(i),
    }
}

pub async fn effect(msg: Effect, mut context: UiContext) -> Result<()> {
    match msg {
        Effect::Submit {
            process_id,
            user_uuid,
            company_belong,
            branch_name,
            currency,
            location,
        } => {
            let input = Input {
                user_uuid,
                new_uuid: BranchUuid::from(UuidType::from(Id::generate())),
                company_belong,
                branch_name,
                currency,
                location,
            };
            handle_submit(process_id, &input, context).await?;
        }
        Effect::Consent {
            process_id,
            user_consent,
        } => {
            context
                .sender_to_process_manager
                .send(MessageToProcessManager::FromUser {
                    process_id,
                    consent: user_consent,
                })
                .await?;
        }
    }
    Ok(())
}

struct DialogDispatchAdapter {
    process_id: ProcessId,
    sender: Commander,
}

impl ProcessDialog for DialogDispatchAdapter {
    fn show(&self) {
        self.sender
            .send(self.process_id, Message::Observe(Observe::ShowDialog));
    }

    fn hide(&self) {
        self.sender
            .send(self.process_id, Message::Observe(Observe::HideDialog));
    }
}

async fn handle_submit(process_id: ProcessId, input: &Input, context: UiContext) -> Result<()> {
    let context1 = context.clone();

    let dialog_signal_adapter = Arc::new(DialogDispatchAdapter {
        process_id,
        sender: context.sender_to_commander.clone(),
    });

    let data: TypeOperationClientInput = Arc::new(input.clone());

    handle_fall_back(
        context.sender_to_error,
        context.cache,
        context.sender_to_process_manager,
        dialog_signal_adapter,
        process_id,
        data,
        move |data| {
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

            let is_ok = result.is_ok();

            context1
                .sender_to_commander
                .send(process_id, Message::Observe(Observe::Result(result)));

            Ok(is_ok)
        },
    )
    .await?;

    Ok(())
}
