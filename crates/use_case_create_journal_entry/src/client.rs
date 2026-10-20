use crate::domain::DebitNotEqualCreditError;
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
use kernel::new_types::SharedEntryUuid;
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

const RESOURCES_NAME_TO_POKE: &[ResourceName] = &[new_resource_name("entries")];
const RESOURCES_NAME_TO_LISTEN: &[ResourceName] = &[new_resource_name("accounts_for_branch")];

// -----------------------------------------------------------------------------
// UI-side data types (mirrors old `ui_model::{Account, DoubleEntry, SingleEntry}`)
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct Account {
    pub row_uuid: AccountForBranchUuid,
    pub is_debit: bool,
    pub is_permanent_account: bool,
    pub account_name: String,
    pub unit_of_measurement_of_quantity: String,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct DoubleEntry {
    pub entry_is_empty: bool,
    pub you_need_to_split_the_entry: bool,
    pub debit_not_equal_credit: Option<DebitNotEqualCreditError>,
    pub singles: Vec<SingleEntry>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct SingleEntry {
    pub user_input_account_name: String,
    pub inferred_account_id: Option<AccountForBranchUuid>,

    pub user_input_is_debit: Option<bool>,
    pub user_input_is_inflow: Option<bool>,
    pub user_input_quantity: Option<f64>,
    pub user_input_amount: Option<f64>,
    pub user_input_inflow_type: Option<InFlowType>,
    pub user_input_outflow_type: Option<OutFlowType>,

    pub inferred_is_debit: Option<bool>,
    pub inferred_is_inflow: Option<bool>,
    pub inferred_quantity: Option<f64>,
    pub inferred_amount: Option<f64>,
    pub inferred_inflow_type: Option<InFlowType>,
    pub inferred_outflow_type: Option<OutFlowType>,

    pub quantity_and_amount_are_zero: bool,
    pub duplicate_account_in_entry: bool,
    pub inventory_is_empty: bool,
    pub the_amount_should_be_positive: bool,
    pub the_quantity_should_be_positive: bool,
    pub quantity_not_equal_amount: bool,
    pub quantity_not_equal_zero: bool,
    pub insufficient_quantity_in_inventory: Option<f64>,
    pub amount_mismatch: Option<f64>,
    pub insufficient_amount_in_inventory: Option<f64>,
}

#[derive(Debug, Clone)]
pub enum SingleEntryField {
    Account(String),
    IsDebit(bool),
    IsInflow(bool),
    InflowType(InFlowType),
    OutflowType(OutFlowType),
    Amount(f64),
    Quantity(f64),
}

// -----------------------------------------------------------------------------
// Client trait impls
// -----------------------------------------------------------------------------

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
    match errr {
        Ok(_) => {}
        Err(_) => return Ok(Err(Box::new(Error::default()))),
    }
    // Note: on the client, journal entries cannot be fully checked because the
    // inventory lives on the server. We return the state_less_operation only.
    let state_less_operation = Input {
        new_uuid: input.new_uuid.clone(),
        belong_to_company_branch: input.belong_to_company_branch.clone(),
        user_uuid: input.user_uuid.clone(),
        shared_entry_id: input.shared_entry_id.clone(),
        double_entries: input.double_entries.clone(),
    };

    // For client cache we don't have a separate Ok; build one with empty inventory.
    let ok = Ok {
        new_uuid: state_less_operation.new_uuid.clone(),
        user_uuid: state_less_operation.user_uuid.clone(),
        time: 0,
        shared_entry_id: state_less_operation.shared_entry_id.clone(),
        double_entry: Vec::new(),
        inventory: std::collections::HashMap::new(),
    };
    Ok(Ok(Arc::new(ok)))
}

pub(crate) type AsyncState = GenricAsyncState<AsyncInput, Ok, Error>;

pub trait GlobalModel {
    fn user_uuid(&self) -> Option<UserUuid>;
    fn selected_company_branch(&self) -> Option<kernel::new_types::BranchUuid>;
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct LocalModel {
    process_id: ProcessId,

    pub show_dialog: Dialog,
    pub is_loading: bool,
    pub shared_entry_id: String,
    pub some_account_are_not_inferred: bool,
    pub error_container_is_empty: bool,
    pub not_all_entry_inferred: bool,
    pub double_entries: Vec<DoubleEntry>,
    pub filtered_list: Vec<Account>,
    pub list_of_available_account: Vec<Account>,
}

impl LocalModel {
    pub fn clone_user_input(&self) -> Self {
        todo!();
    }

    pub fn is_submit_ok(&self) -> bool {
        todo!();
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct AsyncInput {
    pub new_uuid: UuidType,
    pub user_uuid: UserUuid,
    pub belong_to_company_branch: kernel::new_types::BranchUuid,
    pub shared_entry_id: Option<SharedEntryUuid>,
    pub double_entries: Vec<SingleEntryAsync>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SingleEntryAsync {
    pub new_uuid: UuidType,
    pub account: AccountForBranchUuid,
    pub is_debit: Option<bool>,
    pub is_inflow: Option<bool>,
    pub inflow_type: Option<InFlowType>,
    pub outflow_type: Option<OutFlowType>,
    pub amount: Option<f64>,
    pub quantity: Option<f64>,
}

// -----------------------------------------------------------------------------
// Reducer
// -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum LocalChange {
    ShowDialog(Dialog),
    IsLoading(bool),
    SharedEntryId(String),
    SomeAccountAreNotInferred(bool),
    ErrorContainerIsEmpty(bool),
    NotAllEntryInferred(bool),
    DoubleEntries(Vec<DoubleEntry>),
    FilteredList(Vec<Account>),
    AsyncState(AsyncState),
    AccountsList(Vec<Account>),
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
    FetchAccountsForBranch,
}

#[derive(Debug, Clone)]
pub enum Intent {
    Submit,
    Consent(UserConsent),
    Subscribe,
    UnSubscribe,
    AddSingleEntry {
        double_index: usize,
    },
    RemoveSingleEntry {
        double_index: usize,
        single_index: usize,
    },
    AddDoubleEntry,
    RemoveDoubleEntry {
        double_index: usize,
    },
    UpdateSingleEntry {
        double_index: usize,
        single_index: usize,
        value: SingleEntryField,
    },
    SetSharedEntryId(String),
}

#[derive(Debug, Clone)]
pub enum Observe {
    Timeout,
    Result(AsyncState),
    Refresh,
    AccountsFetched(Vec<Account>),
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
                if local_model.is_loading {
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

                let ui_entries = local_model.double_entries.clone();

                let mut double_entries_async = Vec::with_capacity(ui_entries.len());
                for double in &ui_entries {
                    let mut singles = Vec::with_capacity(double.singles.len());
                    for single in &double.singles {
                        let Some(account) = single.inferred_account_id.clone() else {
                            return (
                                vec![LocalChange::SomeAccountAreNotInferred(true)],
                                vec![],
                                vec![],
                            );
                        };
                        singles.push(SingleEntryAsync {
                            new_uuid: UuidType::from(Id::generate()),
                            account,
                            is_debit: single.user_input_is_debit,
                            is_inflow: single.user_input_is_inflow,
                            inflow_type: single.user_input_inflow_type,
                            outflow_type: single.user_input_outflow_type,
                            amount: single.user_input_amount,
                            quantity: single.user_input_quantity,
                        });
                    }
                    double_entries_async.push(singles);
                }

                // Flatten the double entries into a flat list of SingleEntryAsync
                // grouped by double entry index. For simplicity we keep the flat
                // list shape; the DB layer numbers them by position.
                let flat_single: Vec<SingleEntryAsync> =
                    double_entries_async.into_iter().flatten().collect();

                let shared_entry_id = Id::parse(local_model.shared_entry_id.clone())
                    .map(|a| SharedEntryUuid::from(UuidType::from(a)));

                let async_input = AsyncInput {
                    new_uuid: UuidType::from(Id::generate()),
                    user_uuid,
                    belong_to_company_branch,
                    shared_entry_id,
                    double_entries: flat_single,
                };

                let local_change = vec![
                    LocalChange::IsLoading(true),
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
                            LocalChange::IsLoading(false),
                        ],
                        vec![],
                    ),
                    UserConsent::WaitForServerResponse => (
                        vec![LocalChange::ShowDialog(Dialog::Hide)],
                        vec![Effect::SpawnTimer],
                    ),
                    UserConsent::DontWaitForServerResponse => {
                        // Client-only fallback path is not wired yet.
                        (vec![LocalChange::ShowDialog(Dialog::Hide)], vec![])
                    }
                };
                (local_change, vec![], effect)
            }
            Intent::Subscribe => {
                let effect = vec![
                    Effect::Subscribe(local_model.process_id),
                    Effect::FetchAccountsForBranch,
                ];
                (vec![], vec![], effect)
            }
            Intent::UnSubscribe => {
                let effect = vec![Effect::UnSubscribe(local_model.process_id)];
                (vec![], vec![], effect)
            }
            Intent::AddSingleEntry { double_index } => {
                let mut entries = local_model.double_entries.clone();
                if let Some(double) = entries.get_mut(double_index) {
                    double.singles.push(SingleEntry::default());
                    return (vec![LocalChange::DoubleEntries(entries)], vec![], vec![]);
                }
                (vec![], vec![], vec![])
            }
            Intent::RemoveSingleEntry {
                double_index,
                single_index,
            } => {
                let mut entries = local_model.double_entries.clone();
                if let Some(double) = entries.get_mut(double_index)
                    && single_index < double.singles.len()
                {
                    double.singles.remove(single_index);
                    return (vec![LocalChange::DoubleEntries(entries)], vec![], vec![]);
                }
                (vec![], vec![], vec![])
            }
            Intent::AddDoubleEntry => {
                let mut entries = local_model.double_entries.clone();
                entries.push(DoubleEntry::default());
                (vec![LocalChange::DoubleEntries(entries)], vec![], vec![])
            }
            Intent::RemoveDoubleEntry { double_index } => {
                let mut entries = local_model.double_entries.clone();
                if double_index < entries.len() {
                    entries.remove(double_index);
                    return (vec![LocalChange::DoubleEntries(entries)], vec![], vec![]);
                }
                (vec![], vec![], vec![])
            }
            Intent::UpdateSingleEntry {
                double_index,
                single_index,
                value,
            } => {
                let mut entries = local_model.double_entries.clone();
                if let Some(double) = entries.get_mut(double_index)
                    && let Some(single) = double.singles.get_mut(single_index)
                {
                    match value {
                        SingleEntryField::Account(name) => {
                            let master = local_model.list_of_available_account.clone();
                            let filtered =
                                select_strings(master, &name, |a| a.account_name.as_str());
                            single.user_input_account_name = name;
                            single.inferred_account_id = None;
                            return (
                                vec![
                                    LocalChange::FilteredList(filtered),
                                    LocalChange::DoubleEntries(entries),
                                ],
                                vec![],
                                vec![],
                            );
                        }
                        SingleEntryField::IsDebit(b) => single.user_input_is_debit = Some(b),
                        SingleEntryField::IsInflow(b) => single.user_input_is_inflow = Some(b),
                        SingleEntryField::InflowType(t) => single.user_input_inflow_type = Some(t),
                        SingleEntryField::OutflowType(t) => {
                            single.user_input_outflow_type = Some(t)
                        }
                        SingleEntryField::Amount(f) => single.user_input_amount = Some(f),
                        SingleEntryField::Quantity(f) => single.user_input_quantity = Some(f),
                    }
                    return (vec![LocalChange::DoubleEntries(entries)], vec![], vec![]);
                }
                (vec![], vec![], vec![])
            }
            Intent::SetSharedEntryId(uuid_type) => {
                (vec![LocalChange::SharedEntryId(uuid_type)], vec![], vec![])
            }
        },
        Message::Observe(observe) => match observe {
            Observe::Timeout => {
                let local_change = if local_model.is_loading {
                    vec![LocalChange::ShowDialog(Dialog::Show)]
                } else {
                    vec![]
                };
                (local_change, vec![], vec![])
            }
            Observe::Result(result) => match result {
                AsyncState::Success { .. } => {
                    let local_change = vec![
                        LocalChange::ShowDialog(Default::default()),
                        LocalChange::IsLoading(false),
                        LocalChange::DoubleEntries(Default::default()),
                        LocalChange::SomeAccountAreNotInferred(false),
                        LocalChange::FilteredList(Default::default()),
                        LocalChange::AsyncState(result),
                    ];
                    (local_change, vec![], vec![])
                }
                _ => {
                    let local_change = vec![
                        LocalChange::ShowDialog(Default::default()),
                        LocalChange::IsLoading(false),
                        LocalChange::AsyncState(result),
                    ];
                    (local_change, vec![], vec![])
                }
            },
            Observe::Refresh => (vec![], vec![], vec![Effect::FetchAccountsForBranch]),
            Observe::AccountsFetched(accounts) => (
                vec![
                    LocalChange::AccountsList(accounts.clone()),
                    LocalChange::FilteredList(accounts),
                ],
                vec![],
                vec![],
            ),
        },
    }
}

