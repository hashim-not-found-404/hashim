use crate::accounting_domain::utility::accounting_stuff;
use crate::accounting_domain::utility::accounting_stuff::DoubleEntry;
use crate::accounting_domain::utility::accounting_stuff::EntryContainer;
use crate::accounting_domain::utility::correct_journal_input;
use crate::accounting_domain::utility::types;
use crate::accounting_domain::utility::types::MyErrorTrait;
use crate::accounting_domain::utility::types::RowId;
use crate::utility::traits;
use crate::utility::traits::Time;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Deref;
use std::ops::DerefMut;

pub type MyResult = Result<Ok, Error>;

// -----------------------------------------------------------------------------
// Input DTOs
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Input {
    pub new_uuid:                 types::UuidType,
    pub belong_to_company_branch: types::UuidType,
    pub user_uuid:                types::UuidType,
    pub shared_entry_id:          Option<types::UuidType>,
    pub double_entries:           Vec<DoubleEntryInput>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DoubleEntryInput {
    single_entries: Vec<SingleEntryInput>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SingleEntryInput {
    pub new_uuid:     types::UuidType,
    pub account:      types::UuidType,
    pub is_debit:     Option<bool>,
    pub is_inflow:    Option<bool>,
    pub inflow_type:  Option<accounting_stuff::InFlowType>,
    pub outflow_type: Option<accounting_stuff::OutFlowType>,
    pub amount:       Option<f64>,
    pub quantity:     Option<f64>,
}

// -----------------------------------------------------------------------------
// Ok DTOs
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Ok {
    pub new_uuid:        types::UuidType,
    pub user_uuid:       types::UuidType,
    pub time:            u64,
    pub shared_entry_id: Option<types::UuidType>,
    pub double_entry:    Vec<SingleEntryOk>,
    pub inventory:       HashMap<types::UuidType, InventoryWrapper>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SingleEntryOk {
    pub new_uuid:            types::UuidType,
    pub double_entry_number: u32,
    pub account:             types::UuidType,
    pub is_debit:            bool,
    pub out_flow_type:       accounting_stuff::OutFlowType,
    pub quantity:            f64,
    pub amount:              f64,
}

// -----------------------------------------------------------------------------
// Error DTOs
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct Error {
    pub(crate) user_uuid:                Option<types::UserUuidError>,
    pub(crate) new_uuid:                 Option<types::RowIdError>,
    pub(crate) belong_to_company_branch: Option<types::RowIdError>,
    pub(crate) shared_entry_id:          Option<types::RowIdError>,
    pub(crate) container_is_empty:       bool,
    pub(crate) not_all_entry_inferred:   bool,
    pub(crate) double_entries:           Vec<DoubleEntryError>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct DoubleEntryError {
    pub(crate) entry_is_empty:              bool,
    pub(crate) you_need_to_split_the_entry: bool,
    pub(crate) debit_not_equal_credit:      Option<DebitNotEqualCreditError>,
    pub(crate) single_entries:              Vec<SingleEntryError>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct SingleEntryError {
    pub(crate) new_uuid:                           Option<types::RowIdError>,
    pub(crate) account:                            Option<types::RowIdError>,
    pub(crate) quantity_and_amount_are_zero:       bool,
    pub(crate) duplicate_account_in_entry:         bool,
    pub(crate) inventory_is_empty:                 bool,
    pub(crate) the_amount_should_be_positive:      bool,
    pub(crate) the_quantity_should_be_positive:    bool,
    pub(crate) quantity_not_equal_amount:          bool,
    pub(crate) quantity_not_equal_zero:            bool,
    pub(crate) insufficient_quantity_in_inventory: Option<InsufficientQuantityInInventory>,
    pub(crate) amount_mismatch:                    Option<AmountMismatch>,
    pub(crate) insufficient_amount_in_inventory:   Option<InsufficientAmountInInventory>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub(crate) struct DebitNotEqualCreditError {
    total_debit:  f64,
    total_credit: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub(crate) struct InsufficientQuantityInInventory {
    total_quantity: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub(crate) struct AmountMismatch {
    expected_amount: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub(crate) struct InsufficientAmountInInventory {
    total_amount: f64,
}

// -----------------------------------------------------------------------------
// Database read types
// -----------------------------------------------------------------------------

pub struct ReadInput {
    pub new_uuid:                 types::UuidType,
    pub belong_to_company_branch: types::UuidType,
    pub user_uuid:                types::UuidType,
    pub shared_entry_id:          Option<types::UuidType>,
    pub accounts_uuid:            HashSet<types::UuidType>,
    pub new_entries_uuid:         HashSet<types::UuidType>,
}

pub struct ReadOutput {
    pub is_new_uuid_used:         bool,
    pub user_roles:               Vec<types::Role>,
    pub is_shared_entry_exist:    bool,
    pub is_new_entries_uuid_used: HashMap<types::UuidType, bool>,
    pub account_info:             AccountInfoProviderImpl,
}

#[derive(Debug, Clone)]
pub struct AccountInfo {
    pub is_debit:      bool,
    pub in_flow_type:  accounting_stuff::InFlowType,
    pub out_flow_type: accounting_stuff::OutFlowType,
    pub inventory:     InventoryWrapper,
}

pub trait DatabaseRead {
    type Db<'a>;
    fn read(
        db: &mut Self::Db<'_>,
        read_input: &ReadInput,
    ) -> impl Future<Output = Result<ReadOutput, traits::DynamicError>>;
}

// -----------------------------------------------------------------------------
// Views that combine input and error
// -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct SingleEntryView {
    // input
    input: SingleEntryInput,

    // error
    error: SingleEntryError,
}

#[derive(Debug, Clone)]
pub struct DoubleEntryView {
    // input not exist

    // error
    error_entry_is_empty:              bool,
    error_you_need_to_split_the_entry: bool,
    error_debit_not_equal_credit:      Option<DebitNotEqualCreditError>,

    // view
    view_single_entries: Vec<SingleEntryView>,
}

#[derive(Debug, Clone)]
pub struct ContainerView {
    // input
    input_new_uuid:                 types::UuidType,
    input_belong_to_company_branch: types::UuidType,
    input_user_uuid:                types::UuidType,
    input_shared_entry_id:          Option<types::UuidType>,

    // error
    error_user_uuid:                Option<types::UserUuidError>,
    error_new_uuid:                 Option<types::RowIdError>,
    error_belong_to_company_branch: Option<types::RowIdError>,
    error_shared_entry_id:          Option<types::RowIdError>,
    error_container_is_empty:       bool,
    error_not_all_entry_inferred:   bool,

    // view
    view_double_entries: Vec<DoubleEntryView>,
}

// -----------------------------------------------------------------------------
// AccountInfoProvider
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InventoryWrapper(Vec<accounting_stuff::InventoryRecord>);

impl DerefMut for InventoryWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for InventoryWrapper {
    type Target = Vec<accounting_stuff::InventoryRecord>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone)]
pub struct AccountInfoProviderImpl(HashMap<types::UuidType, AccountInfo>);

impl DerefMut for AccountInfoProviderImpl {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for AccountInfoProviderImpl {
    type Target = HashMap<types::UuidType, AccountInfo>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// -----------------------------------------------------------------------------
// implementations
// -----------------------------------------------------------------------------

impl MyErrorTrait for SingleEntryView {
    fn is_there_error(&self) -> bool {
        self.error != SingleEntryError::default()
    }
}

impl MyErrorTrait for DoubleEntryView {
    fn is_there_error(&self) -> bool {
        if self.error_entry_is_empty
            || self.error_you_need_to_split_the_entry
            || self.error_debit_not_equal_credit.is_some()
        {
            return true;
        }
        self.iter_ref().any(|e| e.is_there_error())
    }
}

impl MyErrorTrait for ContainerView {
    fn is_there_error(&self) -> bool {
        if self.error_container_is_empty
            || self.error_user_uuid.is_some()
            || self.error_new_uuid.is_some()
            || self.error_belong_to_company_branch.is_some()
            || self.error_shared_entry_id.is_some()
        {
            return true;
        }
        self.iter_ref().any(|e| e.is_there_error())
    }
}

impl MyErrorTrait for SingleEntryError {
    fn is_there_error(&self) -> bool {
        *self != Self::default()
    }
}

impl MyErrorTrait for DoubleEntryError {
    fn is_there_error(&self) -> bool {
        if self.entry_is_empty
            || self.you_need_to_split_the_entry
            || self.debit_not_equal_credit.is_some()
        {
            return true;
        }
        self.single_entries.iter().any(|e| e.is_there_error())
    }
}

impl MyErrorTrait for Error {
    fn is_there_error(&self) -> bool {
        if self.user_uuid.is_some()
            || self.new_uuid.is_some()
            || self.belong_to_company_branch.is_some()
            || self.shared_entry_id.is_some()
            || self.container_is_empty
        {
            return true;
        }
        self.double_entries.iter().any(|e| e.is_there_error())
    }
}

impl accounting_stuff::SingleEntry for SingleEntryView {
    type AccountId = types::UuidType;

    fn account_id(&self) -> Self::AccountId {
        self.input.account.clone()
    }

    fn is_debit(&self) -> bool {
        self.input.is_debit.unwrap_or(false)
    }

    fn quantity(&self) -> f64 {
        self.input.quantity.unwrap_or(0.0)
    }

    fn amount(&self) -> f64 {
        self.input.amount.unwrap_or(0.0)
    }

    fn inflow_type(&self) -> accounting_stuff::InFlowType {
        self.input.inflow_type.clone().unwrap_or(accounting_stuff::InFlowType::Manual)
    }

    fn outflow_type(&self) -> accounting_stuff::OutFlowType {
        self.input.outflow_type.clone().unwrap_or(accounting_stuff::OutFlowType::Manual)
    }
}

impl accounting_stuff::SingleEntryError for SingleEntryView {
    fn quantity_and_amount_are_zero(&mut self) {
        self.error.quantity_and_amount_are_zero = true;
    }

    fn duplicate_account_in_entry(&mut self) {
        self.error.duplicate_account_in_entry = true;
    }

    fn inventory_is_empty(&mut self) {
        self.error.inventory_is_empty = true;
    }

    fn the_amount_should_be_positive(&mut self) {
        self.error.the_amount_should_be_positive = true;
    }

    fn the_quantity_should_be_positive(&mut self) {
        self.error.the_quantity_should_be_positive = true;
    }

    fn quantity_not_equal_amount(&mut self) {
        self.error.quantity_not_equal_amount = true;
    }

    fn quantity_not_equal_zero(&mut self) {
        self.error.quantity_not_equal_zero = true;
    }

    fn insufficient_quantity_in_inventory(&mut self, total_quantity: f64) {
        self.error.insufficient_quantity_in_inventory = Some(InsufficientQuantityInInventory {
            total_quantity,
        });
    }

    fn amount_mismatch(&mut self, expected_amount: f64) {
        self.error.amount_mismatch = Some(AmountMismatch {
            expected_amount,
        });
    }

    fn insufficient_amount_in_inventory(&mut self, total_amount: f64) {
        self.error.insufficient_amount_in_inventory = Some(InsufficientAmountInInventory {
            total_amount,
        });
    }
}

impl accounting_stuff::DoubleEntry for DoubleEntryView {
    type Iter<'b>
        = std::vec::IntoIter<Self::Single>
    where
        Self: 'b;
    type IterMut<'b>
        = std::slice::IterMut<'b, Self::Single>
    where
        Self: 'b;
    type IterRef<'b>
        = std::slice::Iter<'b, Self::Single>
    where
        Self: 'b;
    type Single = SingleEntryView;

    fn into_iter<'b>(self) -> Self::Iter<'b> {
        self.view_single_entries.into_iter()
    }

    fn iter_ref(&self) -> Self::IterRef<'_> {
        self.view_single_entries.iter()
    }

    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        self.view_single_entries.iter_mut()
    }

    fn set_singles(&mut self, singles: Vec<Self::Single>) {
        self.view_single_entries = singles;
    }

    fn is_empty(&self) -> bool {
        self.view_single_entries.is_empty()
    }

    fn len(&self) -> usize {
        self.view_single_entries.len()
    }

    fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&Self::Single) -> bool,
    {
        self.view_single_entries.retain(f);
    }
}

impl accounting_stuff::DoubleEntryError for DoubleEntryView {
    fn entry_is_empty(&mut self) {
        self.error_entry_is_empty = true;
    }

    fn you_need_to_split_the_entry(&mut self) {
        self.error_you_need_to_split_the_entry = true;
    }

    fn debit_not_equal_credit(&mut self, total_debit: f64, total_credit: f64) {
        self.error_debit_not_equal_credit = Some(DebitNotEqualCreditError {
            total_debit,
            total_credit,
        });
    }
}

impl accounting_stuff::EntryContainer for ContainerView {
    type Double<'b>
        = DoubleEntryView
    where
        Self: 'b;
    type Iter<'b>
        = std::vec::IntoIter<Self::Double<'b>>
    where
        Self: 'b;
    type IterMut<'b>
        = std::slice::IterMut<'b, Self::Double<'b>>
    where
        Self: 'b;
    type IterRef<'b>
        = std::slice::Iter<'b, Self::Double<'b>>
    where
        Self: 'b;

    fn iter<'b>(self) -> Self::Iter<'b> {
        self.view_double_entries.into_iter()
    }

    fn iter_ref(&self) -> Self::IterRef<'_> {
        self.view_double_entries.iter()
    }

    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        self.view_double_entries.iter_mut()
    }

    fn set_doubles(&mut self, doubles: Vec<Self::Double<'_>>) {
        self.view_double_entries = doubles;
    }

    fn is_empty(&self) -> bool {
        self.view_double_entries.is_empty()
    }

    fn len(&self) -> usize {
        self.view_double_entries.len()
    }

    fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&Self::Double<'_>) -> bool,
    {
        self.view_double_entries.retain(f);
    }
}

impl accounting_stuff::EntryContainerError for ContainerView {
    fn container_is_empty(&mut self) {
        self.error_container_is_empty = true;
    }
}

impl accounting_stuff::AccountInfoProvider for AccountInfoProviderImpl {
    type AccountId = types::UuidType;
    type Inventory = InventoryWrapper;

    fn is_debit_nature(&self, id: &Self::AccountId) -> bool {
        self.get(id).unwrap().is_debit
    }

    fn get_or_create_inventory(&mut self, id: &Self::AccountId) -> &mut Self::Inventory {
        &mut self.get_mut(id).unwrap().inventory
    }
}

impl accounting_stuff::Inventory for InventoryWrapper {
    fn push(&mut self, record: accounting_stuff::InventoryRecord) {
        Vec::push(self, record);
    }

    fn clear(&mut self) {
        Vec::clear(self);
    }

    fn is_empty(&self) -> bool {
        Vec::is_empty(self)
    }

    fn iter1(&self) -> impl Iterator<Item = &accounting_stuff::InventoryRecord> {
        self.iter()
    }

    fn iter_mut1(&mut self) -> impl Iterator<Item = &mut accounting_stuff::InventoryRecord> {
        self.iter_mut()
    }

    fn sort_by1<F>(&mut self, compare: F)
    where
        F: FnMut(
            &accounting_stuff::InventoryRecord,
            &accounting_stuff::InventoryRecord,
        ) -> std::cmp::Ordering,
    {
        self.sort_by(compare);
    }

    fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&accounting_stuff::InventoryRecord) -> bool,
    {
        Vec::retain(self, f);
    }

    fn pop(&mut self) -> Option<accounting_stuff::InventoryRecord> {
        Vec::pop(self)
    }
}

impl correct_journal_input::SingleEntry for SingleEntryView {
    type AccountId = types::UuidType;

    fn get_account_id(&self) -> Self::AccountId {
        self.input.account.clone()
    }

    fn get_from_user_input_is_debit(&self) -> Option<bool> {
        self.input.is_debit
    }

    fn get_from_user_input_is_inflow(&self) -> Option<bool> {
        self.input.is_inflow
    }

    fn get_from_user_input_quantity(&self) -> Option<f64> {
        self.input.quantity
    }

    fn get_from_user_input_amount(&self) -> Option<f64> {
        self.input.amount
    }

    fn get_from_user_input_inflow_type(&self) -> Option<accounting_stuff::InFlowType> {
        self.input.inflow_type.clone()
    }

    fn get_from_user_input_outflow_type(&self) -> Option<accounting_stuff::OutFlowType> {
        self.input.outflow_type.clone()
    }

    fn set_user_input_is_debit(&mut self, i: Option<bool>) {
        self.input.is_debit = i;
    }

    fn set_user_input_is_inflow(&mut self, i: Option<bool>) {
        self.input.is_inflow = i;
    }

    fn set_user_input_quantity(&mut self, i: Option<f64>) {
        self.input.quantity = i;
    }

    fn set_user_input_amount(&mut self, i: Option<f64>) {
        self.input.amount = i;
    }

    fn set_user_input_inflow_type(&mut self, i: Option<accounting_stuff::InFlowType>) {
        self.input.inflow_type = i;
    }

    fn set_user_input_outflow_type(&mut self, i: Option<accounting_stuff::OutFlowType>) {
        self.input.outflow_type = i;
    }

    fn set_inferred_is_debit(&mut self, i: Option<bool>) {
        self.input.is_debit = i;
    }

    fn set_inferred_is_inflow(&mut self, i: Option<bool>) {
        self.input.is_inflow = i;
    }

    fn set_inferred_quantity(&mut self, i: Option<f64>) {
        self.input.quantity = i;
    }

    fn set_inferred_amount(&mut self, i: Option<f64>) {
        self.input.amount = i;
    }

    fn set_inferred_inflow_type(&mut self, i: Option<accounting_stuff::InFlowType>) {
        self.input.inflow_type = i;
    }

    fn set_inferred_outflow_type(&mut self, i: Option<accounting_stuff::OutFlowType>) {
        self.input.outflow_type = i;
    }

    fn get_inferred_is_debit(&self) -> Option<bool> {
        self.input.is_debit
    }

    fn get_inferred_is_inflow(&self) -> Option<bool> {
        self.input.is_inflow
    }

    fn get_inferred_quantity(&self) -> Option<f64> {
        self.input.quantity
    }

    fn get_inferred_amount(&self) -> Option<f64> {
        self.input.amount
    }

    fn get_inferred_inflow_type(&self) -> Option<accounting_stuff::InFlowType> {
        self.input.inflow_type.clone()
    }

    fn get_inferred_outflow_type(&self) -> Option<accounting_stuff::OutFlowType> {
        self.input.outflow_type.clone()
    }
}

impl correct_journal_input::AccountInfoProvider for AccountInfoProviderImpl {
    type AccountId = types::UuidType;
    type Inventory = InventoryWrapper;

    fn get_info<'a>(
        &'a self,
        id: &Self::AccountId,
    ) -> Option<correct_journal_input::AccountInfo<&'a Self::Inventory>> {
        let info = self.get(id)?;
        Some(correct_journal_input::AccountInfo {
            is_debit:      info.is_debit,
            in_flow_type:  info.in_flow_type,
            out_flow_type: info.out_flow_type,
            inventory:     &info.inventory,
        })
    }

    fn get_info_mut<'a>(
        &'a mut self,
        id: &Self::AccountId,
    ) -> Option<correct_journal_input::AccountInfo<&'a mut Self::Inventory>> {
        let info = self.get_mut(id)?;
        Some(correct_journal_input::AccountInfo {
            is_debit:      info.is_debit,
            in_flow_type:  info.in_flow_type,
            out_flow_type: info.out_flow_type,
            inventory:     &mut info.inventory,
        })
    }
}
// -----------------------------------------------------------------------------
// Helper functions
// -----------------------------------------------------------------------------

