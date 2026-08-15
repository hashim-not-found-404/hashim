use crate::accounting_domain::utility::accounting_stuff;
use crate::accounting_domain::utility::accounting_stuff::DoubleEntry;
use crate::accounting_domain::utility::accounting_stuff::EntryContainer;
use crate::accounting_domain::utility::correct_journal_input;
use crate::accounting_domain::utility::types;
use crate::accounting_domain::utility::types::MyErrorTrait;
use crate::utility::traits;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::collections::HashSet;

pub type MyResult = Result<Ok, Error>;

// -----------------------------------------------------------------------------
// Input DTOs
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Input {
    pub new_uuid:                 types::UuidType,
    pub belong_to_company_branch: types::UuidType,
    pub user_uuid:                types::UuidType,
    pub notes:                    Option<String>,
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
// Output DTOs
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Ok {
    pub new_uuid:        types::UuidType,
    pub user_uuid:       types::UuidType,
    pub time:            u64,
    pub notes:           Option<String>,
    pub shared_entry_id: Option<types::UuidType>,
    pub double_entry:    Vec<SingleEntryOk>,
    pub inventory:       HashMap<types::UuidType, Vec<accounting_stuff::InventoryRecord>>,
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
// Error DTOs (parallel structure)
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct Error {
    pub(crate) user_uuid:                Option<types::UserUuidError>,
    pub(crate) new_uuid:                 Option<types::RowIdError>,
    pub(crate) belong_to_company_branch: Option<types::RowIdError>,
    pub(crate) shared_entry_id:          Option<types::RowIdError>,
    pub(crate) container_is_empty:       bool,
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
    pub account_info:             HashMap<types::UuidType, AccountInfo>,
}

pub struct AccountInfo {
    pub is_debit:      bool,
    pub in_flow_type:  accounting_stuff::InFlowType,
    pub out_flow_type: accounting_stuff::OutFlowType,
    pub inventory:     Vec<accounting_stuff::InventoryRecord>,
}

pub trait DatabaseRead {
    type Db<'a>;
    fn read(
        db: &mut Self::Db<'_>,
        read_input: &ReadInput,
    ) -> impl Future<Output = Result<ReadOutput, traits::DynamicError>>;
}

// -----------------------------------------------------------------------------
// Views that combine input and error (used by the correction pipeline)
// -----------------------------------------------------------------------------

/// A view that combines a single input entry with its error slot.
pub struct SingleEntryView {
    // input
    input: SingleEntryInput,

    // error
    error: SingleEntryError,
}

/// A view that combines a double entry with its error slot.
pub struct DoubleEntryView {
    // input not exist

    // error
    error_entry_is_empty:              bool,
    error_you_need_to_split_the_entry: bool,
    error_debit_not_equal_credit:      Option<DebitNotEqualCreditError>,

    // view
    view_single_entries: Vec<SingleEntryView>,
}

/// A view that combines the container with its error slot.
pub struct ContainerView {
    // input
    input_new_uuid:                 types::UuidType,
    input_belong_to_company_branch: types::UuidType,
    input_user_uuid:                types::UuidType,
    input_notes:                    Option<String>,
    input_shared_entry_id:          Option<types::UuidType>,

    // error
    error_user_uuid:                Option<types::UserUuidError>,
    error_new_uuid:                 Option<types::RowIdError>,
    error_belong_to_company_branch: Option<types::RowIdError>,
    error_shared_entry_id:          Option<types::RowIdError>,
    error_container_is_empty:       bool,

    // view
    view_double_entries: Vec<DoubleEntryView>,
}

// -----------------------------------------------------------------------------
// MyErrorTrait implementations
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

// The views don't need MyErrorTrait - they delegate to their error fields.

// -----------------------------------------------------------------------------
// Implement accounting_stuff::SingleEntry for SingleEntryView
// -----------------------------------------------------------------------------

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

// -----------------------------------------------------------------------------
// Implement correct_journal_input::SingleEntry for SingleEntryView
// -----------------------------------------------------------------------------

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

// -----------------------------------------------------------------------------
// Implement accounting_stuff::SingleEntryError for SingleEntryView
// -----------------------------------------------------------------------------

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

// -----------------------------------------------------------------------------
// Implement accounting_stuff::DoubleEntry for DoubleEntryView
// -----------------------------------------------------------------------------

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

// -----------------------------------------------------------------------------
// Implement accounting_stuff::DoubleEntryError for DoubleEntryView
// -----------------------------------------------------------------------------

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

// -----------------------------------------------------------------------------
// Implement accounting_stuff::EntryContainer for ContainerView
// -----------------------------------------------------------------------------

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

// -----------------------------------------------------------------------------
// Implement accounting_stuff::EntryContainerError for ContainerView
// -----------------------------------------------------------------------------

impl accounting_stuff::EntryContainerError for ContainerView {
    fn container_is_empty(&mut self) {
        self.error_container_is_empty = true;
    }
}

// -----------------------------------------------------------------------------
// Initialization function - creates a ContainerView from Input + Error
// -----------------------------------------------------------------------------
