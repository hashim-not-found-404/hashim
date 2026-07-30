/*
amount >  0 && quantity >  0 : normal     : this is normal like the inflow to the account
amount >  0 && quantity == 0 : rare       : but it cuse to adjust the inventory like feeding sheep
amount >  0 && quantity <  0 : impossible : this is impossible in real life
amount == 0 && quantity >  0 : rare       : like gift but i dont want this to happen because it will lead to decrease the quantity without any amount and that will make some entry verbose
amount == 0 && quantity == 0 : impossible : this is not make sense because it not make any change
amount == 0 && quantity <  0 : rare       : it happens when the account is dont have any amount in the balance because it came from gifts or we make the amount 0
amount <  0 && quantity >  0 : impossible : this is impossible in real life
amount <  0 && quantity == 0 : rare       : but it cuse to adjust the inventory: like smashing a car or depreciation or market value
amount <  0 && quantity <  0 : normal     : this is normal like the outflow to the account
*/

use crate::accounting_domain::utility::common_subset_sum;
use crate::accounting_domain::utility::types;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;
use std::hash::Hash;

// -----------------------------------------------------------------------------
// Core domain types (value objects, independent of data layout)
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum CostFlowType {
    InFlow(InFlowType),
    OutFlow(OutFlowType),
}

#[derive(PartialEq, Debug, Deserialize, Serialize, Clone, Default)]
pub enum OutFlowType {
    Manual, // reorderable
    QuantityEqualAmount,
    QuantityEqualZero,
    #[default]
    Wac, // reorderable
    Fifo, // sortable
    Lifo, // sortable
    Hifo, // sortable
    Lofo, // sortable
}

impl OutFlowType {
    pub fn as_str(&self) -> &'static str {
        match self {
            OutFlowType::Manual => "Manual",
            OutFlowType::QuantityEqualAmount => "QuantityEqualAmount",
            OutFlowType::QuantityEqualZero => "QuantityEqualZero",
            OutFlowType::Wac => "Wac",
            OutFlowType::Fifo => "Fifo",
            OutFlowType::Lifo => "Lifo",
            OutFlowType::Hifo => "Hifo",
            OutFlowType::Lofo => "Lofo",
        }
    }
}

impl std::str::FromStr for OutFlowType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Manual" => Ok(OutFlowType::Manual),
            "QuantityEqualAmount" => Ok(OutFlowType::QuantityEqualAmount),
            "QuantityEqualZero" => Ok(OutFlowType::QuantityEqualZero),
            "Wac" => Ok(OutFlowType::Wac),
            "Fifo" => Ok(OutFlowType::Fifo),
            "Lifo" => Ok(OutFlowType::Lifo),
            "Hifo" => Ok(OutFlowType::Hifo),
            "Lofo" => Ok(OutFlowType::Lofo),
            _ => Err("unknown OutFlowType".into()),
        }
    }
}

#[derive(PartialEq, Debug, Deserialize, Serialize, Clone, Default)]
pub enum InFlowType {
    #[default]
    Manual,
    QuantityEqualAmount,
    QuantityEqualZero,
}

impl InFlowType {
    pub fn as_str(&self) -> &'static str {
        match self {
            InFlowType::Manual => "Manual",
            InFlowType::QuantityEqualAmount => "QuantityEqualAmount",
            InFlowType::QuantityEqualZero => "QuantityEqualZero",
        }
    }
}