pub fn create_container_view(input: Input) -> ContainerView {
    let view_double_entries: Vec<DoubleEntryView> = input
        .double_entries
        .into_iter()
        .map(|double_input| {
            let view_single_entries: Vec<SingleEntryView> = double_input
                .single_entries
                .into_iter()
                .map(|single_input| {
                    SingleEntryView {
                        input: single_input,
                        error: SingleEntryError::default(),
                    }
                })
                .collect();

            DoubleEntryView {
                error_entry_is_empty: Default::default(),
                error_you_need_to_split_the_entry: Default::default(),
                error_debit_not_equal_credit: Default::default(),
                view_single_entries,
            }
        })
        .collect();

    ContainerView {
        input_new_uuid: input.new_uuid,
        input_belong_to_company_branch: input.belong_to_company_branch,
        input_user_uuid: input.user_uuid,
        input_shared_entry_id: input.shared_entry_id,
        error_user_uuid: Default::default(),
        error_new_uuid: Default::default(),
        error_belong_to_company_branch: Default::default(),
        error_shared_entry_id: Default::default(),
        error_container_is_empty: Default::default(),
        error_not_all_entry_inferred: Default::default(),
        view_double_entries,
    }
}

pub fn split_container_view(container: ContainerView) -> (Input, Error) {
    let ContainerView {
        input_new_uuid,
        input_belong_to_company_branch,
        input_user_uuid,
        input_shared_entry_id,
        error_user_uuid,
        error_new_uuid,
        error_belong_to_company_branch,
        error_shared_entry_id,
        error_container_is_empty,
        error_not_all_entry_inferred,
        view_double_entries,
    } = container;

    let mut double_entries_input = Vec::with_capacity(view_double_entries.len());
    let mut double_entries_error = Vec::with_capacity(view_double_entries.len());

    for double_view in view_double_entries {
        let DoubleEntryView {
            error_entry_is_empty,
            error_you_need_to_split_the_entry,
            error_debit_not_equal_credit,
            view_single_entries,
        } = double_view;

        let mut singles_input = Vec::with_capacity(view_single_entries.len());
        let mut singles_error = Vec::with_capacity(view_single_entries.len());

        for single_view in view_single_entries {
            let SingleEntryView {
                input,
                error,
            } = single_view;
            singles_input.push(input);
            singles_error.push(error);
        }

        let double_input = DoubleEntryInput {
            single_entries: singles_input,
        };
        let double_error = DoubleEntryError {
            entry_is_empty:              error_entry_is_empty,
            you_need_to_split_the_entry: error_you_need_to_split_the_entry,
            debit_not_equal_credit:      error_debit_not_equal_credit,
            single_entries:              singles_error,
        };

        double_entries_input.push(double_input);
        double_entries_error.push(double_error);
    }

    let input = Input {
        new_uuid:                 input_new_uuid,
        belong_to_company_branch: input_belong_to_company_branch,
        user_uuid:                input_user_uuid,
        shared_entry_id:          input_shared_entry_id,
        double_entries:           double_entries_input,
    };

    let error = Error {
        user_uuid:                error_user_uuid,
        new_uuid:                 error_new_uuid,
        belong_to_company_branch: error_belong_to_company_branch,
        shared_entry_id:          error_shared_entry_id,
        container_is_empty:       error_container_is_empty,
        not_all_entry_inferred:   error_not_all_entry_inferred,
        double_entries:           double_entries_error,
    };

    (input, error)
}

