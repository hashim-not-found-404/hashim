use crate::accounting_domain::utility::accounting_stuff;
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
    pub double_entries:           Vec<DoubleEntry>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DoubleEntry(Vec<SingleEntry>);

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SingleEntry {
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

    container_is_empty:        bool,
    pub(crate) double_entries: Vec<DoubleEntryError>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct DoubleEntryError {
    entry_is_empty:              bool,
    you_need_to_split_the_entry: bool,
    debit_not_equal_credit:      Option<DebitNotEqualCreditError>,

    pub(crate) single_entry_errors: Vec<SingleEntryError>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct SingleEntryError {
    pub(crate) new_uuid: Option<types::RowIdError>,
    pub(crate) account:  Option<types::RowIdError>,

    quantity_and_amount_are_zero:       bool,
    duplicate_account_in_entry:         bool,
    inventory_is_empty:                 bool,
    the_amount_should_be_positive:      bool,
    the_quantity_should_be_positive:    bool,
    quantity_not_equal_amount:          bool,
    quantity_not_equal_zero:            bool,
    insufficient_quantity_in_inventory: Option<InsufficientQuantityInInventory>,
    amount_mismatch:                    Option<AmountMismatch>,
    insufficient_amount_in_inventory:   Option<InsufficientAmountInInventory>,
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
// error impls
// -----------------------------------------------------------------------------

impl types::MyErrorTrait for SingleEntryError {}

impl types::MyErrorTrait for DoubleEntryError {
    fn is_there_error(&self) -> bool {
        if self.entry_is_empty
            || self.you_need_to_split_the_entry
            || self.debit_not_equal_credit.is_some()
        {
            return true;
        }

        for line in self.single_entry_errors.iter() {
            if line.is_there_error() {
                return true;
            }
        }

        false
    }
}

impl types::MyErrorTrait for Error {
    fn is_there_error(&self) -> bool {
        if self.container_is_empty
            || self.user_uuid.is_some()
            || self.new_uuid.is_some()
            || self.belong_to_company_branch.is_some()
            || self.shared_entry_id.is_some()
        {
            return true;
        }

        for double in self.double_entries.iter() {
            if double.is_there_error() {
                return true;
            }
        }

        false
    }
}

impl accounting_stuff::ErrorSink for Error {
    fn is_there_error_single_entry(&self, double_idx: usize, single_idx: usize) -> bool {
        self.double_entries[double_idx].single_entry_errors[single_idx].is_there_error()
    }

    fn quantity_and_amount_are_zero(&mut self, double_idx: usize, single_idx: usize) {
        self.double_entries[double_idx].single_entry_errors[single_idx]
            .quantity_and_amount_are_zero = true;
    }

    fn duplicate_account_in_entry(&mut self, double_idx: usize, single_idx: usize) {
        self.double_entries[double_idx].single_entry_errors[single_idx]
            .duplicate_account_in_entry = true;
    }

    fn inventory_is_empty(&mut self, double_idx: usize, single_idx: usize) {
        self.double_entries[double_idx].single_entry_errors[single_idx].inventory_is_empty = true;
    }

    fn the_amount_should_be_positive(&mut self, double_idx: usize, single_idx: usize) {
        self.double_entries[double_idx].single_entry_errors[single_idx]
            .the_amount_should_be_positive = true;
    }

    fn the_quantity_should_be_positive(&mut self, double_idx: usize, single_idx: usize) {
        self.double_entries[double_idx].single_entry_errors[single_idx]
            .the_quantity_should_be_positive = true;
    }

    fn quantity_not_equal_amount(&mut self, double_idx: usize, single_idx: usize) {
        self.double_entries[double_idx].single_entry_errors[single_idx].quantity_not_equal_amount =
            true;
    }

    fn quantity_not_equal_zero(&mut self, double_idx: usize, single_idx: usize) {
        self.double_entries[double_idx].single_entry_errors[single_idx].quantity_not_equal_zero =
            true;
    }

    fn insufficient_quantity_in_inventory(
        &mut self,
        double_idx: usize,
        single_idx: usize,
        total_quantity: f64,
    ) {
        self.double_entries[double_idx].single_entry_errors[single_idx]
            .insufficient_quantity_in_inventory = Some(InsufficientQuantityInInventory {
            total_quantity,
        });
    }

    fn amount_mismatch(&mut self, double_idx: usize, single_idx: usize, expected_amount: f64) {
        self.double_entries[double_idx].single_entry_errors[single_idx].amount_mismatch =
            Some(AmountMismatch {
                expected_amount,
            });
    }

    fn insufficient_amount_in_inventory(
        &mut self,
        double_idx: usize,
        single_idx: usize,
        total_amount: f64,
    ) {
        self.double_entries[double_idx].single_entry_errors[single_idx]
            .insufficient_amount_in_inventory = Some(InsufficientAmountInInventory {
            total_amount,
        });
    }

    fn entry_is_empty(&mut self, double_idx: usize) {
        self.double_entries[double_idx].entry_is_empty = true;
    }

    fn you_need_to_split_the_entry(&mut self, double_idx: usize) {
        self.double_entries[double_idx].you_need_to_split_the_entry = true;
    }

    fn debit_not_equal_credit(&mut self, double_idx: usize, total_debit: f64, total_credit: f64) {
        self.double_entries[double_idx].debit_not_equal_credit = Some(DebitNotEqualCreditError {
            total_debit,
            total_credit,
        });
    }

    fn container_is_empty(&mut self) {
        self.container_is_empty = true;
    }
}

// -----------------------------------------------------------------------------
// logic
// -----------------------------------------------------------------------------

struct MiddelSingleEntry {
    new_uuid:     types::UuidType,
    account:      types::UuidType,
    is_debit:     bool,
    is_inflow:    bool,
    inflow_type:  accounting_stuff::InFlowType,
    outflow_type: accounting_stuff::OutFlowType,
    amount:       f64,
    quantity:     f64,
}

fn map_input_type_to_middel_type(
    entry: Vec<Vec<SingleEntry>>,
    accounts_info: HashMap<types::UuidType, AccountInfo>,
) -> (Vec<Vec<MiddelSingleEntry>>, Vec<String>) {
    todo!()
}
