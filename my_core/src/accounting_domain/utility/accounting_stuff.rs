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
use crate::accounting_domain::utility::types::MyErrorTrait;
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
        if self.entry_is_empty
            || self.you_need_to_split_the_entry
            || self.debit_not_equal_credit.is_some()
        {
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

impl types::MyErrorTrait for SingleEntryError {}

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
                    OutFlowType::Manual => {}
                    OutFlowType::Wac
                    | OutFlowType::Fifo
                    | OutFlowType::Lifo
                    | OutFlowType::Hifo
                    | OutFlowType::Lofo => {
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

            let is_decrease_by_price = match out_flow_type {
                OutFlowType::Manual => false,
                OutFlowType::QuantityEqualAmount => false,
                OutFlowType::QuantityEqualZero => false,
                OutFlowType::Wac => true,
                OutFlowType::Fifo => true,
                OutFlowType::Lifo => true,
                OutFlowType::Hifo => true,
                OutFlowType::Lofo => true,
            };

            if !single_err.is_there_error() {
                apply_entry_on_inventory::<<C::Double as DoubleEntry>::Single, A::Inventory>(
                    time_unix,
                    single.amount(),
                    single.quantity(),
                    is_inflow,
                    is_decrease_by_price,
                    inventory,
                );
            }

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
    is_decrease_by_price: bool,
    inventory: &mut I,
) where
    S: SingleEntry,
    I: Inventory,
{
    let (amt, qty) = match is_inflow {
        true => (amount, quantity),
        false => (-amount, -quantity),
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
        if is_decrease_by_price {
            decrease_inventory(quantity, inventory);
        } else {
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
        }
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

    for record in inventory.iter_mut() {
        if record.quantity <= remaining {
            remaining -= record.quantity;
            to_remove += 1;
        } else {
            let price = price(record.amount, record.quantity);
            record.quantity -= remaining;
            record.amount = record.quantity * price;
            break;
        }
    }

    let mut new_vec = Vec::new();
    let mut skip = to_remove;
    for record in inventory.iter() {
        if skip > 0 {
            skip -= 1;
        } else {
            new_vec.push(record.clone());
        }
    }
    inventory.clear();
    for record in new_vec {
        inventory.push(record);
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
    use super::DebitNotEqualCreditError;
    use super::DoubleEntry;
    use super::DoubleEntryError;
    use super::EntryContainer;
    use super::InFlowType;
    use super::Inventory;
    use super::InventoryRecord;
    use super::OutFlowType;
    use super::SingleEntry;
    use super::SingleEntryError;
    use super::apply_entry_on_inventory;
    use super::decrease_inventory;
    use super::get_amount;
    use super::sort_inventory;
    use super::state_full_check_for_entry;
    use super::state_less_check_for_entry;
    use super::sum_inventory;
    use crate::accounting_domain::utility::accounting_stuff::combine_all_inventory_record_in_one_record;
    use crate::accounting_domain::utility::types::MyErrorTrait;
    use std::collections::HashMap;

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

    #[test]
    fn test_decrease_inventory_fifo_correctness() {
        let mut inv = TestInventory::default();
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  2.0,
            amount:    10.0,
        });
        inv.push(InventoryRecord {
            time_unix: 2,
            quantity:  3.0,
            amount:    15.0,
        });
        inv.push(InventoryRecord {
            time_unix: 3,
            quantity:  5.0,
            amount:    25.0,
        });

        decrease_inventory(4.0, &mut inv);

        let (qty, _amt) = sum_inventory(&inv);
        assert_eq!(qty, 6.0); // 2+3+5 - 4 = 6
        // Check the remaining records: should be [B(qty=1), C(qty=5)]
        let records: Vec<_> = inv.iter().collect();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].quantity, 1.0);
        assert_eq!(records[0].amount, 5.0); // price 5
        assert_eq!(records[1].quantity, 5.0);
        assert_eq!(records[1].amount, 25.0);
    }

    #[test]
    fn test_state_full_does_not_mutate_on_error() {
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        });

        let single = entry("A", false, 5.0, 20.0, InFlowType::Manual, OutFlowType::Manual);
        let entry = TestEntryContainer {
            groups: vec![TestDoubleEntry {
                lines: vec![single],
            }],
        };

        state_full_check_for_entry(100, &entry, &mut provider);

        // Inventory should STILL be exactly as before (10.0 amount, 5 qty)
        let (qty, amt) = sum_inventory(&provider.inventories[&AccountId("A".to_string())]);
        assert_eq!(qty, 5.0);
        assert_eq!(amt, 10.0);
    }

    // -------------------------------------------------------------------------
    // Additional tests for state_less_check_for_entry
    // -------------------------------------------------------------------------

    #[test]
    fn test_state_less_duplicate_account_with_three_entries() {
        // Account A appears at index 0 and 2; only the second occurrence should be flagged
        let e1 = simple_entry("A", true, 1.0, 10.0);
        let e2 = simple_entry("B", false, 1.0, 5.0);
        let e3 = simple_entry("A", true, 1.0, 5.0);
        let double = TestDoubleEntry {
            lines: vec![e1, e2, e3],
        };
        let entry = TestEntryContainer {
            groups: vec![double],
        };
        let err = state_less_check_for_entry(&entry);
        let se = &err.double_entry_errors[0].single_entry_errors;
        assert!(!se[0].duplicate_account_in_entry);
        assert!(!se[1].duplicate_account_in_entry);
        assert!(se[2].duplicate_account_in_entry);
    }

    #[test]
    fn test_state_less_empty_double_entry() {
        let double = TestDoubleEntry {
            lines: vec![],
        };
        let entry = TestEntryContainer {
            groups: vec![double],
        };
        let err = state_less_check_for_entry(&entry);
        assert!(!err.entry_is_empty);
        assert_eq!(err.double_entry_errors.len(), 1);
        let de = &err.double_entry_errors[0];
        assert!(de.entry_is_empty);
        assert_eq!(de.single_entry_errors.len(), 0);
    }

    #[test]
    fn test_state_less_multiple_double_entries() {
        // Two double entries: first is valid, second has mismatch
        let d1 = simple_entry("A", true, 1.0, 10.0);
        let c1 = simple_entry("B", false, 1.0, 10.0); // balanced
        let double1 = TestDoubleEntry {
            lines: vec![d1, c1],
        };

        let d2 = simple_entry("C", true, 1.0, 5.0);
        let c2 = simple_entry("D", false, 1.0, 3.0); // mismatch
        let double2 = TestDoubleEntry {
            lines: vec![d2, c2],
        };

        let entry = TestEntryContainer {
            groups: vec![double1, double2],
        };
        let err = state_less_check_for_entry(&entry);
        assert_eq!(err.double_entry_errors.len(), 2);
        // First entry: no error
        let de1 = &err.double_entry_errors[0];
        assert!(de1.debit_not_equal_credit.is_none());
        // Second entry: error
        let de2 = &err.double_entry_errors[1];
        assert!(de2.debit_not_equal_credit.is_some());
        let dnc = de2.debit_not_equal_credit.as_ref().unwrap();
        assert_eq!(dnc.total_debit, 5.0);
        assert_eq!(dnc.total_credit, 3.0);
    }

    // -------------------------------------------------------------------------
    // Additional tests for state_full_check_for_entry (inventory helpers)
    // -------------------------------------------------------------------------

    #[test]
    fn test_state_full_wac_combines_even_on_error() {
        // This test exposes the bug: Wac outflow combines inventory layers BEFORE checking sufficiency.
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
        // Add two layers
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        });
        inv.push(InventoryRecord {
            time_unix: 2,
            quantity:  3.0,
            amount:    6.0,
        });
        // Total: 8 qty, 16 amt, price=2

        // Outflow 10 qty (> total) with Wac → will error on insufficient quantity
        let single = entry("A", false, 10.0, 20.0, InFlowType::Manual, OutFlowType::Wac);
        let double = TestDoubleEntry {
            lines: vec![single],
        };
        let entry = TestEntryContainer {
            groups: vec![double],
        };
        let err = state_full_check_for_entry(100, &entry, &mut provider);

        // Error should be present
        let se = &err.double_entry_errors[0].single_entry_errors[0];
        assert!(se.insufficient_quantity_in_inventory.is_some());

        // BUT inventory should now be combined into a single record (BUG)
        let inv_after = provider.get_or_create_inventory(&AccountId("A".to_string()));
        assert_eq!(inv_after.iter().count(), 1);
        let rec = inv_after.iter().next().unwrap();
        assert_eq!(rec.quantity, 8.0);
        assert_eq!(rec.amount, 16.0);
        // This shows that the layers were lost even though the entry is invalid.
        // If this is not desired, the design must be changed (e.g., validate on a clone).
    }

    #[test]
    fn test_state_full_manual_outflow_does_combine() {
        // For Manual outflow, inventory layers should remain separate
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        });
        inv.push(InventoryRecord {
            time_unix: 2,
            quantity:  3.0,
            amount:    6.0,
        });

        let single = entry("A", false, 4.0, 8.0, InFlowType::Manual, OutFlowType::Manual);
        let double = TestDoubleEntry {
            lines: vec![single],
        };
        let entry = TestEntryContainer {
            groups: vec![double],
        };
        let _err = state_full_check_for_entry(100, &entry, &mut provider);

        // Inventory should still have two records (Manual doesn't combine)
        let inv_after = provider.get_or_create_inventory(&AccountId("A".to_string()));
        assert_eq!(inv_after.iter().count(), 1);
        // Also amounts should be updated (4 units taken from first record)
        let records: Vec<_> = inv_after.iter().collect();
        assert_eq!(records[0].quantity, 4.0);
        assert_eq!(records[0].amount, 8.0);
    }

    #[test]
    fn test_state_full_fifo_get_amount_correct() {
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        });
        inv.push(InventoryRecord {
            time_unix: 2,
            quantity:  3.0,
            amount:    9.0,
        });

        // Sorting FIFO (oldest first)
        sort_inventory(&OutFlowType::Fifo, inv);
        // get_amount should return correct FIFO cost: for 4 units, 4*2 = 8
        let amt = get_amount(4.0, inv);
        assert_eq!(amt, 8.0);
        // For 6 units: 5*2 + 1*3 = 13
        let amt2 = get_amount(6.0, inv);
        assert_eq!(amt2, 13.0);
    }

    #[test]
    fn test_state_full_lifo_get_amount() {
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        });
        inv.push(InventoryRecord {
            time_unix: 2,
            quantity:  3.0,
            amount:    9.0,
        });

        sort_inventory(&OutFlowType::Lifo, inv);
        // LIFO: newest first (qty=3, price=3), then qty=5, price=2
        let amt = get_amount(4.0, inv);
        // 3 units from second record (3*3=9) + 1 unit from first (1*2=2) => 11
        assert_eq!(amt, 11.0);
    }

    #[test]
    fn test_state_full_hifo_get_amount() {
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        }); // price=2
        inv.push(InventoryRecord {
            time_unix: 2,
            quantity:  3.0,
            amount:    9.0,
        }); // price=3
        inv.push(InventoryRecord {
            time_unix: 3,
            quantity:  2.0,
            amount:    8.0,
        }); // price=4

        sort_inventory(&OutFlowType::Hifo, inv);
        // Highest price first: 4 (qty=2), then 3 (qty=3), then 2 (qty=5)
        let amt = get_amount(3.0, inv);
        // 2*4 + 1*3 = 11
        assert_eq!(amt, 11.0);
    }

    #[test]
    fn test_state_full_lofo_get_amount() {
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        }); // price=2
        inv.push(InventoryRecord {
            time_unix: 2,
            quantity:  3.0,
            amount:    9.0,
        }); // price=3
        inv.push(InventoryRecord {
            time_unix: 3,
            quantity:  2.0,
            amount:    8.0,
        }); // price=4

        sort_inventory(&OutFlowType::Lofo, inv);
        // Lowest price first: 2 (qty=5), then 3 (qty=3), then 4 (qty=2)
        let amt = get_amount(6.0, inv);
        // 5*2 + 1*3 = 13
        assert_eq!(amt, 13.0);
    }

    #[test]
    fn test_state_full_wac_get_amount() {
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        });
        inv.push(InventoryRecord {
            time_unix: 2,
            quantity:  3.0,
            amount:    9.0,
        });
        // total = 8, amt = 19, avg = 2.375

        sort_inventory(&OutFlowType::Wac, inv); // combines into one record
        let amt = get_amount(4.0, inv);
        let expected = 4.0 * (19.0 / 8.0); // 9.5
        assert_eq!(amt, expected);
    }

    #[test]
    fn test_state_full_get_amount_zero_quantity() {
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        });
        let amt = get_amount(0.0, inv);
        assert_eq!(amt, 0.0);
    }

    // -------------------------------------------------------------------------
    // Tests for apply_entry_on_inventory (direct unit tests)
    // -------------------------------------------------------------------------

    #[test]
    fn test_apply_entry_normal_inflow() {
        let mut inv = TestInventory::default();
        apply_entry_on_inventory::<TestSingleEntry, _>(100, 20.0, 5.0, true, true, &mut inv);
        assert_eq!(inv.iter().count(), 1);
        let rec = inv.iter().next().unwrap();
        assert_eq!(rec.time_unix, 100);
        assert_eq!(rec.quantity, 5.0);
        assert_eq!(rec.amount, 20.0);
    }

    #[test]
    fn test_apply_entry_normal_outflow() {
        let mut inv = TestInventory::default();
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  10.0,
            amount:    30.0,
        });
        apply_entry_on_inventory::<TestSingleEntry, _>(200, 30.0, 10.0, false, true, &mut inv);
        // After outflow, inventory should be empty
        assert!(inv.is_empty());
    }

    #[test]
    fn test_apply_entry_rare_amount_positive_qty_zero() {
        let mut inv = TestInventory::default();
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        });
        apply_entry_on_inventory::<TestSingleEntry, _>(300, 3.0, 0.0, true, true, &mut inv);
        // Should adjust amount: total_amt = 13, qty = 5
        let (qty, amt) = sum_inventory(&inv);
        assert_eq!(qty, 5.0);
        assert_eq!(amt, 13.0);
        assert_eq!(inv.iter().count(), 1);
        let rec = inv.iter().next().unwrap();
        assert_eq!(rec.time_unix, 300);
        assert_eq!(rec.quantity, 5.0);
        assert_eq!(rec.amount, 13.0);
    }

    #[test]
    fn test_apply_entry_rare_amount_zero_qty_positive() {
        let mut inv = TestInventory::default();
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        });
        apply_entry_on_inventory::<TestSingleEntry, _>(400, 0.0, 2.0, true, true, &mut inv);
        let (qty, amt) = sum_inventory(&inv);
        assert_eq!(qty, 7.0);
        assert_eq!(amt, 10.0);
        let rec = inv.iter().next().unwrap();
        assert_eq!(rec.time_unix, 400);
        assert_eq!(rec.quantity, 7.0);
        assert_eq!(rec.amount, 10.0);
    }

    #[test]
    fn test_apply_entry_rare_amount_zero_qty_negative() {
        let mut inv = TestInventory::default();
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        });
        apply_entry_on_inventory::<TestSingleEntry, _>(500, 0.0, 2.0, false, true, &mut inv);
        let (qty, amt) = sum_inventory(&inv);
        assert_eq!(qty, 3.0);
        assert_eq!(amt, 10.0);
        let rec = inv.iter().next().unwrap();
        assert_eq!(rec.time_unix, 500);
        assert_eq!(rec.quantity, 3.0);
        assert_eq!(rec.amount, 10.0);
    }

    #[test]
    fn test_apply_entry_rare_amount_negative_qty_zero() {
        let mut inv = TestInventory::default();
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        });
        apply_entry_on_inventory::<TestSingleEntry, _>(600, 5.0, 0.0, false, true, &mut inv);
        let (qty, amt) = sum_inventory(&inv);
        assert_eq!(qty, 5.0);
        assert_eq!(amt, 5.0);
        let rec = inv.iter().next().unwrap();
        assert_eq!(rec.time_unix, 600);
        assert_eq!(rec.quantity, 5.0);
        assert_eq!(rec.amount, 5.0);
    }

    #[test]
    #[should_panic(expected = "internal error: entered unreachable code")]
    fn test_apply_entry_impossible_panics() {
        let mut inv = TestInventory::default();
        // amount > 0, quantity < 0 is impossible
        apply_entry_on_inventory::<TestSingleEntry, _>(700, 10.0, -5.0, true, true, &mut inv);
    }

    // -------------------------------------------------------------------------
    // Tests for SingleEntryError::is_there_error
    // -------------------------------------------------------------------------

    #[test]
    fn test_single_entry_error_is_there_error() {
        let err = SingleEntryError::default();
        assert!(!err.is_there_error());

        let mut err2 = SingleEntryError::default();
        err2.quantity_and_amount_are_zero = true;
        assert!(err2.is_there_error());

        let mut err3 = SingleEntryError::default();
        err3.insufficient_quantity_in_inventory = Some(super::InsufficientQuantityInInventory {
            total_quantity: 5.0,
        });
        assert!(err3.is_there_error());
    }

    // -------------------------------------------------------------------------
    // More tests for state_full_check_for_entry
    // -------------------------------------------------------------------------

    #[test]
    fn test_state_full_outflow_amount_mismatch_lifo() {
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
        // Two layers: first (older) qty=5, amt=10 (price=2); second (newer) qty=3, amt=9 (price=3)
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        });
        inv.push(InventoryRecord {
            time_unix: 2,
            quantity:  3.0,
            amount:    9.0,
        });

        // LIFO outflow: 4 units → take 3 from newest (3*3=9) + 1 from oldest (1*2=2) = 11
        // But we set amount=15 → mismatch
        let single = entry("A", false, 4.0, 15.0, InFlowType::Manual, OutFlowType::Lifo);
        let double = TestDoubleEntry {
            lines: vec![single],
        };
        let entry = TestEntryContainer {
            groups: vec![double],
        };
        let err = state_full_check_for_entry(100, &entry, &mut provider);
        let se = &err.double_entry_errors[0].single_entry_errors[0];
        assert!(se.amount_mismatch.is_some());
        let am = se.amount_mismatch.as_ref().unwrap();
        assert_eq!(am.expected_amount, 11.0);
    }

    #[test]
    fn test_state_full_outflow_amount_mismatch_hifo() {
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
        // Layers: price 2 (qty=5), price 4 (qty=2), price 3 (qty=3)
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        });
        inv.push(InventoryRecord {
            time_unix: 2,
            quantity:  2.0,
            amount:    8.0,
        });
        inv.push(InventoryRecord {
            time_unix: 3,
            quantity:  3.0,
            amount:    9.0,
        });

        // Hifo: highest price first (4, then 3, then 2). For 3 units: 2*4 + 1*3 = 11
        let single = entry("A", false, 3.0, 12.0, InFlowType::Manual, OutFlowType::Hifo);
        let double = TestDoubleEntry {
            lines: vec![single],
        };
        let entry = TestEntryContainer {
            groups: vec![double],
        };
        let err = state_full_check_for_entry(100, &entry, &mut provider);
        let se = &err.double_entry_errors[0].single_entry_errors[0];
        assert!(se.amount_mismatch.is_some());
        let am = se.amount_mismatch.as_ref().unwrap();
        assert_eq!(am.expected_amount, 11.0);
    }

    #[test]
    fn test_state_full_outflow_amount_mismatch_lofo() {
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
        // Layers: price 2 (qty=5), price 4 (qty=2), price 3 (qty=3)
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        });
        inv.push(InventoryRecord {
            time_unix: 2,
            quantity:  2.0,
            amount:    8.0,
        });
        inv.push(InventoryRecord {
            time_unix: 3,
            quantity:  3.0,
            amount:    9.0,
        });

        // Lofo: lowest price first (2, then 3, then 4). For 6 units: 5*2 + 1*3 = 13
        let single = entry("A", false, 6.0, 14.0, InFlowType::Manual, OutFlowType::Lofo);
        let double = TestDoubleEntry {
            lines: vec![single],
        };
        let entry = TestEntryContainer {
            groups: vec![double],
        };
        let err = state_full_check_for_entry(100, &entry, &mut provider);
        let se = &err.double_entry_errors[0].single_entry_errors[0];
        assert!(se.amount_mismatch.is_some());
        let am = se.amount_mismatch.as_ref().unwrap();
        assert_eq!(am.expected_amount, 13.0);
    }

    #[test]
    fn test_state_full_credit_nature_account_inflow() {
        // Account G has Credit nature (false in setup_provider).
        // For Credit nature, an inflow means Credit (is_debit_state = false).
        // The test entry: amount=10, quantity=2, debit=false, which should be inflow.
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("G".to_string()));
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        });

        let single = entry("G", false, 2.0, 10.0, InFlowType::Manual, OutFlowType::Manual);
        let double = TestDoubleEntry {
            lines: vec![single],
        };
        let entry = TestEntryContainer {
            groups: vec![double],
        };
        let err = state_full_check_for_entry(100, &entry, &mut provider);
        // No errors expected
        assert!(err.double_entry_errors[0].single_entry_errors[0].is_there_error() == false);
        // Inventory should have been increased by 2 qty and 10 amt
        let (qty, amt) = sum_inventory(&provider.inventories[&AccountId("G".to_string())]);
        assert_eq!(qty, 7.0);
        assert_eq!(amt, 20.0);
    }

    #[test]
    fn test_state_full_credit_nature_account_outflow() {
        // For Credit nature, outflow means Debit (is_debit_state = true).
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("G".to_string()));
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        });

        let single = entry("G", true, 2.0, 4.0, InFlowType::Manual, OutFlowType::Manual);
        let double = TestDoubleEntry {
            lines: vec![single],
        };
        let entry = TestEntryContainer {
            groups: vec![double],
        };
        let err = state_full_check_for_entry(100, &entry, &mut provider);
        // No errors expected
        assert!(err.double_entry_errors[0].single_entry_errors[0].is_there_error() == false);
        // Inventory should have decreased (outflow)
        let (qty, amt) = sum_inventory(&provider.inventories[&AccountId("G".to_string())]);
        assert_eq!(qty, 3.0);
        assert_eq!(amt, 6.0);
    }

    #[test]
    fn test_state_full_partial_application_when_one_line_errors() {
        // Two lines in a double entry: first is error (insufficient amount), second is valid.
        // Since apply is per-line if no error, the second line will still apply.
        let mut provider = setup_provider();
        // Account A: inventory 5 qty, 10 amt
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        });

        // Line 1: outflow of 20 amt (error)
        let e1 = entry("A", false, 5.0, 20.0, InFlowType::Manual, OutFlowType::Manual);
        // Line 2: inflow of 3 qty, 15 amt (valid)
        let e2 = entry("A", true, 3.0, 15.0, InFlowType::Manual, OutFlowType::Manual);
        let double = TestDoubleEntry {
            lines: vec![e1, e2],
        };
        let entry = TestEntryContainer {
            groups: vec![double],
        };
        let err = state_full_check_for_entry(100, &entry, &mut provider);

        // Error should be present on line 1
        assert!(
            err.double_entry_errors[0].single_entry_errors[0]
                .insufficient_amount_in_inventory
                .is_some()
        );
        // Line 2 should have no error
        assert!(!err.double_entry_errors[0].single_entry_errors[1].is_there_error());

        // However, line 2 will have been applied (since no error), so inventory increased by 3 qty and 15 amt
        // But line 1 error should not have applied. But line 2 applied, so total inventory: 5+3=8 qty, 10+15=25 amt.
        // Additionally, sort_inventory for Manual combined layers, so we have one record.
        let (qty, amt) = sum_inventory(&provider.inventories[&AccountId("A".to_string())]);
        assert_eq!(qty, 8.0);
        assert_eq!(amt, 25.0);
        // This is partial application – you may want this or not. It's documented by this test.
    }

    // -------------------------------------------------------------------------
    // Tests for DoubleEntryError::is_there_error
    // -------------------------------------------------------------------------

    #[test]
    fn test_double_entry_error_is_there_error() {
        let mut de = DoubleEntryError::default();
        assert!(!de.is_there_error());

        de.entry_is_empty = true;
        assert!(de.is_there_error());

        de.entry_is_empty = false;
        de.you_need_to_split_the_entry = true;
        assert!(de.is_there_error());

        de.you_need_to_split_the_entry = false;
        de.debit_not_equal_credit = Some(DebitNotEqualCreditError {
            total_debit:  1.0,
            total_credit: 2.0,
        });
        assert!(de.is_there_error());

        de.debit_not_equal_credit = None;
        let se = SingleEntryError {
            quantity_and_amount_are_zero: true,
            ..Default::default()
        };
        de.single_entry_errors.push(se);
        assert!(de.is_there_error());
    }

    // -------------------------------------------------------------------------
    // Direct tests for inventory helper functions
    // -------------------------------------------------------------------------

    #[test]
    fn test_sum_inventory_direct() {
        let mut inv = TestInventory::default();
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  2.0,
            amount:    5.0,
        });
        inv.push(InventoryRecord {
            time_unix: 2,
            quantity:  3.0,
            amount:    7.0,
        });
        let (q, a) = sum_inventory(&inv);
        assert_eq!(q, 5.0);
        assert_eq!(a, 12.0);
    }

    #[test]
    fn test_combine_all_inventory_record_in_one_record_direct() {
        let mut inv = TestInventory::default();
        inv.push(InventoryRecord {
            time_unix: 10,
            quantity:  2.0,
            amount:    5.0,
        });
        inv.push(InventoryRecord {
            time_unix: 20,
            quantity:  3.0,
            amount:    7.0,
        });
        combine_all_inventory_record_in_one_record(&mut inv);
        assert_eq!(inv.iter().count(), 1);
        let rec = inv.iter().next().unwrap();
        assert_eq!(rec.time_unix, 20);
        assert_eq!(rec.quantity, 5.0);
        assert_eq!(rec.amount, 12.0);
    }

    // -------------------------------------------------------------------------
    // More tests for decrease_inventory
    // -------------------------------------------------------------------------

    #[test]
    fn test_decrease_inventory_exact_consumption() {
        let mut inv = TestInventory::default();
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  2.0,
            amount:    4.0,
        });
        inv.push(InventoryRecord {
            time_unix: 2,
            quantity:  3.0,
            amount:    9.0,
        });
        decrease_inventory(2.0, &mut inv); // consume exactly first record
        let records: Vec<_> = inv.iter().collect();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].quantity, 3.0);
        assert_eq!(records[0].amount, 9.0);
    }

    #[test]
    fn test_decrease_inventory_consume_all() {
        let mut inv = TestInventory::default();
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  2.0,
            amount:    4.0,
        });
        inv.push(InventoryRecord {
            time_unix: 2,
            quantity:  3.0,
            amount:    9.0,
        });
        decrease_inventory(5.0, &mut inv);
        assert!(inv.is_empty());
    }

    #[test]
    fn test_decrease_inventory_multiple_records() {
        let mut inv = TestInventory::default();
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        });
        inv.push(InventoryRecord {
            time_unix: 2,
            quantity:  3.0,
            amount:    9.0,
        });
        decrease_inventory(7.0, &mut inv);
        // Should consume all first (5) and 2 from second → remaining second qty=1, amt=3
        let records: Vec<_> = inv.iter().collect();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].quantity, 1.0);
        assert_eq!(records[0].amount, 3.0);
    }

    // -------------------------------------------------------------------------
    // More tests for apply_entry_on_inventory (rare cases)
    // -------------------------------------------------------------------------

    #[test]
    fn test_apply_entry_rare_clear_inventory() {
        let mut inv = TestInventory::default();
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        });
        // Rare case: amount == 0, quantity negative that exactly cancels total
        apply_entry_on_inventory::<TestSingleEntry, _>(200, 0.0, 5.0, false, true, &mut inv);
        // Should clear inventory
        assert_eq!(inv.0[0].quantity, 0.0);
        assert_eq!(inv.0[0].amount, 10.0);
    }

    #[test]
    fn test_apply_entry_rare_adjustment_where_total_becomes_zero() {
        let mut inv = TestInventory::default();
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        });
        apply_entry_on_inventory::<TestSingleEntry, _>(300, 10.0, 0.0, false, true, &mut inv);
        assert_eq!(inv.0[0].quantity, 5.0);
        assert_eq!(inv.0[0].amount, 0.0);
    }

    // -------------------------------------------------------------------------
    // State-full tests for rare inflows (amount>0, qty=0 and amount=0, qty>0)
    // -------------------------------------------------------------------------

    #[test]
    fn test_state_full_rare_inflow_amount_positive_qty_zero() {
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        });

        let single = entry("A", true, 0.0, 3.0, InFlowType::Manual, OutFlowType::Manual);
        let double = TestDoubleEntry {
            lines: vec![single],
        };
        let entry = TestEntryContainer {
            groups: vec![double],
        };
        let err = state_full_check_for_entry(100, &entry, &mut provider);
        // Should have no error (it's rare but allowed)
        assert!(!err.double_entry_errors[0].single_entry_errors[0].is_there_error());
        // Inventory should have amount increased, quantity unchanged
        let (qty, amt) = sum_inventory(&provider.inventories[&AccountId("A".to_string())]);
        assert_eq!(qty, 5.0);
        assert_eq!(amt, 13.0);
    }

    #[test]
    fn test_state_full_rare_inflow_amount_zero_qty_positive() {
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        });

        let single = entry("A", true, 2.0, 0.0, InFlowType::Manual, OutFlowType::Manual);
        let double = TestDoubleEntry {
            lines: vec![single],
        };
        let entry = TestEntryContainer {
            groups: vec![double],
        };
        let err = state_full_check_for_entry(100, &entry, &mut provider);
        assert!(!err.double_entry_errors[0].single_entry_errors[0].is_there_error());
        let (qty, amt) = sum_inventory(&provider.inventories[&AccountId("A".to_string())]);
        assert_eq!(qty, 7.0);
        assert_eq!(amt, 10.0);
    }

    // -------------------------------------------------------------------------
    // State-full tests for rare outflows (amount<0, qty=0 and amount=0, qty<0)
    // -------------------------------------------------------------------------

    #[test]
    fn test_state_full_rare_outflow_amount_negative_qty_zero() {
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        });

        // Outflow with amount=5 (so amt = -5), qty=0
        let single = entry("A", false, 0.0, 5.0, InFlowType::Manual, OutFlowType::Manual);
        let double = TestDoubleEntry {
            lines: vec![single],
        };
        let entry = TestEntryContainer {
            groups: vec![double],
        };
        let err = state_full_check_for_entry(100, &entry, &mut provider);
        assert!(!err.double_entry_errors[0].single_entry_errors[0].is_there_error());
        let (qty, amt) = sum_inventory(&provider.inventories[&AccountId("A".to_string())]);
        assert_eq!(qty, 5.0);
        assert_eq!(amt, 5.0); // 10 - 5
    }

    #[test]
    fn test_state_full_rare_outflow_amount_zero_qty_negative() {
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        });

        let single = entry("A", false, 2.0, 0.0, InFlowType::Manual, OutFlowType::Manual);
        let double = TestDoubleEntry {
            lines: vec![single],
        };
        let entry = TestEntryContainer {
            groups: vec![double],
        };
        let err = state_full_check_for_entry(100, &entry, &mut provider);
        assert!(!err.double_entry_errors[0].single_entry_errors[0].is_there_error());
        let (qty, amt) = sum_inventory(&provider.inventories[&AccountId("A".to_string())]);
        assert_eq!(qty, 3.0);
        assert_eq!(amt, 10.0);
    }

    // -------------------------------------------------------------------------
    // Test sorting of Hifo and Lofo directly (not just get_amount)
    // -------------------------------------------------------------------------

    #[test]
    fn test_sort_inventory_hifo_direct() {
        let mut inv = TestInventory::default();
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        }); // price 2
        inv.push(InventoryRecord {
            time_unix: 2,
            quantity:  3.0,
            amount:    9.0,
        }); // price 3
        inv.push(InventoryRecord {
            time_unix: 3,
            quantity:  2.0,
            amount:    8.0,
        }); // price 4
        sort_inventory(&OutFlowType::Hifo, &mut inv);
        let prices: Vec<f64> = inv.iter().map(|r| r.amount / r.quantity).collect();
        assert_eq!(prices, vec![4.0, 3.0, 2.0]);
    }

    #[test]
    fn test_sort_inventory_lofo_direct() {
        let mut inv = TestInventory::default();
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        }); // price 2
        inv.push(InventoryRecord {
            time_unix: 2,
            quantity:  3.0,
            amount:    9.0,
        }); // price 3
        inv.push(InventoryRecord {
            time_unix: 3,
            quantity:  2.0,
            amount:    8.0,
        }); // price 4
        sort_inventory(&OutFlowType::Lofo, &mut inv);
        let prices: Vec<f64> = inv.iter().map(|r| r.amount / r.quantity).collect();
        assert_eq!(prices, vec![2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_state_full_manual_outflow_ignores_amount_mismatch() {
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        });

        let single = entry("A", false, 2.0, 10.0, InFlowType::Manual, OutFlowType::Manual);
        let double = TestDoubleEntry {
            lines: vec![single],
        };
        let entry = TestEntryContainer {
            groups: vec![double],
        };
        let err = state_full_check_for_entry(100, &entry, &mut provider);

        dbg!(&err);
        let se = &err.double_entry_errors[0].single_entry_errors[0];
        assert!(se.amount_mismatch.is_none());
        let (qty, amt) = sum_inventory(&provider.inventories[&AccountId("A".to_string())]);
        assert_eq!(qty, 3.0);
        assert_eq!(amt, 0.0);
    }

    #[test]
    fn test_apply_entry_manual_outflow_uses_given_amount() {
        let mut inv = TestInventory::default();
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        });
        // Manual outflow: is_decrease_by_price = false
        apply_entry_on_inventory::<TestSingleEntry, _>(100, 3.0, 2.0, false, false, &mut inv);
        let (qty, amt) = sum_inventory(&inv);
        assert_eq!(qty, 3.0); // 5 - 2
        assert_eq!(amt, 7.0); // 10 - 3 (given amount)
    }

    #[test]
    fn test_decrease_inventory_zero_quantity() {
        let mut inv = TestInventory::default();
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        });
        decrease_inventory(0.0, &mut inv);
        let (qty, amt) = sum_inventory(&inv);
        assert_eq!(qty, 5.0);
        assert_eq!(amt, 10.0);
    }

    #[test]
    fn test_get_amount_empty_inventory() {
        let inv = TestInventory::default();
        let amt = get_amount(5.0, &inv);
        assert_eq!(amt, 0.0);
    }
}
