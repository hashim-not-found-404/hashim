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
    entry_is_empty:                 bool,
    you_need_to_split_the_entry:    bool,
    debit_not_equal_credit:         Option<DebitNotEqualCreditError>,
    pub(crate) single_entry_errors: Vec<SingleEntryError>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct SingleEntryError {
    pub(crate) new_uuid: Option<types::RowIdError>,
    pub(crate) account:  Option<types::RowIdError>,

    is_debit_or_inflow_missing:         bool,
    amount_missing:                     bool,
    quantity_missing:                   bool,
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
// Intermediate types for validation
// -----------------------------------------------------------------------------

/// Fully resolved single entry after inference from account info.
#[derive(Clone, Debug)]
struct MiddelSingleEntry {
    new_uuid:     types::UuidType,
    account:      types::UuidType,
    is_debit:     bool,
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

    fn is_debit(&self) -> bool {
        self.is_debit
    }

    fn quantity(&self) -> f64 {
        self.quantity
    }

    fn amount(&self) -> f64 {
        self.amount
    }

    fn flow_type(&self) -> (accounting_stuff::InFlowType, accounting_stuff::OutFlowType) {
        (self.inflow_type.clone(), self.outflow_type.clone())
    }
}

struct InferredDoubleEntry(Vec<MiddelSingleEntry>);

impl accounting_stuff::DoubleEntry for InferredDoubleEntry {
    type Iter<'a> = std::slice::Iter<'a, MiddelSingleEntry>;
    type Single = MiddelSingleEntry;

    fn iter(&self) -> Self::Iter<'_> {
        self.0.iter()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}

struct InferredEntryContainer(Vec<InferredDoubleEntry>);

impl accounting_stuff::EntryContainer for InferredEntryContainer {
    type Double = InferredDoubleEntry;
    type Iter<'a> = std::slice::Iter<'a, InferredDoubleEntry>;

    fn iter(&self) -> Self::Iter<'_> {
        self.0.iter()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}

/// Mutable wrapper to provide `AccountInfoProvider` with mutable inventory.
struct AccountInfoProviderMut<'a> {
    info: &'a mut HashMap<types::UuidType, AccountInfo>,
}

impl<'a> accounting_stuff::AccountInfoProvider for AccountInfoProviderMut<'a> {
    type AccountId = types::UuidType;
    type Inventory = Vec<accounting_stuff::InventoryRecord>;

    fn is_debit_nature(&self, id: &Self::AccountId) -> bool {
        self.info.get(id).map(|a| a.is_debit).unwrap_or(true)
    }

