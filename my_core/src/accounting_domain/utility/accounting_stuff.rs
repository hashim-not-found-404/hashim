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
    None, // reorderable
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
            OutFlowType::None => "None",
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
            "None" => Ok(OutFlowType::None),
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
    None,
    QuantityEqualAmount,
    QuantityEqualZero,
    Wac,
}

impl InFlowType {
    pub fn as_str(&self) -> &'static str {
        match self {
            InFlowType::None => "None",
            InFlowType::QuantityEqualAmount => "QuantityEqualAmount",
            InFlowType::QuantityEqualZero => "QuantityEqualZero",
            InFlowType::Wac => "Wac",
        }
    }
}

impl std::str::FromStr for InFlowType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "None" => Ok(InFlowType::None),
            "QuantityEqualAmount" => Ok(InFlowType::QuantityEqualAmount),
            "QuantityEqualZero" => Ok(InFlowType::QuantityEqualZero),
            "Wac" => Ok(InFlowType::Wac),
            _ => Err("unknown InFlowType".into()),
        }
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Nature {
    Debit,
    Credit,
}

#[derive(Debug, PartialEq, Clone)]
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
struct SingleEntryError {
    quantity_and_amount_are_zero:       bool,
    duplicate_account_in_entry:         bool,
    account_info_not_found:             bool,
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
pub(crate) struct Error {
    entry_is_empty:         bool,
    debit_not_equal_credit: Option<DebitNotEqualCreditError>,
    single_entry_errors:    Vec<SingleEntryError>,
}

impl types::MyErrorTrait for Error {
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
    fn quantity(&self) -> f64;
    fn amount(&self) -> f64;

    /// Resolve the cost flow type. For statuses that only provide a direction
    /// (like M1 or M2), this method should ask the provider for the default
    /// flow types of the account.
    fn resolve_flow_type<A: AccountInfoProvider<AccountId = Self::AccountId>>(
        &self,
        provider: &A,
    ) -> CostFlowType;
}

/// A container of single entries (e.g., a double‑entry group or a whole journal entry).
pub trait EntryContainer {
    type Single: SingleEntry;
    type Iter<'a>: Iterator<Item = &'a Self::Single> + ExactSizeIterator
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

    fn get_nature(&self, id: &Self::AccountId) -> Option<Nature>;
    fn get_inventory_mut(&mut self, id: &Self::AccountId) -> Option<&mut Self::Inventory>;
    fn get_or_create_inventory(&mut self, id: &Self::AccountId) -> &mut Self::Inventory;