pub fn is_fully_inferred(container: &ContainerView) -> bool {
    container.view_double_entries.iter().all(|double| {
        double.view_single_entries.iter().all(|single| {
            single.input.is_debit.is_some()
                && single.input.quantity.is_some()
                && single.input.amount.is_some()
                && single.input.inflow_type.is_some()
                && single.input.outflow_type.is_some()
        })
    })
}

fn create_double_entry_from_container_view(container: ContainerView) -> Vec<SingleEntryOk> {
    let mut double_entry_ok = Vec::new();
    for (double_idx, double_view) in container.view_double_entries.iter().enumerate() {
        for single_view in &double_view.view_single_entries {
            let single_ok = SingleEntryOk {
                new_uuid:            single_view.input.new_uuid.clone(),
                double_entry_number: double_idx as u32,
                account:             single_view.input.account.clone(),
                is_debit:            single_view.input.is_debit.unwrap(),
                out_flow_type:       single_view.input.outflow_type.clone().unwrap(),
                quantity:            single_view.input.quantity.unwrap(),
                amount:              single_view.input.amount.unwrap(),
            };
            double_entry_ok.push(single_ok);
        }
    }
    double_entry_ok
}

fn extract_inventory(
    account_info: AccountInfoProviderImpl,
) -> HashMap<types::UuidType, InventoryWrapper> {
    let mut inventory_map = HashMap::new();
    for (account_uuid, info) in account_info.0 {
        inventory_map.insert(account_uuid, info.inventory);
    }
    inventory_map
}