impl std::str::FromStr for InFlowType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Manual" => Ok(InFlowType::Manual),
            "QuantityEqualAmount" => Ok(InFlowType::QuantityEqualAmount),
            "QuantityEqualZero" => Ok(InFlowType::QuantityEqualZero),
            _ => Err("unknown InFlowType".into()),
        }
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Nature {
    Debit,
    Credit,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct InventoryRecord {
    pub(crate) time_unix: u64,
    pub(crate) quantity:  f64,
    pub(crate) amount:    f64,
}

// -----------------------------------------------------------------------------
// Error types (serializable for API responses)
// -----------------------------------------------------------------------------

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

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub(crate) struct SingleEntryError {
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
pub(crate) struct DoubleEntryError {
    entry_is_empty:              bool,
    you_need_to_split_the_entry: bool,
    debit_not_equal_credit:      Option<DebitNotEqualCreditError>,
    single_entry_errors:         Vec<SingleEntryError>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub(crate) struct EntryError {
    entry_is_empty:      bool,
    double_entry_errors: Vec<DoubleEntryError>,
}

impl types::MyErrorTrait for DoubleEntryError {
    fn is_there_error(&self) -> bool {
        if self.entry_is_empty || self.debit_not_equal_credit.is_some() {
            return true;
        }

        for line in self.single_entry_errors.iter() {
            if *line != Default::default() {
                return true;
            }
        }

        false
    }
}
// -----------------------------------------------------------------------------
// Trait definitions – the core abstraction for accounting logic
// -----------------------------------------------------------------------------

/// Represents a single entry line (e.g., a line in a journal entry).
pub trait SingleEntry {
    type AccountId: Eq + Hash + Clone;

    fn account_id(&self) -> &Self::AccountId;
    fn is_debit(&self) -> bool;
    fn quantity(&self) -> f64;
    fn amount(&self) -> f64;
    fn flow_type(&self) -> (InFlowType, OutFlowType);
}

pub trait DoubleEntry {
    type Single: SingleEntry;
    type Iter<'a>: Iterator<Item = &'a Self::Single> + ExactSizeIterator
    where
        Self: 'a;

    fn iter(&self) -> Self::Iter<'_>;
    fn is_empty(&self) -> bool;
    fn len(&self) -> usize;
}

/// A container of single entries (e.g., a double‑entry group or a whole journal entry).
pub trait EntryContainer {
    type Double: DoubleEntry;
    type Iter<'a>: Iterator<Item = &'a Self::Double> + ExactSizeIterator
    where
        Self: 'a;

    fn iter(&self) -> Self::Iter<'_>;
    fn is_empty(&self) -> bool;
    fn len(&self) -> usize;
}

/// Provides account information (nature) and inventory for a given account.
pub trait AccountInfoProvider {
    type AccountId: Eq + Hash + Clone;
    type Inventory: Inventory;

    fn is_debit_nature(&self, id: &Self::AccountId) -> bool;
    fn get_or_create_inventory(&mut self, id: &Self::AccountId) -> &mut Self::Inventory;
}

/// A collection of inventory records, with operations needed for accounting.
pub trait Inventory {
    fn push(&mut self, record: InventoryRecord);
    fn clear(&mut self);
    fn is_empty(&self) -> bool;
    fn iter(&self) -> impl Iterator<Item = &InventoryRecord>;
    fn iter_mut(&mut self) -> impl Iterator<Item = &mut InventoryRecord>;
    fn sort_by<F>(&mut self, compare: F)
    where
        F: FnMut(&InventoryRecord, &InventoryRecord) -> std::cmp::Ordering;
    fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&InventoryRecord) -> bool;
    fn pop(&mut self) -> Option<InventoryRecord>;
}

// -----------------------------------------------------------------------------
// Helper functions (non‑generic)
// -----------------------------------------------------------------------------

pub(crate) fn is_debit(is_debit_nature: bool, is_inflow: bool) -> bool {
    match (is_debit_nature, is_inflow) {
        (true, true) => true,
        (true, false) => false,
        (false, true) => false,
        (false, false) => true,
    }
}

pub(crate) fn is_inflow(is_debit_nature: bool, is_debit_state: bool) -> bool {
    match (is_debit_nature, is_debit_state) {
        (true, true) => true,
        (true, false) => false,
        (false, true) => false,
        (false, false) => true,
    }
}

fn price(amount: f64, quantity: f64) -> f64 {
    amount / quantity
}

// -----------------------------------------------------------------------------
// Generic accounting functions
// -----------------------------------------------------------------------------

/// State‑less check: only checks the entries themselves, no inventory/account info needed.
pub(crate) fn state_less_check_for_entry<C>(entry: &C) -> EntryError
where
    C: EntryContainer,
    C::Double: DoubleEntry,
    <C::Double as DoubleEntry>::Single: SingleEntry,
{
    let mut entry_err = EntryError::default();
    if entry.is_empty() {
        entry_err.entry_is_empty = true;
        return entry_err;
    }

    for double in entry.iter() {
        let mut double_err = DoubleEntryError::default();
        if double.is_empty() {
            double_err.entry_is_empty = true;
            entry_err.double_entry_errors.push(double_err);
            continue;
        }

        double_err.single_entry_errors = Vec::with_capacity(entry.len());
        let mut seen_accounts = HashSet::with_capacity(entry.len());

        let mut total_debit = 0.0;
        let mut total_credit = 0.0;

        let mut debit_side = Vec::new();
        let mut credit_side = Vec::new();

        for single in double.iter() {
            let mut single_err = SingleEntryError::default();

            if !seen_accounts.insert(single.account_id().clone()) {
                single_err.duplicate_account_in_entry = true;
            }

            if single.amount() == 0.0 && single.quantity() == 0.0 {
                single_err.quantity_and_amount_are_zero = true;
            }
            if single.amount() < 0.0 {
                single_err.the_amount_should_be_positive = true;
            }
            if single.quantity() < 0.0 {
                single_err.the_quantity_should_be_positive = true;
            }

            if single.is_debit() {
                total_debit += single.amount();
                debit_side.push(single);
            } else {
                total_credit += single.amount();
                credit_side.push(single);
            }

            double_err.single_entry_errors.push(single_err);
        }

        if total_debit != total_credit {
            double_err.debit_not_equal_credit = Some(DebitNotEqualCreditError {
                total_debit,
                total_credit,
            });
        } else {
            let a = common_subset_sum::split_to_max(&debit_side, &credit_side, &|a| {
                wrapper::T(a.amount())
            });

            if a.len() > 1 {
                double_err.you_need_to_split_the_entry = true
            }
        }

        entry_err.double_entry_errors.push(double_err);
    }

    entry_err
}

/// Full validation including account info and inventory.
pub(crate) fn state_full_check_for_entry<C, A>(
    time_unix: u64,
    entry: &C,
    account_info: &mut A,
) -> EntryError
where
    C: EntryContainer,
    C::Double: DoubleEntry,
    <C::Double as DoubleEntry>::Single: SingleEntry,
    A: AccountInfoProvider<
        AccountId = <<C::Double as DoubleEntry>::Single as SingleEntry>::AccountId,
    >,
    A::Inventory: Inventory,
{
    let mut entry_err = EntryError::default();
    entry_err.double_entry_errors = Vec::with_capacity(entry.len());

    for double in entry.iter() {
        let mut double_err = DoubleEntryError::default();
        double_err.single_entry_errors = Vec::with_capacity(double.len());

        for single in double.iter() {
            let mut single_err = SingleEntryError::default();

            let account_id = single.account_id();
            let nature = account_info.is_debit_nature(account_id);
            let (in_flow_type, out_flow_type) = single.flow_type();
            let inventory = account_info.get_or_create_inventory(account_id);

            // Check if inventory is empty
            if inventory.is_empty() {
                single_err.inventory_is_empty = true;
            }

            let is_inflow = is_inflow(nature, single.is_debit());

            let (amt, qty) = match is_inflow {
                true => (single.amount(), single.quantity()),
                false => (-single.amount(), -single.quantity()),
            };

            if is_inflow {
                match in_flow_type {
                    InFlowType::Manual => {}
                    InFlowType::QuantityEqualAmount => {
                        if qty != amt {
                            single_err.quantity_not_equal_amount = true;
                        }
                    }
                    InFlowType::QuantityEqualZero => {
                        if qty != 0.0 {
                            single_err.quantity_not_equal_zero = true;
                        }
                    }
                }
            } else {
                let (total_qty, total_amt) = sum_inventory(inventory);
                // Check sufficient amount
                if total_amt + amt < 0.0 {
                    single_err.insufficient_amount_in_inventory =
                        Some(InsufficientAmountInInventory {
                            total_amount: total_amt,
                        });
                }
                if total_qty + qty < 0.0 {
                    single_err.insufficient_quantity_in_inventory =
                        Some(InsufficientQuantityInInventory {
                            total_quantity: total_qty,
                        });
                }

                // Sort inventory for the flow type
                sort_inventory(&out_flow_type, inventory);

                // Check expected amount based on cost method
                match out_flow_type {
                    OutFlowType::Wac
                    | OutFlowType::Fifo
                    | OutFlowType::Lifo
                    | OutFlowType::Hifo
                    | OutFlowType::Lofo
                    | OutFlowType::Manual => {
                        let expected_amount = get_amount(single.quantity(), inventory);
                        if expected_amount != single.amount() {
                            single_err.amount_mismatch = Some(AmountMismatch {
                                expected_amount,
                            });
                        }
                    }
                    OutFlowType::QuantityEqualAmount => {
                        if qty != amt {
                            single_err.quantity_not_equal_amount = true;
                        }
                    }
                    OutFlowType::QuantityEqualZero => {
                        if qty != 0.0 {
                            single_err.quantity_not_equal_zero = true;
                        }
                    }
                }
            }

            apply_entry_on_inventory::<<C::Double as DoubleEntry>::Single, A::Inventory>(
                time_unix,
                single.amount(),
                single.quantity(),
                is_inflow,
                inventory,
            );

            double_err.single_entry_errors.push(single_err);
        }
        entry_err.double_entry_errors.push(double_err);
    }

    entry_err
}

/// Apply the entry to the inventory, updating records.
pub fn apply_entry_on_inventory<S, I>(
    time_unix: u64,
    amount: f64,
    quantity: f64,
    is_inflow: bool,
    inventory: &mut I,
) where
    S: SingleEntry,
    I: Inventory,
{
    let (amt, qty) = match is_inflow {
        true => (amount.abs(), quantity.abs()),
        false => (-amount.abs(), -quantity.abs()),
    };

    if amt > 0.0 && qty > 0.0 {
        inventory.push(InventoryRecord {
            time_unix,
            quantity,
            amount,
        });
    } else if (amt == 0.0) != (qty == 0.0) {
        let (total_qty, total_amt) = sum_inventory(inventory);
        if total_qty + qty == 0.0 && total_amt + amt == 0.0 {
            inventory.clear();
        } else {
            inventory.clear();
            inventory.push(InventoryRecord {
                time_unix,
                quantity: total_qty + qty,
                amount: total_amt + amt,
            });
        }
    } else if amt < 0.0 && qty < 0.0 {
        decrease_inventory(quantity, inventory);
    } else {
        unreachable!();
    }
}

// -----------------------------------------------------------------------------
// Inventory helper functions (generic over Inventory)
// -----------------------------------------------------------------------------

pub fn sum_inventory<I: Inventory>(inventory: &I) -> (f64, f64) {
    let mut total_qty = 0.0;
    let mut total_amt = 0.0;
    for record in inventory.iter() {
        total_qty += record.quantity;
        total_amt += record.amount;
    }
    (total_qty, total_amt)
}

pub fn combine_all_inventory_record_in_one_record<I: Inventory>(inventory: &mut I) {
    let mut total = InventoryRecord {
        time_unix: 0,
        quantity:  0.0,
        amount:    0.0,
    };
    // We need to iterate and accumulate. We cannot remove while iterating,
    // so we first collect and then clear and push.
    for record in inventory.iter() {
        total.quantity += record.quantity;
        total.amount += record.amount;
        if record.time_unix > total.time_unix {
            total.time_unix = record.time_unix;
        }
    }
    inventory.clear();
    inventory.push(total);
}

pub fn sort_inventory<I: Inventory>(out_flow_type: &OutFlowType, inventory: &mut I) {
    match out_flow_type {
        OutFlowType::QuantityEqualAmount
        | OutFlowType::QuantityEqualZero
        | OutFlowType::Manual
        | OutFlowType::Wac => {
            combine_all_inventory_record_in_one_record(inventory);
        }
        OutFlowType::Fifo => {
            inventory.sort_by(|a, b| a.time_unix.cmp(&b.time_unix));
        }
        OutFlowType::Lifo => {
            inventory.sort_by(|a, b| b.time_unix.cmp(&a.time_unix));
        }
        OutFlowType::Hifo => {
            inventory.sort_by(|a, b| {
                price(b.amount, b.quantity)
                    .partial_cmp(&price(a.amount, a.quantity))
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        OutFlowType::Lofo => {
            inventory.sort_by(|a, b| {
                price(a.amount, a.quantity)
                    .partial_cmp(&price(b.amount, b.quantity))
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }
    }
}

pub fn get_amount<I: Inventory>(quantity: f64, inventory: &I) -> f64 {
    let mut remaining = quantity;
    let mut accumulator = 0.0;

    // FIFO order (assuming inventory is sorted appropriately)
    for record in inventory.iter() {
        if record.quantity <= remaining {
            remaining -= record.quantity;
            accumulator += record.amount;
        } else {
            accumulator += remaining * price(record.amount, record.quantity);
            break;
        }
    }
    accumulator
}

pub fn decrease_inventory<I: Inventory>(quantity: f64, inventory: &mut I) {
    let mut remaining = quantity;
    let mut to_remove = 0;

    // FIFO order
    for record in inventory.iter_mut() {
        if record.quantity <= remaining {
            remaining -= record.quantity;
            to_remove += 1;
        } else {
            record.quantity -= remaining;
            record.amount = record.quantity * price(record.amount, record.quantity);
            break;
        }
    }

    for _ in 0..to_remove {
        inventory.pop();
    }
}

mod wrapper {
    use std::iter::Sum;
    use std::ops::Add;
    use std::ops::Sub;

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub(crate) struct T(pub f64);

    impl Eq for T {}
    impl std::hash::Hash for T {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.0.to_bits().hash(state);
        }
    }

    impl Add for T {
        type Output = Self;

        fn add(self, other: Self) -> Self {
            T(self.0 + other.0)
        }
    }
    impl Sub for T {
        type Output = Self;

        fn sub(self, other: Self) -> Self {
            T(self.0 - other.0)
        }
    }
    impl Default for T {
        fn default() -> Self {
            T(0.0)
        }
    }
    impl Sum for T {
        fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
            iter.fold(T::default(), |acc, x| acc + x)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AccountInfoProvider;
    use super::DoubleEntry;
    use super::EntryContainer;
    use super::InFlowType;
    use super::Inventory;
    use super::InventoryRecord;
    use super::OutFlowType;
    use super::SingleEntry;
    use super::state_full_check_for_entry;
    use super::state_less_check_for_entry;
    use std::alloc::System;
    use std::collections::HashMap;
    use std::time::SystemTime;

    // -------------------------------------------------------------------------
    // Dummy implementations of traits for testing
    // -------------------------------------------------------------------------

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    struct AccountId(String);

    #[derive(Clone, Debug)]
    struct TestSingleEntry {
        account:  AccountId,
        debit:    bool,
        qty:      f64,
        amt:      f64,
        in_flow:  InFlowType,
        out_flow: OutFlowType,
    }

    impl SingleEntry for TestSingleEntry {
        type AccountId = AccountId;

        fn account_id(&self) -> &Self::AccountId {
            &self.account
        }

        fn is_debit(&self) -> bool {
            self.debit
        }

        fn quantity(&self) -> f64 {
            self.qty
        }

        fn amount(&self) -> f64 {
            self.amt
        }

        fn flow_type(&self) -> (InFlowType, OutFlowType) {
            (self.in_flow.clone(), self.out_flow.clone())
        }
    }

    #[derive(Clone, Debug)]
    struct TestDoubleEntry {
        lines: Vec<TestSingleEntry>,
    }

    impl DoubleEntry for TestDoubleEntry {
        type Iter<'a> = std::slice::Iter<'a, TestSingleEntry>;
        type Single = TestSingleEntry;

        fn iter(&self) -> Self::Iter<'_> {
            self.lines.iter()
        }

        fn is_empty(&self) -> bool {
            self.lines.is_empty()
        }

        fn len(&self) -> usize {
            self.lines.len()
        }
    }

    #[derive(Clone, Debug)]
    struct TestEntryContainer {
        groups: Vec<TestDoubleEntry>,
    }

    impl EntryContainer for TestEntryContainer {
        type Double = TestDoubleEntry;
        type Iter<'a> = std::slice::Iter<'a, TestDoubleEntry>;

        fn iter(&self) -> Self::Iter<'_> {
            self.groups.iter()
        }

        fn is_empty(&self) -> bool {
            self.groups.is_empty()
        }

        fn len(&self) -> usize {
            self.groups.len()
        }
    }

    // Dummy Inventory – a simple vector wrapper
    #[derive(Clone, Debug, Default)]
    struct TestInventory(Vec<InventoryRecord>);

    impl Inventory for TestInventory {
        fn push(&mut self, record: InventoryRecord) {
            self.0.push(record);
        }

        fn clear(&mut self) {
            self.0.clear();
        }

        fn is_empty(&self) -> bool {
            self.0.is_empty()
        }

        fn iter(&self) -> impl Iterator<Item = &InventoryRecord> {
            self.0.iter()
        }

        fn iter_mut(&mut self) -> impl Iterator<Item = &mut InventoryRecord> {
            self.0.iter_mut()
        }

        fn sort_by<F>(&mut self, compare: F)
        where
            F: FnMut(&InventoryRecord, &InventoryRecord) -> std::cmp::Ordering,
        {
            self.0.sort_by(compare)
        }

        fn retain<F>(&mut self, f: F)
        where
            F: FnMut(&InventoryRecord) -> bool,
        {
            self.0.retain(f)
        }

        fn pop(&mut self) -> Option<InventoryRecord> {
            self.0.pop()
        }
    }

    // AccountInfoProvider that stores nature and inventory per account
    struct TestAccountInfoProvider {
        natures:     HashMap<AccountId, bool>, // true = Debit nature
        inventories: HashMap<AccountId, TestInventory>,
    }

    impl TestAccountInfoProvider {
        fn new() -> Self {
            Self {
                natures:     HashMap::new(),
                inventories: HashMap::new(),
            }
        }

        fn add_account(&mut self, id: AccountId, is_debit_nature: bool) {
            self.natures.insert(id.clone(), is_debit_nature);
            self.inventories.entry(id).or_insert_with(TestInventory::default);
        }
    }

    impl AccountInfoProvider for TestAccountInfoProvider {
        type AccountId = AccountId;
        type Inventory = TestInventory;

        fn is_debit_nature(&self, id: &Self::AccountId) -> bool {
            *self.natures.get(id).unwrap_or(&true) // default to Debit
        }

        fn get_or_create_inventory(&mut self, id: &Self::AccountId) -> &mut Self::Inventory {
            self.inventories.entry(id.clone()).or_insert_with(TestInventory::default)
        }
    }

    // Helper to create a TestSingleEntry quickly
    fn entry(
        account: &str,
        debit: bool,
        qty: f64,
        amt: f64,
        in_flow: InFlowType,
        out_flow: OutFlowType,
    ) -> TestSingleEntry {
        TestSingleEntry {
            account: AccountId(account.to_string()),
            debit,
            qty,
            amt,
            in_flow,
            out_flow,
        }
    }

    fn simple_entry(account: &str, debit: bool, qty: f64, amt: f64) -> TestSingleEntry {
        entry(account, debit, qty, amt, InFlowType::Manual, OutFlowType::Manual)
    }

    // -------------------------------------------------------------------------
    // Tests for state_less_check_for_entry
    // -------------------------------------------------------------------------

    #[test]
    fn test_state_less_empty_entry() {
        let entry = TestEntryContainer {
            groups: vec![],
        };
        let err = state_less_check_for_entry(&entry);
        assert!(err.entry_is_empty);
        assert_eq!(err.double_entry_errors.len(), 0);
    }

    #[test]
    fn test_state_less_duplicate_account() {
        let single1 = simple_entry("A", true, 1.0, 10.0);
        let single2 = simple_entry("A", false, 1.0, 10.0); // same account
        let double = TestDoubleEntry {
            lines: vec![single1, single2],
        };
        let entry = TestEntryContainer {
            groups: vec![double],
        };
        let err = state_less_check_for_entry(&entry);
        assert!(!err.entry_is_empty);
        assert_eq!(err.double_entry_errors.len(), 1);
        let de = &err.double_entry_errors[0];
        assert!(!de.entry_is_empty);
        assert_eq!(de.single_entry_errors.len(), 2);
        assert!(de.single_entry_errors[1].duplicate_account_in_entry);
    }

    #[test]
    fn test_state_less_zero_qty_and_amount() {
        let single = entry("A", true, 0.0, 0.0, InFlowType::Manual, OutFlowType::Manual);
        let double = TestDoubleEntry {
            lines: vec![single],
        };
        let entry = TestEntryContainer {
            groups: vec![double],
        };
        let err = state_less_check_for_entry(&entry);
        let se = &err.double_entry_errors[0].single_entry_errors[0];
        assert!(se.quantity_and_amount_are_zero);
    }

    #[test]
    fn test_state_less_negative_values() {
        let single = simple_entry("A", true, -1.0, -5.0);
        let double = TestDoubleEntry {
            lines: vec![single],
        };
        let entry = TestEntryContainer {
            groups: vec![double],
        };
        let err = state_less_check_for_entry(&entry);
        let se = &err.double_entry_errors[0].single_entry_errors[0];
        assert!(se.the_amount_should_be_positive);
        assert!(se.the_quantity_should_be_positive);
    }

    #[test]
    fn test_state_less_debit_credit_mismatch() {
        // Debit total = 10, Credit total = 8
        let d1 = simple_entry("A", true, 1.0, 10.0);
        let c1 = simple_entry("B", false, 1.0, 8.0);
        let double = TestDoubleEntry {
            lines: vec![d1, c1],
        };
        let entry = TestEntryContainer {
            groups: vec![double],
        };
        let err = state_less_check_for_entry(&entry);
        let de = &err.double_entry_errors[0];
        assert!(de.debit_not_equal_credit.is_some());
        let dnc = de.debit_not_equal_credit.as_ref().unwrap();
        assert_eq!(dnc.total_debit, 10.0);
        assert_eq!(dnc.total_credit, 8.0);
    }

    #[test]
    fn test_state_less_splittable_entry() {
        // Debit: [A=1, D=4, E=5] total=10, Credit: [B=2, C=3, F=5] total=10
        // This can be split into 5=2+3 and 1+4=5, so you_need_to_split_the_entry = true
        let d1 = entry("A", true, 1.0, 1.0, InFlowType::Manual, OutFlowType::Manual);
        let d2 = entry("D", true, 1.0, 4.0, InFlowType::Manual, OutFlowType::Manual);
        let d3 = entry("E", true, 1.0, 5.0, InFlowType::Manual, OutFlowType::Manual);
        let c1 = entry("B", false, 1.0, 2.0, InFlowType::Manual, OutFlowType::Manual);
        let c2 = entry("C", false, 1.0, 3.0, InFlowType::Manual, OutFlowType::Manual);
        let c3 = entry("F", false, 1.0, 5.0, InFlowType::Manual, OutFlowType::Manual);
        let double = TestDoubleEntry {
            lines: vec![d1, d2, d3, c1, c2, c3],
        };
        let entry = TestEntryContainer {
            groups: vec![double],
        };
        let err = state_less_check_for_entry(&entry);
        let de = &err.double_entry_errors[0];
        assert!(de.you_need_to_split_the_entry);
    }

    #[test]
    fn test_state_less_non_splittable_entry() {
        // Debit: [1,2,6] total=9, Credit: [4,5] total=9 – atomic, no split
        let d1 = simple_entry("A", true, 1.0, 1.0);
        let d2 = simple_entry("B", true, 1.0, 2.0);
        let d3 = simple_entry("F", true, 1.0, 6.0);
        let c1 = simple_entry("D", false, 1.0, 4.0);
        let c2 = simple_entry("E", false, 1.0, 5.0);
        let double = TestDoubleEntry {
            lines: vec![d1, d2, d3, c1, c2],
        };
        let entry = TestEntryContainer {
            groups: vec![double],
        };
        let err = state_less_check_for_entry(&entry);
        let de = &err.double_entry_errors[0];
        assert!(!de.you_need_to_split_the_entry);
    }

    // -------------------------------------------------------------------------
    // Tests for state_full_check_for_entry
    // -------------------------------------------------------------------------

    fn setup_provider() -> TestAccountInfoProvider {
        let mut provider = TestAccountInfoProvider::new();
        // Add accounts with their nature (true = Debit)
        provider.add_account(AccountId("A".to_string()), true);
        provider.add_account(AccountId("B".to_string()), true);
        provider.add_account(AccountId("C".to_string()), true);
        provider.add_account(AccountId("D".to_string()), true);
        provider.add_account(AccountId("E".to_string()), true);
        provider.add_account(AccountId("F".to_string()), true);
        provider.add_account(AccountId("G".to_string()), false); // Credit nature
        provider
    }

    #[test]
    fn test_state_full_inventory_empty_error() {
        let mut provider = setup_provider();
        let single = simple_entry("A", true, 1.0, 10.0); // inflow, but inventory empty
        let double = TestDoubleEntry {
            lines: vec![single],
        };
        let entry = TestEntryContainer {
            groups: vec![double],
        };
        let err = state_full_check_for_entry(100000000000, &entry, &mut provider);
        let se = &err.double_entry_errors[0].single_entry_errors[0];
        assert!(se.inventory_is_empty);
    }

    #[test]
    fn test_state_full_quantity_equal_amount_inflow() {
        let mut provider = setup_provider();
        // First, add some inventory to avoid empty error
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  10.0,
            amount:    100.0,
        });

        // Now test inflow with QuantityEqualAmount
        let single =
            entry("A", true, 5.0, 5.0, InFlowType::QuantityEqualAmount, OutFlowType::Manual);
        let double = TestDoubleEntry {
            lines: vec![single],
        };
        let entry = TestEntryContainer {
            groups: vec![double],
        };
        let err = state_full_check_for_entry(100000000000, &entry, &mut provider);
        let se = &err.double_entry_errors[0].single_entry_errors[0];
        assert!(!se.quantity_not_equal_amount); // should be ok
    }

    #[test]
    fn test_state_full_quantity_equal_amount_mismatch() {
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  10.0,
            amount:    100.0,
        });

        let single = entry(
            "A",
            true,
            5.0,
            4.0, // qty != amount
            InFlowType::QuantityEqualAmount,
            OutFlowType::Manual,
        );
        let double = TestDoubleEntry {
            lines: vec![single],
        };
        let entry = TestEntryContainer {
            groups: vec![double],
        };
        let err = state_full_check_for_entry(100000000000, &entry, &mut provider);
        let se = &err.double_entry_errors[0].single_entry_errors[0];
        assert!(se.quantity_not_equal_amount);
    }

    #[test]
    fn test_state_full_quantity_equal_zero_inflow() {
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  10.0,
            amount:    100.0,
        });

        let single =
            entry("A", true, 0.0, 10.0, InFlowType::QuantityEqualZero, OutFlowType::Manual);
        let double = TestDoubleEntry {
            lines: vec![single],
        };
        let entry = TestEntryContainer {
            groups: vec![double],
        };
        let err = state_full_check_for_entry(100000000000, &entry, &mut provider);
        let se = &err.double_entry_errors[0].single_entry_errors[0];
        assert!(!se.quantity_not_equal_zero); // ok
    }

    #[test]
    fn test_state_full_quantity_equal_zero_mismatch() {
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  10.0,
            amount:    100.0,
        });

        let single = entry(
            "A",
            true,
            1.0, // qty != 0
            10.0,
            InFlowType::QuantityEqualZero,
            OutFlowType::Manual,
        );
        let double = TestDoubleEntry {
            lines: vec![single],
        };
        let entry = TestEntryContainer {
            groups: vec![double],
        };
        let err = state_full_check_for_entry(100000000000, &entry, &mut provider);
        let se = &err.double_entry_errors[0].single_entry_errors[0];
        assert!(se.quantity_not_equal_zero);
    }

    #[test]
    fn test_state_full_outflow_insufficient_amount() {
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        }); // total amt = 10

        // Outflow of 20.0 amount (debit nature, but outflow = Credit)
        let single = entry("A", false, 5.0, 20.0, InFlowType::Manual, OutFlowType::Manual);
        let double = TestDoubleEntry {
            lines: vec![single],
        };
        let entry = TestEntryContainer {
            groups: vec![double],
        };
        let err = state_full_check_for_entry(100000000000, &entry, &mut provider);
        let se = &err.double_entry_errors[0].single_entry_errors[0];
        assert!(se.insufficient_amount_in_inventory.is_some());
        let ia = se.insufficient_amount_in_inventory.as_ref().unwrap();
        assert_eq!(ia.total_amount, 10.0);
    }

    #[test]
    fn test_state_full_outflow_insufficient_quantity() {
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  2.0,
            amount:    10.0,
        });

        let single = entry(
            "A",
            false,
            5.0, // qty > 2
            20.0,
            InFlowType::Manual,
            OutFlowType::Manual,
        );
        let double = TestDoubleEntry {
            lines: vec![single],
        };
        let entry = TestEntryContainer {
            groups: vec![double],
        };
        let err = state_full_check_for_entry(100000000000, &entry, &mut provider);
        let se = &err.double_entry_errors[0].single_entry_errors[0];
        assert!(se.insufficient_quantity_in_inventory.is_some());
        let iq = se.insufficient_quantity_in_inventory.as_ref().unwrap();
        assert_eq!(iq.total_quantity, 2.0);
    }

    #[test]
    fn test_state_full_outflow_amount_mismatch_wac() {
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
        // One record: qty=10, amt=100 => price=10
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  10.0,
            amount:    100.0,
        });

        // Wac outflow: expected amount = quantity * (total_amt/total_qty) = 2*10 = 20
        // But we set amount=25 => mismatch
        let single = entry("A", false, 2.0, 25.0, InFlowType::Manual, OutFlowType::Wac);
        let double = TestDoubleEntry {
            lines: vec![single],
        };
        let entry = TestEntryContainer {
            groups: vec![double],
        };
        let err = state_full_check_for_entry(100000000000, &entry, &mut provider);
        let se = &err.double_entry_errors[0].single_entry_errors[0];
        assert!(se.amount_mismatch.is_some());
        let am = se.amount_mismatch.as_ref().unwrap();
        assert_eq!(am.expected_amount, 20.0);
    }

    #[test]
    fn test_state_full_outflow_amount_mismatch_fifo() {
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
        // Two records: first qty=5, amt=20 (price=4); second qty=3, amt=15 (price=5)
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    20.0,
        });
        inv.push(InventoryRecord {
            time_unix: 2,
            quantity:  3.0,
            amount:    15.0,
        });

        // Outflow 4 units: FIFO takes 4 from first record: amount = 4*4 = 16
        let single = entry(
            "A",
            false,
            4.0,
            20.0, // mismatch
            InFlowType::Manual,
            OutFlowType::Fifo,
        );
        let double = TestDoubleEntry {
            lines: vec![single],
        };
        let entry = TestEntryContainer {
            groups: vec![double],
        };
        let err = state_full_check_for_entry(100000000000, &entry, &mut provider);
        let se = &err.double_entry_errors[0].single_entry_errors[0];
        assert!(se.amount_mismatch.is_some());
        let am = se.amount_mismatch.as_ref().unwrap();
        assert_eq!(am.expected_amount, 16.0);
    }

    // Test state_full_check_for_entry with out_flow type QuantityEqualAmount and QuantityEqualZero for outflow.
    #[test]
    fn test_state_full_outflow_quantity_equal_amount() {
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  10.0,
            amount:    20.0,
        });

        // Outflow with QuantityEqualAmount: qty should equal amount
        let single =
            entry("A", false, 2.0, 2.0, InFlowType::Manual, OutFlowType::QuantityEqualAmount);
        let double = TestDoubleEntry {
            lines: vec![single],
        };
        let entry = TestEntryContainer {
            groups: vec![double],
        };
        let err = state_full_check_for_entry(100000000000, &entry, &mut provider);
        let se = &err.double_entry_errors[0].single_entry_errors[0];
        assert!(!se.quantity_not_equal_amount);
    }

    #[test]
    fn test_state_full_outflow_quantity_equal_amount_mismatch() {
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  10.0,
            amount:    20.0,
        });

        let single =
            entry("A", false, 2.0, 3.0, InFlowType::Manual, OutFlowType::QuantityEqualAmount);
        let double = TestDoubleEntry {
            lines: vec![single],
        };
        let entry = TestEntryContainer {
            groups: vec![double],
        };
        let err = state_full_check_for_entry(100000000000, &entry, &mut provider);
        let se = &err.double_entry_errors[0].single_entry_errors[0];
        assert!(se.quantity_not_equal_amount);
    }

    #[test]
    fn test_state_full_outflow_quantity_equal_zero() {
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  10.0,
            amount:    20.0,
        });

        let single =
            entry("A", false, 0.0, 5.0, InFlowType::Manual, OutFlowType::QuantityEqualZero);
        let double = TestDoubleEntry {
            lines: vec![single],
        };
        let entry = TestEntryContainer {
            groups: vec![double],
        };
        let err = state_full_check_for_entry(100000000000, &entry, &mut provider);
        let se = &err.double_entry_errors[0].single_entry_errors[0];
        assert!(!se.quantity_not_equal_zero);
    }

    #[test]
    fn test_state_full_outflow_quantity_equal_zero_mismatch() {
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  10.0,
            amount:    20.0,
        });

        let single =
            entry("A", false, 1.0, 5.0, InFlowType::Manual, OutFlowType::QuantityEqualZero);
        let double = TestDoubleEntry {
            lines: vec![single],
        };
        let entry = TestEntryContainer {
            groups: vec![double],
        };
        let err = state_full_check_for_entry(100000000000, &entry, &mut provider);
        let se = &err.double_entry_errors[0].single_entry_errors[0];
        assert!(se.quantity_not_equal_zero);
    }

    // Test that state_full_check_for_entry correctly handles the "inventory empty" check for outflow.
    #[test]
    fn test_state_full_outflow_empty_inventory() {
        let mut provider = setup_provider();
        // No inventory for account A
        let single = simple_entry("A", false, 1.0, 10.0);
        let double = TestDoubleEntry {
            lines: vec![single],
        };
        let entry = TestEntryContainer {
            groups: vec![double],
        };
        let err = state_full_check_for_entry(100000000000, &entry, &mut provider);
        let se = &err.double_entry_errors[0].single_entry_errors[0];
        assert!(se.inventory_is_empty);
    }
}
