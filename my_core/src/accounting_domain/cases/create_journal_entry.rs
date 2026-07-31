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
/// Resolve user‑supplied entries into fully‑inferred `MiddelSingleEntry` values.
/// This function performs a two‑pass resolution:
/// 1. First pass: resolve each single entry with available data, marking missing fields.
/// 2. Second pass: try to infer missing amounts/quantities from other singles in the same double entry.
/// If inference fails, set appropriate errors.
fn map_input_type_to_middel_type(
    err: &mut Error,
    entry: Vec<Vec<SingleEntry>>,
    accounts_info: &HashMap<types::UuidType, AccountInfo>,
) -> Vec<Vec<MiddelSingleEntry>> {
    let mut resolved = Vec::with_capacity(entry.len());
    let mut missing_flags = Vec::with_capacity(entry.len()); // track which fields are missing per single

    // ───── First pass: resolve with placeholders ─────
    for (double_idx, double) in entry.iter().enumerate() {
        let mut resolved_singles = Vec::with_capacity(double.len());
        let mut flags = Vec::with_capacity(double.len());

        for (single_idx, single) in double.iter().enumerate() {
            // 1. Account existence
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
            let (is_debit, is_inflow) = match (single.is_debit, single.is_inflow) {
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

            // 4. Amount and quantity – store values and mark missing
            let (amount, qty, amount_missing, qty_missing) = match (single.amount, single.quantity)
            {
                (Some(a), Some(q)) => (a, q, false, false),
                (Some(a), None) => (a, 0.0, false, true),
                (None, Some(q)) => (0.0, q, true, false),
                (None, None) => (0.0, 0.0, true, true),
            };

            resolved_singles.push(MiddelSingleEntry {
                new_uuid: single.new_uuid.clone(),
                account: single.account.clone(),
                is_debit,
                inflow_type,
                outflow_type,
                amount,
                quantity: qty,
            });
            flags.push((amount_missing, qty_missing));
        }
        resolved.push(resolved_singles);
        missing_flags.push(flags);
    }

    // ───── Second pass: cross‑inference within each double entry ─────
    for (double_idx, double) in resolved.iter_mut().enumerate() {
        let flags = &missing_flags[double_idx];
        let len = double.len();
        if len == 0 {
            continue;
        }

        // Collect all singles that have both amount and quantity (so we can compute price)
        let mut price_samples = Vec::new();
        for (idx, single) in double.iter().enumerate() {
            let (amt_missing, qty_missing) = flags[idx];
            if !amt_missing && !qty_missing && single.quantity != 0.0 {
                let price = single.amount / single.quantity;
                price_samples.push((idx, price));
            }
        }

        // If we have at least one price sample, try to infer missing values.
        if !price_samples.is_empty() {
            // We'll use the first price sample (or we could average them).
            let (_, avg_price) = price_samples[0];

            for (idx, single) in double.iter_mut().enumerate() {
                let (amt_missing, qty_missing) = flags[idx];
                if amt_missing && !qty_missing {
                    // Missing amount, but quantity present → infer amount = quantity * price
                    single.amount = single.quantity * avg_price;
                    // Clear the missing flag for amount (so we don't set error later)
                    // But we still need to record that it was inferred; we can keep the error if we want,
                    // or we can clear it. We'll clear it since we successfully inferred.
                    err.double_entries[double_idx].single_entry_errors[idx].amount_missing = false;
                } else if !amt_missing && qty_missing {
                    // Missing quantity, but amount present → infer quantity = amount / price
                    if avg_price != 0.0 {
                        single.quantity = single.amount / avg_price;
                        err.double_entries[double_idx].single_entry_errors[idx].quantity_missing =
                            false;
                    }
                }
                // If both missing, we cannot infer from price.
            }
        }

        // After price‑based inference, if there are still missing amounts/quantities,
        // we can also infer from debit/credit balance (if one side is missing).
        // For now, we'll just let the accounting validation catch them.
    }

    // ───── Final step: set errors for any remaining missing fields ─────
    for (double_idx, double) in resolved.iter().enumerate() {
        let flags = &missing_flags[double_idx];
        for (single_idx, single) in double.iter().enumerate() {
            let (amt_missing, qty_missing) = flags[single_idx];
            if amt_missing
                && err.double_entries[double_idx].single_entry_errors[single_idx].amount_missing
            {
                // Already set, but we might have cleared it above if inferred.
                // If still true, it means we couldn't infer.
                err.double_entries[double_idx].single_entry_errors[single_idx].amount_missing =
                    true;
            }
            if qty_missing
                && err.double_entries[double_idx].single_entry_errors[single_idx].quantity_missing
            {
                err.double_entries[double_idx].single_entry_errors[single_idx].quantity_missing =
                    true;
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounting_domain::utility::accounting_stuff;
    use std::collections::HashMap;

    // -------------------------------------------------------------------------
    // Helper to create a dummy Input for init_error_sink (only for allocation)
    // This is not a test helper; it's used to create the initial error sink.
    // -------------------------------------------------------------------------
    fn dummy_input_with_double_entries(singles_per_double: &Vec<Vec<SingleEntry>>) -> Error {
        let mut double_entries = Vec::with_capacity(singles_per_double.len());
        for singles in singles_per_double {
            double_entries.push(DoubleEntry(singles.clone()));
        }
        let entry = Input {
            new_uuid: types::UuidType([0; 16]),
            belong_to_company_branch: types::UuidType([0; 16]),
            user_uuid: types::UuidType([0; 16]),
            notes: None,
            shared_entry_id: None,
            double_entries,
        };
        init_error_sink(&entry)
    }

    // -------------------------------------------------------------------------
    // Helper: create 10 distinct accounts with different natures, flow types,
    // and inventory records.
    // -------------------------------------------------------------------------
    fn create_ten_accounts_info() -> HashMap<types::UuidType, AccountInfo> {
        let mut map = HashMap::new();

        // Account 0: Debit nature, Manual flows, inventory: one record
        let uuid0 = types::UuidType([0; 16]);
        map.insert(uuid0.clone(), AccountInfo {
            is_debit:      true,
            in_flow_type:  accounting_stuff::InFlowType::Manual,
            out_flow_type: accounting_stuff::OutFlowType::Manual,
            inventory:     vec![accounting_stuff::InventoryRecord {
                time_unix: 1,
                quantity:  10.0,
                amount:    100.0,
            }],
        });

        // Account 1: Debit nature, QuantityEqualAmount inflow, Wac outflow
        let uuid1 = types::UuidType([1; 16]);
        map.insert(uuid1.clone(), AccountInfo {
            is_debit:      true,
            in_flow_type:  accounting_stuff::InFlowType::QuantityEqualAmount,
            out_flow_type: accounting_stuff::OutFlowType::Wac,
            inventory:     vec![
                accounting_stuff::InventoryRecord {
                    time_unix: 2,
                    quantity:  5.0,
                    amount:    50.0,
                },
                accounting_stuff::InventoryRecord {
                    time_unix: 3,
                    quantity:  3.0,
                    amount:    45.0,
                },
            ],
        });

        // Account 2: Debit nature, QuantityEqualZero inflow, Fifo outflow
        let uuid2 = types::UuidType([2; 16]);
        map.insert(uuid2.clone(), AccountInfo {
            is_debit:      true,
            in_flow_type:  accounting_stuff::InFlowType::QuantityEqualZero,
            out_flow_type: accounting_stuff::OutFlowType::Fifo,
            inventory:     vec![accounting_stuff::InventoryRecord {
                time_unix: 4,
                quantity:  8.0,
                amount:    120.0,
            }],
        });

        // Account 3: Debit nature, Manual inflow, Lifo outflow
        let uuid3 = types::UuidType([3; 16]);
        map.insert(uuid3.clone(), AccountInfo {
            is_debit:      true,
            in_flow_type:  accounting_stuff::InFlowType::Manual,
            out_flow_type: accounting_stuff::OutFlowType::Lifo,
            inventory:     vec![
                accounting_stuff::InventoryRecord {
                    time_unix: 5,
                    quantity:  2.0,
                    amount:    30.0,
                },
                accounting_stuff::InventoryRecord {
                    time_unix: 6,
                    quantity:  4.0,
                    amount:    80.0,
                },
            ],
        });

        // Account 4: Debit nature, QuantityEqualAmount inflow, Hifo outflow
        let uuid4 = types::UuidType([4; 16]);
        map.insert(uuid4.clone(), AccountInfo {
            is_debit:      true,
            in_flow_type:  accounting_stuff::InFlowType::QuantityEqualAmount,
            out_flow_type: accounting_stuff::OutFlowType::Hifo,
            inventory:     vec![
                accounting_stuff::InventoryRecord {
                    time_unix: 7,
                    quantity:  6.0,
                    amount:    90.0,
                },
                accounting_stuff::InventoryRecord {
                    time_unix: 8,
                    quantity:  2.0,
                    amount:    40.0,
                },
            ],
        });

        // Account 5: Credit nature, Manual flows
        let uuid5 = types::UuidType([5; 16]);
        map.insert(uuid5.clone(), AccountInfo {
            is_debit:      false,
            in_flow_type:  accounting_stuff::InFlowType::Manual,
            out_flow_type: accounting_stuff::OutFlowType::Manual,
            inventory:     vec![accounting_stuff::InventoryRecord {
                time_unix: 9,
                quantity:  7.0,
                amount:    70.0,
            }],
        });

        // Account 6: Credit nature, QuantityEqualAmount inflow, Wac outflow
        let uuid6 = types::UuidType([6; 16]);
        map.insert(uuid6.clone(), AccountInfo {
            is_debit:      false,
            in_flow_type:  accounting_stuff::InFlowType::QuantityEqualAmount,
            out_flow_type: accounting_stuff::OutFlowType::Wac,
            inventory:     vec![accounting_stuff::InventoryRecord {
                time_unix: 10,
                quantity:  9.0,
                amount:    180.0,
            }],
        });

        // Account 7: Credit nature, QuantityEqualZero inflow, Fifo outflow
        let uuid7 = types::UuidType([7; 16]);
        map.insert(uuid7.clone(), AccountInfo {
            is_debit:      false,
            in_flow_type:  accounting_stuff::InFlowType::QuantityEqualZero,
            out_flow_type: accounting_stuff::OutFlowType::Fifo,
            inventory:     vec![
                accounting_stuff::InventoryRecord {
                    time_unix: 11,
                    quantity:  4.0,
                    amount:    60.0,
                },
                accounting_stuff::InventoryRecord {
                    time_unix: 12,
                    quantity:  3.0,
                    amount:    75.0,
                },
            ],
        });

        // Account 8: Credit nature, Manual inflow, Lifo outflow
        let uuid8 = types::UuidType([8; 16]);
        map.insert(uuid8.clone(), AccountInfo {
            is_debit:      false,
            in_flow_type:  accounting_stuff::InFlowType::Manual,
            out_flow_type: accounting_stuff::OutFlowType::Lifo,
            inventory:     vec![accounting_stuff::InventoryRecord {
                time_unix: 13,
                quantity:  5.0,
                amount:    55.0,
            }],
        });

        // Account 9: Credit nature, QuantityEqualAmount inflow, Hifo outflow
        let uuid9 = types::UuidType([9; 16]);
        map.insert(uuid9.clone(), AccountInfo {
            is_debit:      false,
            in_flow_type:  accounting_stuff::InFlowType::QuantityEqualAmount,
            out_flow_type: accounting_stuff::OutFlowType::Hifo,
            inventory:     vec![
                accounting_stuff::InventoryRecord {
                    time_unix: 14,
                    quantity:  6.0,
                    amount:    120.0,
                },
                accounting_stuff::InventoryRecord {
                    time_unix: 15,
                    quantity:  2.0,
                    amount:    50.0,
                },
            ],
        });

        map
    }

    // -------------------------------------------------------------------------
    // 1. Basic success – all fields present
    // -------------------------------------------------------------------------
    #[test]
    fn test_map_basic_success() {
        let account_uuid = types::UuidType([1; 16]);
        let mut accounts_info = HashMap::new();
        accounts_info.insert(account_uuid.clone(), AccountInfo {
            is_debit:      true,
            in_flow_type:  accounting_stuff::InFlowType::Manual,
            out_flow_type: accounting_stuff::OutFlowType::Manual,
            inventory:     Vec::new(),
        });

        let single = SingleEntry {
            new_uuid:     types::UuidType([0; 16]),
            account:      account_uuid.clone(),
            is_debit:     Some(true),
            is_inflow:    Some(false),
            inflow_type:  Some(accounting_stuff::InFlowType::Manual),
            outflow_type: Some(accounting_stuff::OutFlowType::Wac),
            amount:       Some(100.0),
            quantity:     Some(10.0),
        };
        let entry = vec![vec![single]];
        let mut err = dummy_input_with_double_entries(&entry);

        let resolved = map_input_type_to_middel_type(&mut err, entry, &accounts_info);

        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].len(), 1);
        let m = &resolved[0][0];
        assert_eq!(m.account, account_uuid);
        assert_eq!(m.is_debit, true);
        assert_eq!(m.inflow_type, accounting_stuff::InFlowType::Manual);
        assert_eq!(m.outflow_type, accounting_stuff::OutFlowType::Wac);
        assert_eq!(m.amount, 100.0);
        assert_eq!(m.quantity, 10.0);
        assert!(!err.is_there_error(), "Expected no errors");
    }

    // -------------------------------------------------------------------------
    // 2. Infer is_debit from is_inflow (nature = debit)
    // -------------------------------------------------------------------------
    #[test]
    fn test_infer_is_debit_from_is_inflow() {
        let account_uuid = types::UuidType([1; 16]);
        let mut accounts_info = HashMap::new();
        accounts_info.insert(account_uuid.clone(), AccountInfo {
            is_debit:      true, // debit nature
            in_flow_type:  accounting_stuff::InFlowType::Manual,
            out_flow_type: accounting_stuff::OutFlowType::Manual,
            inventory:     Vec::new(),
        });

        let single = SingleEntry {
            new_uuid:     types::UuidType([0; 16]),
            account:      account_uuid.clone(),
            is_debit:     None,        // missing, will infer
            is_inflow:    Some(false), // outflow → for debit nature, is_debit should be false
            inflow_type:  None,
            outflow_type: None,
            amount:       Some(100.0),
            quantity:     Some(10.0),
        };
        let entry = vec![vec![single]];
        let mut err = dummy_input_with_double_entries(&entry);

        let resolved = map_input_type_to_middel_type(&mut err, entry, &accounts_info);
        let m = &resolved[0][0];
        // is_inflow = false, nature = debit → is_debit = false
        assert_eq!(m.is_debit, false);
        assert!(!err.is_there_error());
    }

    // -------------------------------------------------------------------------
    // 3. Infer is_inflow from is_debit (nature = debit)
    // -------------------------------------------------------------------------
    #[test]
    fn test_infer_is_inflow_from_is_debit() {
        let account_uuid = types::UuidType([1; 16]);
        let mut accounts_info = HashMap::new();
        accounts_info.insert(account_uuid.clone(), AccountInfo {
            is_debit:      true, // debit nature
            in_flow_type:  accounting_stuff::InFlowType::Manual,
            out_flow_type: accounting_stuff::OutFlowType::Manual,
            inventory:     Vec::new(),
        });

        let single = SingleEntry {
            new_uuid:     types::UuidType([0; 16]),
            account:      account_uuid.clone(),
            is_debit:     Some(true), // debit state
            is_inflow:    None,       // will infer
            inflow_type:  None,
            outflow_type: None,
            amount:       Some(100.0),
            quantity:     Some(10.0),
        };
        let entry = vec![vec![single]];
        let mut err = dummy_input_with_double_entries(&entry);

        let resolved = map_input_type_to_middel_type(&mut err, entry, &accounts_info);
        let m = &resolved[0][0];
        // is_debit = true, nature = debit → is_inflow = true
        assert_eq!(m.is_debit, true);
        assert!(!err.is_there_error());
    }

    // -------------------------------------------------------------------------
    // 4. Both is_debit and is_inflow missing – set error and fallback
    // -------------------------------------------------------------------------
    #[test]
    fn test_both_is_debit_and_is_inflow_missing() {
        let account_uuid = types::UuidType([1; 16]);
        let mut accounts_info = HashMap::new();
        accounts_info.insert(account_uuid.clone(), AccountInfo {
            is_debit:      true,
            in_flow_type:  accounting_stuff::InFlowType::Manual,
            out_flow_type: accounting_stuff::OutFlowType::Manual,
            inventory:     Vec::new(),
        });

        let single = SingleEntry {
            new_uuid:     types::UuidType([0; 16]),
            account:      account_uuid.clone(),
            is_debit:     None,
            is_inflow:    None,
            inflow_type:  None,
            outflow_type: None,
            amount:       Some(100.0),
            quantity:     Some(10.0),
        };
        let entry = vec![vec![single]];
        let mut err = dummy_input_with_double_entries(&entry);

        let resolved = map_input_type_to_middel_type(&mut err, entry, &accounts_info);
        let m = &resolved[0][0];
        // Fallback is_debit = true
        assert_eq!(m.is_debit, true);
        assert!(err.double_entries[0].single_entry_errors[0].is_debit_or_inflow_missing);
    }

    // -------------------------------------------------------------------------
    // 5. Account not found – set error and skip
    // -------------------------------------------------------------------------
    #[test]
    fn test_account_not_found() {
        let account_uuid = types::UuidType([1; 16]);
        let accounts_info = HashMap::new(); // empty

        let single = SingleEntry {
            new_uuid:     types::UuidType([0; 16]),
            account:      account_uuid.clone(),
            is_debit:     Some(true),
            is_inflow:    Some(false),
            inflow_type:  None,
            outflow_type: None,
            amount:       Some(100.0),
            quantity:     Some(10.0),
        };
        let entry = vec![vec![single]];
        let mut err = dummy_input_with_double_entries(&entry);

        let resolved = map_input_type_to_middel_type(&mut err, entry, &accounts_info);
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].len(), 0); // skipped
        assert!(err.double_entries[0].single_entry_errors[0].account.is_some());
    }

    // -------------------------------------------------------------------------
    // 6. Amount missing – set error and amount = 0.0
    // -------------------------------------------------------------------------
    #[test]
    fn test_amount_missing() {
        let account_uuid = types::UuidType([1; 16]);
        let mut accounts_info = HashMap::new();
        accounts_info.insert(account_uuid.clone(), AccountInfo {
            is_debit:      true,
            in_flow_type:  accounting_stuff::InFlowType::Manual,
            out_flow_type: accounting_stuff::OutFlowType::Manual,
            inventory:     Vec::new(),
        });

        let single = SingleEntry {
            new_uuid:     types::UuidType([0; 16]),
            account:      account_uuid.clone(),
            is_debit:     Some(true),
            is_inflow:    Some(false),
            inflow_type:  None,
            outflow_type: None,
            amount:       None,
            quantity:     Some(10.0),
        };
        let entry = vec![vec![single]];
        let mut err = dummy_input_with_double_entries(&entry);

        let resolved = map_input_type_to_middel_type(&mut err, entry, &accounts_info);
        let m = &resolved[0][0];
        assert_eq!(m.amount, 0.0);
        assert_eq!(m.quantity, 10.0);
        assert!(err.double_entries[0].single_entry_errors[0].amount_missing);
    }

    // -------------------------------------------------------------------------
    // 7. Quantity missing – set error and quantity = 0.0
    // -------------------------------------------------------------------------
    #[test]
    fn test_quantity_missing() {
        let account_uuid = types::UuidType([1; 16]);
        let mut accounts_info = HashMap::new();
        accounts_info.insert(account_uuid.clone(), AccountInfo {
            is_debit:      true,
            in_flow_type:  accounting_stuff::InFlowType::Manual,
            out_flow_type: accounting_stuff::OutFlowType::Manual,
            inventory:     Vec::new(),
        });

        let single = SingleEntry {
            new_uuid:     types::UuidType([0; 16]),
            account:      account_uuid.clone(),
            is_debit:     Some(true),
            is_inflow:    Some(false),
            inflow_type:  None,
            outflow_type: None,
            amount:       Some(100.0),
            quantity:     None,
        };
        let entry = vec![vec![single]];
        let mut err = dummy_input_with_double_entries(&entry);

        let resolved = map_input_type_to_middel_type(&mut err, entry, &accounts_info);
        let m = &resolved[0][0];
        assert_eq!(m.amount, 100.0);
        assert_eq!(m.quantity, 0.0);
        assert!(err.double_entries[0].single_entry_errors[0].quantity_missing);
    }

    // -------------------------------------------------------------------------
    // 8. Both amount and quantity missing – both errors
    // -------------------------------------------------------------------------
    #[test]
    fn test_both_amount_and_quantity_missing() {
        let account_uuid = types::UuidType([1; 16]);
        let mut accounts_info = HashMap::new();
        accounts_info.insert(account_uuid.clone(), AccountInfo {
            is_debit:      true,
            in_flow_type:  accounting_stuff::InFlowType::Manual,
            out_flow_type: accounting_stuff::OutFlowType::Manual,
            inventory:     Vec::new(),
        });

        let single = SingleEntry {
            new_uuid:     types::UuidType([0; 16]),
            account:      account_uuid.clone(),
            is_debit:     Some(true),
            is_inflow:    Some(false),
            inflow_type:  None,
            outflow_type: None,
            amount:       None,
            quantity:     None,
        };
        let entry = vec![vec![single]];
        let mut err = dummy_input_with_double_entries(&entry);

        let resolved = map_input_type_to_middel_type(&mut err, entry, &accounts_info);
        let m = &resolved[0][0];
        assert_eq!(m.amount, 0.0);
        assert_eq!(m.quantity, 0.0);
        assert!(err.double_entries[0].single_entry_errors[0].amount_missing);
        assert!(err.double_entries[0].single_entry_errors[0].quantity_missing);
    }

    // -------------------------------------------------------------------------
    // 9. Cross‑inference: amount missing, quantity present, price available
    // -------------------------------------------------------------------------
    #[test]
    fn test_cross_infer_amount_from_quantity_and_price() {
        let account_uuid = types::UuidType([1; 16]);
        let mut accounts_info = HashMap::new();
        accounts_info.insert(account_uuid.clone(), AccountInfo {
            is_debit:      true,
            in_flow_type:  accounting_stuff::InFlowType::Manual,
            out_flow_type: accounting_stuff::OutFlowType::Manual,
            inventory:     Vec::new(),
        });

        let s0 = SingleEntry {
            new_uuid:     types::UuidType([0; 16]),
            account:      account_uuid.clone(),
            is_debit:     Some(true),
            is_inflow:    Some(false),
            inflow_type:  None,
            outflow_type: None,
            amount:       Some(100.0),
            quantity:     Some(10.0), // price = 10
        };
        let s1 = SingleEntry {
            new_uuid:     types::UuidType([1; 16]),
            account:      account_uuid.clone(),
            is_debit:     Some(true),
            is_inflow:    Some(false),
            inflow_type:  None,
            outflow_type: None,
            amount:       None,
            quantity:     Some(5.0),
        };
        let entry = vec![vec![s0, s1]];
        let mut err = dummy_input_with_double_entries(&entry);

        let resolved = map_input_type_to_middel_type(&mut err, entry, &accounts_info);
        assert_eq!(resolved[0].len(), 2);
        let m1 = &resolved[0][1];
        // Should infer amount = 5.0 * 10 = 50.0
        assert_eq!(m1.amount, 50.0);
        assert_eq!(m1.quantity, 5.0);
        assert!(!err.double_entries[0].single_entry_errors[1].amount_missing);
    }

    // -------------------------------------------------------------------------
    // 10. Cross‑inference: quantity missing, amount present, price available
    // -------------------------------------------------------------------------
    #[test]
    fn test_cross_infer_quantity_from_amount_and_price() {
        let account_uuid = types::UuidType([1; 16]);
        let mut accounts_info = HashMap::new();
        accounts_info.insert(account_uuid.clone(), AccountInfo {
            is_debit:      true,
            in_flow_type:  accounting_stuff::InFlowType::Manual,
            out_flow_type: accounting_stuff::OutFlowType::Manual,
            inventory:     Vec::new(),
        });

        let s0 = SingleEntry {
            new_uuid:     types::UuidType([0; 16]),
            account:      account_uuid.clone(),
            is_debit:     Some(true),
            is_inflow:    Some(false),
            inflow_type:  None,
            outflow_type: None,
            amount:       Some(100.0),
            quantity:     Some(10.0), // price = 10
        };
        let s1 = SingleEntry {
            new_uuid:     types::UuidType([1; 16]),
            account:      account_uuid.clone(),
            is_debit:     Some(true),
            is_inflow:    Some(false),
            inflow_type:  None,
            outflow_type: None,
            amount:       Some(50.0),
            quantity:     None,
        };
        let entry = vec![vec![s0, s1]];
        let mut err = dummy_input_with_double_entries(&entry);

        let resolved = map_input_type_to_middel_type(&mut err, entry, &accounts_info);
        let m1 = &resolved[0][1];
        assert_eq!(m1.amount, 50.0);
        assert_eq!(m1.quantity, 5.0);
        assert!(!err.double_entries[0].single_entry_errors[1].quantity_missing);
    }

    // -------------------------------------------------------------------------
    // 11. Cross‑inference with multiple price samples (uses first)
    // -------------------------------------------------------------------------
    #[test]
    fn test_cross_infer_with_multiple_price_samples() {
        let account_uuid = types::UuidType([1; 16]);
        let mut accounts_info = HashMap::new();
        accounts_info.insert(account_uuid.clone(), AccountInfo {
            is_debit:      true,
            in_flow_type:  accounting_stuff::InFlowType::Manual,
            out_flow_type: accounting_stuff::OutFlowType::Manual,
            inventory:     Vec::new(),
        });

        let s0 = SingleEntry {
            new_uuid:     types::UuidType([0; 16]),
            account:      account_uuid.clone(),
            is_debit:     Some(true),
            is_inflow:    Some(false),
            inflow_type:  None,
            outflow_type: None,
            amount:       Some(100.0),
            quantity:     Some(10.0), // price=10
        };
        let s1 = SingleEntry {
            new_uuid:     types::UuidType([1; 16]),
            account:      account_uuid.clone(),
            is_debit:     Some(true),
            is_inflow:    Some(false),
            inflow_type:  None,
            outflow_type: None,
            amount:       Some(200.0),
            quantity:     Some(20.0), // price=10
        };
        let s2 = SingleEntry {
            new_uuid:     types::UuidType([2; 16]),
            account:      account_uuid.clone(),
            is_debit:     Some(true),
            is_inflow:    Some(false),
            inflow_type:  None,
            outflow_type: None,
            amount:       None,
            quantity:     Some(5.0),
        };
        let entry = vec![vec![s0, s1, s2]];
        let mut err = dummy_input_with_double_entries(&entry);

        let resolved = map_input_type_to_middel_type(&mut err, entry, &accounts_info);
        let m2 = &resolved[0][2];
        assert_eq!(m2.amount, 50.0); // 5 * 10
        assert_eq!(m2.quantity, 5.0);
        assert!(!err.double_entries[0].single_entry_errors[2].amount_missing);
    }

    // -------------------------------------------------------------------------
    // 12. No price sample – no inference, flags remain
    // -------------------------------------------------------------------------
    #[test]
    fn test_cross_infer_no_price_sample() {
        let account_uuid = types::UuidType([1; 16]);
        let mut accounts_info = HashMap::new();
        accounts_info.insert(account_uuid.clone(), AccountInfo {
            is_debit:      true,
            in_flow_type:  accounting_stuff::InFlowType::Manual,
            out_flow_type: accounting_stuff::OutFlowType::Manual,
            inventory:     Vec::new(),
        });

        let s0 = SingleEntry {
            new_uuid:     types::UuidType([0; 16]),
            account:      account_uuid.clone(),
            is_debit:     Some(true),
            is_inflow:    Some(false),
            inflow_type:  None,
            outflow_type: None,
            amount:       Some(100.0),
            quantity:     None, // no price
        };
        let s1 = SingleEntry {
            new_uuid:     types::UuidType([1; 16]),
            account:      account_uuid.clone(),
            is_debit:     Some(true),
            is_inflow:    Some(false),
            inflow_type:  None,
            outflow_type: None,
            amount:       None,
            quantity:     Some(5.0),
        };
        let entry = vec![vec![s0, s1]];
        let mut err = dummy_input_with_double_entries(&entry);

        let resolved = map_input_type_to_middel_type(&mut err, entry, &accounts_info);
        let m1 = &resolved[0][1];
        assert_eq!(m1.amount, 0.0);
        assert_eq!(m1.quantity, 5.0);
        assert!(err.double_entries[0].single_entry_errors[1].amount_missing);
    }

    // -------------------------------------------------------------------------
    // 13. Zero quantity – not used as price sample
    // -------------------------------------------------------------------------
    #[test]
    fn test_cross_infer_zero_quantity_not_used_as_price() {
        let account_uuid = types::UuidType([1; 16]);
        let mut accounts_info = HashMap::new();
        accounts_info.insert(account_uuid.clone(), AccountInfo {
            is_debit:      true,
            in_flow_type:  accounting_stuff::InFlowType::Manual,
            out_flow_type: accounting_stuff::OutFlowType::Manual,
            inventory:     Vec::new(),
        });

        let s0 = SingleEntry {
            new_uuid:     types::UuidType([0; 16]),
            account:      account_uuid.clone(),
            is_debit:     Some(true),
            is_inflow:    Some(false),
            inflow_type:  None,
            outflow_type: None,
            amount:       Some(100.0),
            quantity:     Some(0.0), // zero → not a valid price
        };
        let s1 = SingleEntry {
            new_uuid:     types::UuidType([1; 16]),
            account:      account_uuid.clone(),
            is_debit:     Some(true),
            is_inflow:    Some(false),
            inflow_type:  None,
            outflow_type: None,
            amount:       None,
            quantity:     Some(5.0),
        };
        let entry = vec![vec![s0, s1]];
        let mut err = dummy_input_with_double_entries(&entry);

        let resolved = map_input_type_to_middel_type(&mut err, entry, &accounts_info);
        let m1 = &resolved[0][1];
        assert_eq!(m1.amount, 0.0);
        assert!(err.double_entries[0].single_entry_errors[1].amount_missing);
    }

    // -------------------------------------------------------------------------
    // 14. Empty double entry – skipped
    // -------------------------------------------------------------------------
    #[test]
    fn test_empty_double_entry() {
        let accounts_info = HashMap::new();
        let entry = vec![vec![]];
        let mut err = dummy_input_with_double_entries(&entry);

        let resolved = map_input_type_to_middel_type(&mut err, entry, &accounts_info);
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].len(), 0);
        // No errors because we don't allocate single_entry_errors for empty double?
        // Actually init_error_sink allocates for each double, but with 0 singles.
        // So there is no error slot.
    }

    // -------------------------------------------------------------------------
    // 15. Multiple double entries
    // -------------------------------------------------------------------------
    #[test]
    fn test_multiple_double_entries() {
        let account_uuid = types::UuidType([1; 16]);
        let mut accounts_info = HashMap::new();
        accounts_info.insert(account_uuid.clone(), AccountInfo {
            is_debit:      true,
            in_flow_type:  accounting_stuff::InFlowType::Manual,
            out_flow_type: accounting_stuff::OutFlowType::Manual,
            inventory:     Vec::new(),
        });

        let s0 = SingleEntry {
            new_uuid:     types::UuidType([0; 16]),
            account:      account_uuid.clone(),
            is_debit:     Some(true),
            is_inflow:    Some(false),
            inflow_type:  None,
            outflow_type: None,
            amount:       Some(10.0),
            quantity:     Some(1.0),
        };
        let s1 = SingleEntry {
            new_uuid:     types::UuidType([1; 16]),
            account:      account_uuid.clone(),
            is_debit:     Some(true),
            is_inflow:    Some(false),
            inflow_type:  None,
            outflow_type: None,
            amount:       Some(20.0),
            quantity:     Some(2.0),
        };
        let entry = vec![vec![s0], vec![s1]];
        let mut err = dummy_input_with_double_entries(&entry);

        let resolved = map_input_type_to_middel_type(&mut err, entry, &accounts_info);
        assert_eq!(resolved.len(), 2);
        assert_eq!(resolved[0].len(), 1);
        assert_eq!(resolved[1].len(), 1);
        assert_eq!(resolved[0][0].amount, 10.0);
        assert_eq!(resolved[1][0].amount, 20.0);
    }

    // -------------------------------------------------------------------------
    // 16. Credit nature account
    // -------------------------------------------------------------------------
    #[test]
    fn test_credit_nature_account() {
        let account_uuid = types::UuidType([5; 16]); // use account 5 from the ten accounts (credit nature)
        let accounts_info = create_ten_accounts_info();
        assert!(accounts_info.contains_key(&account_uuid));

        let single = SingleEntry {
            new_uuid:     types::UuidType([0; 16]),
            account:      account_uuid.clone(),
            is_debit:     None,
            is_inflow:    Some(true),
            inflow_type:  None,
            outflow_type: None,
            amount:       Some(100.0),
            quantity:     Some(10.0),
        };
        let entry = vec![vec![single]];
        let mut err = dummy_input_with_double_entries(&entry);

        let resolved = map_input_type_to_middel_type(&mut err, entry, &accounts_info);
        let m = &resolved[0][0];
        // nature=credit, is_inflow=true → is_debit = false
        assert_eq!(m.is_debit, false);
    }

    // -------------------------------------------------------------------------
    // 17. Flow types fallback to account defaults (using account 1)
    // -------------------------------------------------------------------------
    #[test]
    fn test_flow_types_fallback_to_defaults() {
        let account_uuid = types::UuidType([1; 16]); // account with QuantityEqualAmount & Wac
        let accounts_info = create_ten_accounts_info();
        assert!(accounts_info.contains_key(&account_uuid));

        let single = SingleEntry {
            new_uuid:     types::UuidType([0; 16]),
            account:      account_uuid.clone(),
            is_debit:     Some(true),
            is_inflow:    Some(false),
            inflow_type:  None, // should fallback to QuantityEqualAmount
            outflow_type: None, // should fallback to Wac
            amount:       Some(100.0),
            quantity:     Some(10.0),
        };
        let entry = vec![vec![single]];
        let mut err = dummy_input_with_double_entries(&entry);

        let resolved = map_input_type_to_middel_type(&mut err, entry, &accounts_info);
        let m = &resolved[0][0];
        assert_eq!(m.inflow_type, accounting_stuff::InFlowType::QuantityEqualAmount);
        assert_eq!(m.outflow_type, accounting_stuff::OutFlowType::Wac);
    }

    // -------------------------------------------------------------------------
    // 18. User‑provided flow types override defaults (using account 1)
    // -------------------------------------------------------------------------
    #[test]
    fn test_user_flow_types_override_defaults() {
        let account_uuid = types::UuidType([1; 16]); // account with QuantityEqualAmount & Wac
        let accounts_info = create_ten_accounts_info();
        assert!(accounts_info.contains_key(&account_uuid));

        let single = SingleEntry {
            new_uuid:     types::UuidType([0; 16]),
            account:      account_uuid.clone(),
            is_debit:     Some(true),
            is_inflow:    Some(false),
            inflow_type:  Some(accounting_stuff::InFlowType::Manual),
            outflow_type: Some(accounting_stuff::OutFlowType::Fifo),
            amount:       Some(100.0),
            quantity:     Some(10.0),
        };
        let entry = vec![vec![single]];
        let mut err = dummy_input_with_double_entries(&entry);

        let resolved = map_input_type_to_middel_type(&mut err, entry, &accounts_info);
        let m = &resolved[0][0];
        assert_eq!(m.inflow_type, accounting_stuff::InFlowType::Manual);
        assert_eq!(m.outflow_type, accounting_stuff::OutFlowType::Fifo);
    }

    // -------------------------------------------------------------------------
    // 19. Cross‑inference with quantity=0 target – should infer amount=0 and clear error
    // -------------------------------------------------------------------------
    #[test]
    fn test_cross_infer_with_zero_quantity_target() {
        let account_uuid = types::UuidType([1; 16]);
        let mut accounts_info = HashMap::new();
        accounts_info.insert(account_uuid.clone(), AccountInfo {
            is_debit:      true,
            in_flow_type:  accounting_stuff::InFlowType::Manual,
            out_flow_type: accounting_stuff::OutFlowType::Manual,
            inventory:     Vec::new(),
        });

        let s0 = SingleEntry {
            new_uuid:     types::UuidType([0; 16]),
            account:      account_uuid.clone(),
            is_debit:     Some(true),
            is_inflow:    Some(false),
            inflow_type:  None,
            outflow_type: None,
            amount:       Some(100.0),
            quantity:     Some(10.0), // price = 10
        };
        let s1 = SingleEntry {
            new_uuid:     types::UuidType([1; 16]),
            account:      account_uuid.clone(),
            is_debit:     Some(true),
            is_inflow:    Some(false),
            inflow_type:  None,
            outflow_type: None,
            amount:       None,
            quantity:     Some(0.0), // zero quantity
        };
        let entry = vec![vec![s0, s1]];
        let mut err = dummy_input_with_double_entries(&entry);

        let resolved = map_input_type_to_middel_type(&mut err, entry, &accounts_info);
        let m1 = &resolved[0][1];
        // amount = 0.0 * 10 = 0.0, and we clear the missing flag.
        assert_eq!(m1.amount, 0.0);
        assert_eq!(m1.quantity, 0.0);
        assert!(!err.double_entries[0].single_entry_errors[1].amount_missing);
    }

    // -------------------------------------------------------------------------
    // 20. Both is_debit and is_inflow missing, and amount/quantity missing – multiple errors
    // -------------------------------------------------------------------------
    #[test]
    fn test_multiple_errors_combined() {
        let account_uuid = types::UuidType([1; 16]);
        let mut accounts_info = HashMap::new();
        accounts_info.insert(account_uuid.clone(), AccountInfo {
            is_debit:      true,
            in_flow_type:  accounting_stuff::InFlowType::Manual,
            out_flow_type: accounting_stuff::OutFlowType::Manual,
            inventory:     Vec::new(),
        });

        let single = SingleEntry {
            new_uuid:     types::UuidType([0; 16]),
            account:      account_uuid.clone(),
            is_debit:     None,
            is_inflow:    None,
            inflow_type:  None,
            outflow_type: None,
            amount:       None,
            quantity:     None,
        };
        let entry = vec![vec![single]];
        let mut err = dummy_input_with_double_entries(&entry);

        let resolved = map_input_type_to_middel_type(&mut err, entry, &accounts_info);
        assert_eq!(resolved[0].len(), 1);
        let m = &resolved[0][0];
        // Fallbacks: is_debit = true, amount=0, quantity=0
        assert_eq!(m.is_debit, true);
        assert_eq!(m.amount, 0.0);
        assert_eq!(m.quantity, 0.0);
        // All errors should be set
        let se = &err.double_entries[0].single_entry_errors[0];
        assert!(se.is_debit_or_inflow_missing);
        assert!(se.amount_missing);
        assert!(se.quantity_missing);
    }

    // -------------------------------------------------------------------------
    // 21. Test with multiple accounts from the ten-account set,
    //     each with missing flow types – ensure each gets its correct default.
    // -------------------------------------------------------------------------
    #[test]
    fn test_ten_accounts_each_with_missing_flow_types() {
        let accounts_info = create_ten_accounts_info();
        let mut singles = Vec::new();

        // For each account, create a SingleEntry with no flow types, but with amount/quantity.
        for (i, (uuid, info)) in accounts_info.iter().enumerate() {
            let single = SingleEntry {
                new_uuid:     types::UuidType([i as u8; 16]),
                account:      uuid.clone(),
                is_debit:     Some(true), // we'll set is_debit to avoid inference, focus on flow types
                is_inflow:    Some(false),
                inflow_type:  None,
                outflow_type: None,
                amount:       Some(100.0 + i as f64),
                quantity:     Some(10.0 + i as f64),
            };
            singles.push(single);
        }

        let entry = vec![singles];
        let mut err = dummy_input_with_double_entries(&entry);

        let resolved = map_input_type_to_middel_type(&mut err, entry, &accounts_info);

        // Check each resolved single has the correct flow types from account info.
        for (i, single) in resolved[0].iter().enumerate() {
            let expected_info = accounts_info.get(&single.account).unwrap();
            assert_eq!(single.inflow_type, expected_info.in_flow_type);
            assert_eq!(single.outflow_type, expected_info.out_flow_type);
        }

        assert!(!err.is_there_error(), "No errors expected");
    }
}