// -----------------------------------------------------------------------------
// Input validation methods
// -----------------------------------------------------------------------------

impl Input {
    pub(crate) fn state_less_check<Id: RowId>(&self) -> Error {
        let mut container = create_container_view(self.clone());

        if !Id::validate(&self.new_uuid) {
            container.error_new_uuid = Some(types::RowIdError::Invalid);
        }
        if !Id::validate(&self.user_uuid) {
            container.error_user_uuid = Some(types::UserUuidError::Invalid);
        }
        if !Id::validate(&self.belong_to_company_branch) {
            container.error_belong_to_company_branch = Some(types::RowIdError::Invalid);
        }

        if let Some(shared_id) = &self.shared_entry_id {
            if !Id::validate(shared_id) {
                container.error_shared_entry_id = Some(types::RowIdError::Invalid);
            }
        }

        for double_view in container.view_double_entries.iter_mut() {
            for single_view in double_view.view_single_entries.iter_mut() {
                if !Id::validate(&single_view.input.new_uuid) {
                    single_view.error.new_uuid = Some(types::RowIdError::Invalid);
                }
                if !Id::validate(&single_view.input.account) {
                    single_view.error.account = Some(types::RowIdError::Invalid);
                }
            }
        }

        // Detect duplicate new_uuid within the input itself
        let mut seen_uuids = HashSet::new();
        for double_view in container.view_double_entries.iter_mut() {
            for single_view in double_view.view_single_entries.iter_mut() {
                if !seen_uuids.insert(single_view.input.new_uuid.clone()) {
                    single_view.error.new_uuid = Some(types::RowIdError::Duplicated);
                }
            }
        }

        let (_, error) = split_container_view(container);
        error
    }

