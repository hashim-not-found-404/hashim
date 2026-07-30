use crate::accounting_domain::utility::accounting_stuff;
use crate::accounting_domain::utility::types;
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
    pub(crate) double_entries:           Vec<DoubleEntryError>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct DoubleEntryError {
    pub(crate) accounting_error:    accounting_stuff::Error,
    pub(crate) single_entry_errors: Vec<SingleEntryError>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct SingleEntryError {
    pub(crate) new_uuid: Option<types::RowIdError>,
    pub(crate) account:  Option<types::RowIdError>,
}

impl types::MyErrorTrait for Error {
    fn is_there_error(&self) -> bool {
        if self.user_uuid.is_some()
            || self.new_uuid.is_some()
            || self.belong_to_company_branch.is_some()
            || self.shared_entry_id.is_some()
        {
            return true;
        }

        for double in self.double_entries.iter() {
            if double.accounting_error.is_there_error() {
                return true;
            }

            for line in double.single_entry_errors.iter() {
                if *line != Default::default() {
                    return true;
                }
            }
        }

        false
    }
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
// Accounting trait implementations
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

impl accounting_stuff::SingleEntry for MiddelSingleEntry {
    type AccountId = types::UuidType;

    fn account_id(&self) -> &Self::AccountId {
        &self.account
    }

    fn quantity(&self) -> f64 {
        self.quantity
    }

    fn amount(&self) -> f64 {
        self.amount
    }

    fn flow_type(&self) -> accounting_stuff::CostFlowType {
        if self.is_inflow {
            return accounting_stuff::CostFlowType::InFlow(self.inflow_type.clone());
        }
        return accounting_stuff::CostFlowType::OutFlow(self.outflow_type.clone());
    }
}

impl accounting_stuff::EntryContainer for Vec<MiddelSingleEntry> {
    type Iter<'a> = std::slice::Iter<'a, MiddelSingleEntry>;
    type Single = MiddelSingleEntry;

    fn iter(&self) -> Self::Iter<'_> {
        self.as_slice().iter()
    }

    fn is_empty(&self) -> bool {
        Vec::is_empty(self)
    }

    fn len(&self) -> usize {
        Vec::len(self)
    }
}

impl accounting_stuff::Inventory for Vec<accounting_stuff::InventoryRecord> {
    fn push(&mut self, record: accounting_stuff::InventoryRecord) {
        Vec::push(self, record);
    }

    fn clear(&mut self) {
        Vec::clear(self);
    }

    fn is_empty(&self) -> bool {
        Vec::is_empty(self)
    }

    fn iter(&self) -> impl Iterator<Item = &accounting_stuff::InventoryRecord> {
        self.as_slice().iter()
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut accounting_stuff::InventoryRecord> {
        self.as_mut_slice().iter_mut()
    }

    fn sort_by<F>(&mut self, compare: F)
    where
        F: FnMut(
            &accounting_stuff::InventoryRecord,
            &accounting_stuff::InventoryRecord,
        ) -> std::cmp::Ordering,
    {
        self.as_mut_slice().sort_by(compare);
    }

    fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&accounting_stuff::InventoryRecord) -> bool,
    {
        <Vec<accounting_stuff::InventoryRecord>>::retain(self, f);
    }

    fn pop(&mut self) -> Option<accounting_stuff::InventoryRecord> {
        Vec::pop(self)
    }
}

impl accounting_stuff::AccountInfoProvider for HashMap<types::UuidType, AccountInfo> {
    type AccountId = types::UuidType;
    type Inventory = Vec<accounting_stuff::InventoryRecord>;

    fn is_debit_nature(&self, id: &Self::AccountId) -> bool {
        self.get(id).map(|info| info.is_debit).unwrap()
    }

    fn get_or_create_inventory(&mut self, id: &Self::AccountId) -> &mut Self::Inventory {
        &mut self
            .entry(id.clone())
            .or_insert_with(|| {
                AccountInfo {
                    is_debit:      false,
                    in_flow_type:  accounting_stuff::InFlowType::Manual,
                    out_flow_type: accounting_stuff::OutFlowType::Manual,
                    inventory:     Vec::new(),
                }
            })
            .inventory
    }
}

// -----------------------------------------------------------------------------
// Input methods (business logic)
// -----------------------------------------------------------------------------

fn map_input_type_to_middel_type(
    entry: Vec<Vec<SingleEntry>>,
    accounts_info: HashMap<types::UuidType, AccountInfo>,
) -> (Vec<Vec<MiddelSingleEntry>>, Vec<String>) {
    todo!()
}