    /// Get the default InFlowType and OutFlowType for a given account.
    /// This is used when the entry status only provides a direction flag (M1, M2).
    fn get_default_flow_types(&self, id: &Self::AccountId) -> Option<(InFlowType, OutFlowType)>;
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

fn is_debit(nature: Nature, is_inflow: bool) -> bool {
    match (nature, is_inflow) {
        (Nature::Debit, true) => true,
        (Nature::Debit, false) => false,
        (Nature::Credit, true) => false,
        (Nature::Credit, false) => true,
    }
}

fn price(amount: f64, quantity: f64) -> f64 {
    amount / quantity
}

// -----------------------------------------------------------------------------
// Generic accounting functions
// -----------------------------------------------------------------------------

/// State‑less check: only checks the entries themselves, no inventory/account info needed.
pub(crate) fn state_less_check_for_entry<C>(entry: &C) -> Error
where
    C: EntryContainer,
    C::Single: SingleEntry,
{
    let mut errr = Error::default();
    if entry.is_empty() {
        errr.entry_is_empty = true;
        return errr;
    }

    // Pre‑allocate to the exact length
    errr.single_entry_errors = vec![SingleEntryError::default(); entry.len()];

    let mut seen_accounts = HashSet::with_capacity(entry.len());

    for (i, single) in entry.iter().enumerate() {
        let mut single_err = SingleEntryError::default();

        if single.amount() == 0.0 && single.quantity() == 0.0 {
            single_err.quantity_and_amount_are_zero = true;
        }
        if single.amount() < 0.0 {
            single_err.the_amount_should_be_positive = true;
        }
        if single.quantity() < 0.0 {
            single_err.the_quantity_should_be_positive = true;
        }

        if !seen_accounts.insert(single.account_id().clone()) {
            single_err.duplicate_account_in_entry = true;
        }

        errr.single_entry_errors[i] = single_err;
    }

    errr
}

/// Full validation including account info and inventory.
pub(crate) fn state_full_check_for_entry<C, A>(entry: &C, account_info: &mut A) -> Error
where
    C: EntryContainer,
    C::Single: SingleEntry,
    A: AccountInfoProvider<AccountId = <C::Single as SingleEntry>::AccountId>,
    A::Inventory: Inventory,
{
    let mut errr = Error::default();

    // Pre‑allocate to the exact length
    errr.single_entry_errors = vec![SingleEntryError::default(); entry.len()];

    let mut total_debit = 0.0;
    let mut total_credit = 0.0;

    for (i, single) in entry.iter().enumerate() {
        let mut single_err = SingleEntryError::default();

        let account_id = single.account_id();
        let nature = account_info.get_nature(account_id);
        let flow_type = single.resolve_flow_type(account_info);
        let inventory_opt = account_info.get_inventory_mut(account_id);

        match (nature, inventory_opt) {
            (Some(nature), Some(inventory)) => {
                // Check if inventory is empty
                if inventory.is_empty() {
                    single_err.inventory_is_empty = true;
                }

                let is_inflow = matches!(flow_type, CostFlowType::InFlow(_));

                if is_debit(nature, is_inflow) {
                    total_debit += single.amount();
                } else {
                    total_credit += single.amount();
                }

                // Process according to flow type
                match flow_type {
                    CostFlowType::InFlow(in_flow_type) => {
                        match in_flow_type {
                            InFlowType::None => {}
                            InFlowType::Wac => {
                                let (total_qty, total_amt) = sum_inventory(inventory);
                                if total_qty != 0.0 {
                                    let expected_amt =
                                        single.quantity() * price(total_amt, total_qty);
                                    if single.amount() != expected_amt {
                                        single_err.amount_mismatch = Some(AmountMismatch {
                                            expected_amount: expected_amt,
                                        });
                                    }
                                }
                            }
                            InFlowType::QuantityEqualAmount => {
                                if single.quantity() != single.amount() {
                                    single_err.quantity_not_equal_amount = true;
                                }
                            }
                            InFlowType::QuantityEqualZero => {
                                if single.quantity() != 0.0 {
                                    single_err.quantity_not_equal_zero = true;
                                }
                            }
                        }
                    }
                    CostFlowType::OutFlow(out_flow_type) => {
                        let (total_qty, total_amt) = sum_inventory(inventory);
                        // Check sufficient amount
                        if total_amt + single.amount() < 0.0 {
                            single_err.insufficient_amount_in_inventory =
                                Some(InsufficientAmountInInventory {
                                    total_amount: total_amt,
                                });
                        }
                        if total_qty + single.quantity() < 0.0 {
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
                            | OutFlowType::None => {
                                let expected_amt = get_amount(single.quantity(), inventory);
                                if expected_amt != single.amount() {
                                    single_err.amount_mismatch = Some(AmountMismatch {
                                        expected_amount: expected_amt,
                                    });
                                }
                            }
                            OutFlowType::QuantityEqualAmount => {
                                if single.quantity() != single.amount() {
                                    single_err.quantity_not_equal_amount = true;
                                }
                            }
                            OutFlowType::QuantityEqualZero => {
                                if single.quantity() != 0.0 {
                                    single_err.quantity_not_equal_zero = true;
                                }
                            }
                        }
                    }
                }
            }
            (None, _) => {
                single_err.account_info_not_found = true;
            }
            (_, None) => {
                // This should not happen if nature exists, but we handle it gracefully
                single_err.account_info_not_found = true;
            }
        }

        errr.single_entry_errors[i] = single_err;
    }

    if total_debit != total_credit {
        errr.debit_not_equal_credit = Some(DebitNotEqualCreditError {
            total_debit,
            total_credit,
        });
    }

    errr
}

/// Apply the entry to the inventory, updating records.
pub fn apply_entry_on_inventory<C, A>(time_unix: u64, entry: &C, account_info: &mut A)
where
    C: EntryContainer,
    C::Single: SingleEntry,
    A: AccountInfoProvider<AccountId = <C::Single as SingleEntry>::AccountId>,
    A::Inventory: Inventory,
{
    for single in entry.iter() {
        let account_id = single.account_id();
        let flow_type = single.resolve_flow_type(account_info);
        let inventory = account_info.get_or_create_inventory(account_id);

        let (amt, qty) = match flow_type {
            CostFlowType::InFlow(_) => (single.amount(), single.quantity()),
            CostFlowType::OutFlow(_) => (-single.amount(), -single.quantity()),
        };

        if amt > 0.0 && qty > 0.0 {
            inventory.push(InventoryRecord {
                time_unix,
                quantity: single.quantity(),
                amount: single.amount(),
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
            decrease_inventory(single.quantity(), inventory);
        } else {
            unreachable!();
        }
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
        | OutFlowType::None
        | OutFlowType::Wac => {
            combine_all_inventory_record_in_one_record(inventory);
        }
        OutFlowType::Fifo => {
            inventory.sort_by(|a, b| b.time_unix.cmp(&a.time_unix));
        }
        OutFlowType::Lifo => {
            inventory.sort_by(|a, b| a.time_unix.cmp(&b.time_unix));
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