    pub(crate) async fn state_full_check<Db: DatabaseRead, Ti: Time>(
        &self,
        db: &mut Db::Db<'_>,
    ) -> Result<MyResult, traits::DynamicError> {
        let mut accounts_uuid = HashSet::new();
        let mut new_entries_uuid = HashSet::new();
        for double in &self.double_entries {
            for single in &double.single_entries {
                accounts_uuid.insert(single.account.clone());
                new_entries_uuid.insert(single.new_uuid.clone());
            }
        }

        let read_input = ReadInput {
            new_uuid: self.new_uuid.clone(),
            belong_to_company_branch: self.belong_to_company_branch.clone(),
            user_uuid: self.user_uuid.clone(),
            shared_entry_id: self.shared_entry_id.clone(),
            accounts_uuid,
            new_entries_uuid,
        };

        let read_output = Db::read(db, &read_input).await?;

        let mut container = create_container_view(self.clone());

        if read_output.is_new_uuid_used {
            container.error_new_uuid = Some(types::RowIdError::Duplicated);
        }

        if !types::Role::has_any(&read_output.user_roles, &[
            types::Role::Manager,
            types::Role::CoManager,
        ]) {
            container.error_user_uuid = Some(types::UserUuidError::YouDontHavePermissionToDoThat);
        }

        if let Some(_) = &self.shared_entry_id {
            if !read_output.is_shared_entry_exist {
                container.error_shared_entry_id = Some(types::RowIdError::NotExist);
            }
        }

        // Check for duplicate new UUIDs already existing in the database
        for (uuid, used) in &read_output.is_new_entries_uuid_used {
            if *used {
                for double_view in container.view_double_entries.iter_mut() {
                    for single_view in double_view.view_single_entries.iter_mut() {
                        if single_view.input.new_uuid == *uuid {
                            single_view.error.new_uuid = Some(types::RowIdError::Duplicated);
                        }
                    }
                }
            }
        }

        let mut account_info = read_output.account_info;

        correct_journal_input::correct_the_input(
            Ti::now_as_unix_milliseconds(),
            &mut container,
            account_info.clone(),
        );

        if !is_fully_inferred(&container) {
            container.error_not_all_entry_inferred = true;

            let (_, error) = split_container_view(container);
            return Ok(Err(error));
        }
        accounting_stuff::state_less_check_for_entry(&mut container);

        accounting_stuff::state_full_check_for_entry(
            Ti::now_as_unix_milliseconds(),
            &mut container,
            &mut account_info,
        );

        if container.is_there_error() {
            let (_, error) = split_container_view(container);
            return Ok(Err(error));
        }

        let ok = Ok {
            new_uuid:        self.new_uuid.clone(),
            user_uuid:       self.user_uuid.clone(),
            time:            Ti::now_as_unix_milliseconds(),
            shared_entry_id: self.shared_entry_id.clone(),
            double_entry:    create_double_entry_from_container_view(container),
            inventory:       extract_inventory(account_info),
        };

        Ok(Ok(ok))
    }
}