pub fn local_apply(msg: LocalChange, local_model: &mut LocalModel) {
    match msg {
        LocalChange::ShowDialog(i) => local_model.show_dialog = i,
        LocalChange::IsLoading(i) => local_model.is_loading = i,
        LocalChange::SharedEntryId(i) => local_model.shared_entry_id = i,
        LocalChange::SomeAccountAreNotInferred(i) => local_model.some_account_are_not_inferred = i,
        LocalChange::ErrorContainerIsEmpty(i) => local_model.error_container_is_empty = i,
        LocalChange::NotAllEntryInferred(i) => local_model.not_all_entry_inferred = i,
        LocalChange::DoubleEntries(i) => local_model.double_entries = i,
        LocalChange::FilteredList(i) => local_model.filtered_list = i,
        LocalChange::AsyncState(i) => {
            let _ = i;
        }
        LocalChange::AccountsList(i) => local_model.list_of_available_account = i,
    }
}

pub fn global_apply(msg: GlobalChange, _: &mut impl GlobalModel) {
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
        Effect::FetchAccountsForBranch => {
            // TODO: once `use_case_get_all_accounts_for_branch` is migrated, call
            // its client::fetch here and send back `Observe::AccountsFetched`.
            let _ = context;
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
        // Group the flat single entries back into one double entry. For now
        // keep them all in a single double entry; the DB layer assigns
        // double_entry_number sequentially.
        let double_entries = vec![crate::domain::DoubleEntryInput {
            single_entries: input
                .double_entries
                .iter()
                .map(|s| crate::domain::SingleEntryInput {
                    new_uuid: s.new_uuid.clone(),
                    account: s.account.clone(),
                    is_debit: s.is_debit,
                    is_inflow: s.is_inflow,
                    inflow_type: s.inflow_type,
                    outflow_type: s.outflow_type,
                    amount: s.amount,
                    quantity: s.quantity,
                })
                .collect(),
        }];

        Input {
            new_uuid: input.new_uuid.clone(),
            belong_to_company_branch: input.belong_to_company_branch.clone(),
            user_uuid: input.user_uuid.clone(),
            shared_entry_id: input.shared_entry_id.clone(),
            double_entries,
        }
    }
}