    fn get_or_create_inventory(&mut self, id: &Self::AccountId) -> &mut Self::Inventory {
        &mut self.info.get_mut(id).expect("Account info not found").inventory
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
        <[accounting_stuff::InventoryRecord]>::iter(self)
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut accounting_stuff::InventoryRecord> {
        <[accounting_stuff::InventoryRecord]>::iter_mut(self)
    }

    fn sort_by<F>(&mut self, compare: F)
    where
        F: FnMut(
            &accounting_stuff::InventoryRecord,
            &accounting_stuff::InventoryRecord,
        ) -> std::cmp::Ordering,
    {
        <[accounting_stuff::InventoryRecord]>::sort_by(self, compare);
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

// -----------------------------------------------------------------------------
// Helpers for error initialization and resolution
// -----------------------------------------------------------------------------

/// Pre‑allocate an `Error` sink matching the structure of `entry`.
pub(crate) fn init_error_sink(entry: &Input) -> Error {
    let mut err = Error::default();
    err.double_entries = vec![DoubleEntryError::default(); entry.double_entries.len()];
    for (i, double) in entry.double_entries.iter().enumerate() {
        err.double_entries[i].single_entry_errors =
            vec![SingleEntryError::default(); double.0.len()];
    }
    err
}

/// Resolve user‑supplied entries into fully‑inferred `MiddelSingleEntry` values.
/// This function never panics; it sets errors in the sink for any missing or invalid data.
fn map_input_type_to_middel_type(
    err: &mut Error,
    entry: Vec<Vec<SingleEntry>>,
    accounts_info: &HashMap<types::UuidType, AccountInfo>,
) -> Vec<Vec<MiddelSingleEntry>> {
    let mut resolved = Vec::with_capacity(entry.len());

    for (double_idx, double) in entry.iter().enumerate() {
        let mut resolved_singles = Vec::with_capacity(double.len());
        for (single_idx, single) in double.iter().enumerate() {
            // 1. Check account exists
            let account_info = match accounts_info.get(&single.account) {
                Some(info) => info,
                None => {
                    err.double_entries[double_idx].single_entry_errors[single_idx].account =
                        Some(types::RowIdError::NotExist);
                    // Skip this single – without account info we cannot proceed.
                    continue;
                }
            };

            // 2. Infer is_debit and is_inflow
            let (is_debit, _is_inflow) = match (single.is_debit, single.is_inflow) {
                (Some(d), Some(i)) => (d, i),
                (Some(d), None) => {
                    let i = accounting_stuff::is_inflow(account_info.is_debit, d);
                    (d, i)
                }
                (None, Some(i)) => {
                    let d = accounting_stuff::is_debit(account_info.is_debit, i);
                    (d, i)
                }
                (None, None) => {
                    err.double_entries[double_idx].single_entry_errors[single_idx]
                        .is_debit_or_inflow_missing = true;
                    // Fallback: set is_debit = true (debit) and is_inflow = true (inflow)
                    // This allows validation to continue.
                    (true, true)
                }
            };

            // 3. Flow types – fallback to account defaults if not provided
            let inflow_type =
                single.inflow_type.clone().unwrap_or_else(|| account_info.in_flow_type.clone());
            let outflow_type =
                single.outflow_type.clone().unwrap_or_else(|| account_info.out_flow_type.clone());

            // 4. Amount and quantity – must be provided; if missing, set 0.0 and mark error
            let (amount, quantity) = match (single.amount, single.quantity) {
                (Some(a), Some(q)) => (a, q),
                (Some(a), None) => {
                    err.double_entries[double_idx].single_entry_errors[single_idx]
                        .quantity_missing = true;
                    (a, 0.0)
                }
                (None, Some(q)) => {
                    err.double_entries[double_idx].single_entry_errors[single_idx].amount_missing =
                        true;
                    (0.0, q)
                }
                (None, None) => {
                    err.double_entries[double_idx].single_entry_errors[single_idx].amount_missing =
                        true;
                    err.double_entries[double_idx].single_entry_errors[single_idx]
                        .quantity_missing = true;
                    (0.0, 0.0)
                }
            };

            resolved_singles.push(MiddelSingleEntry {
                new_uuid: single.new_uuid.clone(),
                account: single.account.clone(),
                is_debit,
                inflow_type,
                outflow_type,
                amount,
                quantity,
            });
        }
        resolved.push(resolved_singles);
    }

    resolved
}

// -----------------------------------------------------------------------------
// Validation methods on Input
// -----------------------------------------------------------------------------

impl Input {
    /// Shallow state‑less check: only validates UUIDs and basic structure.
    pub(crate) fn state_less_check<Id: types::RowId>(&self) -> Error {
        let mut err = init_error_sink(self);

        if !Id::validate(&self.new_uuid) {
            err.new_uuid = Some(types::RowIdError::Invalid);
        }
        if !Id::validate(&self.belong_to_company_branch) {
            err.belong_to_company_branch = Some(types::RowIdError::Invalid);
        }
        if !Id::validate(&self.user_uuid) {
            err.user_uuid = Some(types::UserUuidError::Invalid);
        }
        if let Some(ref shared) = self.shared_entry_id {
            if !Id::validate(shared) {
                err.shared_entry_id = Some(types::RowIdError::Invalid);
            }
        }

        // Validate each single entry's UUIDs
        for (i, double) in self.double_entries.iter().enumerate() {
            for (j, single) in double.0.iter().enumerate() {
                if !Id::validate(&single.new_uuid) {
                    err.double_entries[i].single_entry_errors[j].new_uuid =
                        Some(types::RowIdError::Invalid);
                }
                if !Id::validate(&single.account) {
                    err.double_entries[i].single_entry_errors[j].account =
                        Some(types::RowIdError::Invalid);
                }
            }
        }

        err
    }

    /// Full validation: fetch account info, resolve entries, and run accounting checks.
    pub(crate) async fn state_full_operation<
        Id: types::RowId,
        Db: DatabaseRead,
        Ti: traits::Time,
    >(
        &self,
        db: &mut Db::Db<'_>,
    ) -> Result<MyResult, traits::DynamicError> {
        // 1. State‑less check (shallow)
        let mut err = self.state_less_check::<Id>();
        if err.is_there_error() {
            return Ok(Err(err));
        }

        // 2. Prepare read input
        let accounts_uuid: HashSet<_> = self
            .double_entries
            .iter()
            .flat_map(|d| d.0.iter().map(|s| s.account.clone()))
            .collect();
        let new_entries_uuid: HashSet<_> = self
            .double_entries
            .iter()
            .flat_map(|d| d.0.iter().map(|s| s.new_uuid.clone()))
            .collect();

        let read_input = ReadInput {
            new_uuid: self.new_uuid.clone(),
            belong_to_company_branch: self.belong_to_company_branch.clone(),
            user_uuid: self.user_uuid.clone(),
            shared_entry_id: self.shared_entry_id.clone(),
            accounts_uuid,
            new_entries_uuid,
        };

        // 3. Read from database
        let read_output = Db::read(db, &read_input).await?;

        // 4. Check permissions
        if !types::Role::has_any(&read_output.user_roles, &[
            types::Role::Manager,
            types::Role::CoManager,
        ]) {
            err.user_uuid = Some(types::UserUuidError::YouDontHavePermissionToDoThat);
        }

        // 5. Check UUID conflicts
        if read_output.is_new_uuid_used {
            err.new_uuid = Some(types::RowIdError::Duplicated);
        }
        if self.shared_entry_id.is_some() {
            if !read_output.is_shared_entry_exist {
                err.shared_entry_id = Some(types::RowIdError::NotExist);
            }
        }

        // Check individual new_entries_uuid
        for (i, double) in self.double_entries.iter().enumerate() {
            for (j, single) in double.0.iter().enumerate() {
                if *read_output.is_new_entries_uuid_used.get(&single.new_uuid).unwrap_or(&false) {
                    err.double_entries[i].single_entry_errors[j].new_uuid =
                        Some(types::RowIdError::Duplicated);
                }
            }
        }

        // 6. Resolve entries using account_info
        let resolved = map_input_type_to_middel_type(
            &mut err,
            self.double_entries.iter().map(|d| d.0.clone()).collect(),
            &read_output.account_info,
        );

        // 7. Build inferred container and run accounting validation
        let mut account_info_mut = read_output.account_info;
        let mut inferred_double_entries = Vec::with_capacity(resolved.len());
        for double in resolved {
            inferred_double_entries.push(InferredDoubleEntry(double));
        }

        let inferred_container = InferredEntryContainer(inferred_double_entries);
        let mut provider = AccountInfoProviderMut {
            info: &mut account_info_mut,
        };
        let time = Ti::now_as_unix_milliseconds();

        // 8. Run accounting validation (delegated to accounting_stuff)
        accounting_stuff::state_less_check_for_entry(&mut err, &inferred_container);
        accounting_stuff::state_full_check_for_entry(
            time,
            &mut err,
            &inferred_container,
            &mut provider,
        );

        if err.is_there_error() {
            return Ok(Err(err));
        }

        // 9. Build the output
        let mut double_entry_ok = Vec::new();
        for (double_idx, double) in inferred_container.0.iter().enumerate() {
            for single in &double.0 {
                double_entry_ok.push(SingleEntryOk {
                    new_uuid:            single.new_uuid.clone(),
                    double_entry_number: double_idx as u32,
                    account:             single.account.clone(),
                    is_debit:            single.is_debit,
                    out_flow_type:       single.outflow_type.clone(),
                    quantity:            single.quantity,
                    amount:              single.amount,
                });
            }
        }

        let ok = Ok {
            new_uuid: self.new_uuid.clone(),
            user_uuid: self.user_uuid.clone(),
            time,
            notes: self.notes.clone(),
            shared_entry_id: self.shared_entry_id.clone(),
            double_entry: double_entry_ok,
            inventory: account_info_mut.into_iter().map(|(k, v)| (k, v.inventory)).collect(),
        };

        Ok(Ok(ok))
    }
}
