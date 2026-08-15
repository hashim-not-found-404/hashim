use crate::accounting_domain::utility::accounting_stuff;
use crate::accounting_domain::utility::types;
use crate::accounting_domain::utility::types::MyErrorTrait;
use crate::utility::iterators;
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
pub struct DoubleEntryInput(Vec<SingleEntryInput>);

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
// Error types
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct Error {
    pub(crate) user_uuid:                Option<types::UserUuidError>,
    pub(crate) new_uuid:                 Option<types::RowIdError>,
    pub(crate) belong_to_company_branch: Option<types::RowIdError>,
    pub(crate) shared_entry_id:          Option<types::RowIdError>,

    pub(crate) container_is_empty: bool,
    pub(crate) double_entries:     Vec<DoubleEntryError>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct DoubleEntryError {
    pub(crate) entry_is_empty:              bool,
    pub(crate) you_need_to_split_the_entry: bool,
    pub(crate) debit_not_equal_credit:      Option<DebitNotEqualCreditError>,
    pub(crate) single_entry_errors:         Vec<SingleEntryError>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct SingleEntryError {
    pub(crate) new_uuid: Option<types::RowIdError>,
    pub(crate) account:  Option<types::RowIdError>,

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
// impls
// -----------------------------------------------------------------------------

impl types::MyErrorTrait for SingleEntryError {
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

        for line in &self.single_entry_errors {
            if line.is_there_error() {
                return true;
            }
        }

        false
    }
}

impl MyErrorTrait for Error {
    fn is_there_error(&self) -> bool {
        if self.container_is_empty
            || self.user_uuid.is_some()
            || self.new_uuid.is_some()
            || self.belong_to_company_branch.is_some()
            || self.shared_entry_id.is_some()
        {
            return true;
        }

        for double in &self.double_entries {
            if double.is_there_error() {
                return true;
            }
        }

        false
    }
}

type SingleEntryInputAndError<'a> = iterators::PairIter<'a, SingleEntryInput, SingleEntryError>;
type DoubleEntryInputAndError<'a> = iterators::PairIter<'a, DoubleEntryInput, DoubleEntryError>;
type ContainerEntryInputAndError<'a> = iterators::PairIter<'a, Input, Error>;

pub(crate) fn init_error_sink(entry: &Input) -> Error {
    let mut err = Error::default();
    err.double_entries = vec![DoubleEntryError::default(); entry.double_entries.len()];
    for (i, double) in entry.double_entries.iter().enumerate() {
        err.double_entries[i].single_entry_errors =
            vec![SingleEntryError::default(); double.0.len()];
    }
    err
}
