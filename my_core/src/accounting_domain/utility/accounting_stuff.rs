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

use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;

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

#[derive(PartialEq, Debug)]
enum Nature {
    Debit,
    Credit,
}

#[derive(PartialEq, Debug, Default)]
struct AccountingEntry<AccountID> {
    double_entry: Vec<SingleEntry<AccountID>>,
}

#[derive(PartialEq, Debug)]
struct SingleEntry<AccountID> {
    cost_flow_type: CostFlowType,
    account_id:     AccountID,
    quantity:       f64,
    amount:         f64,
}

#[derive(Debug, PartialEq)]
pub struct InventoryRecord {
    time_unix: u64,
    quantity:  f64,
    amount:    f64,
}

struct AccountInfo {
    nature:            Nature,
    inventory_records: Vec<InventoryRecord>,
}

type AccountInfoM<AccountID> = HashMap<AccountID, AccountInfo>;

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

pub(crate) trait DoubleEntryUtile: IntoIterator<Item = Self::SingleEntry> {
    type SingleEntry: SingleEntryUtile;

    fn is_empty(&self) -> bool;
    fn len(&self) -> usize;
}

pub(crate) trait SingleEntryUtile {
    type AccountId: Eq + Hash;

    fn get_account(&self) -> Self::AccountId;
    fn get_quantity(&self) -> f64;
    fn get_amount(&self) -> f64;
}

pub(crate) fn state_less_check_for_entry<De: DoubleEntryUtile>(entry: De) -> Error {
    let mut errr = Error::default();
    if entry.is_empty() {
        errr.entry_is_empty = true;
        return errr;
    }

    errr.single_entry_errors = Vec::with_capacity(entry.len());

    let mut accounts: HashSet<<De::SingleEntry as SingleEntryUtile>::AccountId> =
        HashSet::with_capacity(entry.len());

    for (i, single) in entry.into_iter().enumerate() {
        if single.get_amount() == 0.0 && single.get_quantity() == 0.0 {
            errr.single_entry_errors[i].quantity_and_amount_are_zero = true;
        }

        if single.get_amount() < 0.0 {
            errr.single_entry_errors[i].the_amount_should_be_positive = true;
        }

        if single.get_quantity() < 0.0 {
            errr.single_entry_errors[i].the_quantity_should_be_positive = true;
        }

        if !accounts.insert(single.get_account()) {
            errr.single_entry_errors[i].duplicate_account_in_entry = true;
        }
    }

    errr
}

fn is_debit(is_debit_nature: bool, is_inflow: bool) -> bool {
    match (is_debit_nature, is_inflow) {
        (true, true) => true,
        (true, false) => false,
        (false, true) => false,
        (false, false) => true,
    }
}

fn state_full_check_for_entry<AccountID: Eq + Hash>(
    entry: &AccountingEntry<AccountID>,
    account_info: &mut AccountInfoM<AccountID>,
) -> Error {
    let mut errr = Error::default();

    errr.single_entry_errors = Vec::with_capacity(entry.double_entry.len());

    let mut total_debit = 0.0;
    let mut total_credit = 0.0;

    for (i, single) in entry.double_entry.iter().enumerate() {
        match account_info.get_mut(&single.account_id) {
            None => {
                errr.single_entry_errors[i].account_info_not_found = true;
            }
            Some(AccountInfo {
                nature,
                inventory_records,
            }) => {
                if is_debit(
                    matches!(nature, Nature::Debit),
                    matches!(single.cost_flow_type, CostFlowType::InFlow(_)),
                ) {
                    total_debit += single.amount;
                } else {
                    total_credit += single.amount;
                }

                if inventory_records.is_empty() {
                    errr.single_entry_errors[i].inventory_is_empty = true;
                }

                match &single.cost_flow_type {
                    CostFlowType::InFlow(in_flow_type) => {
                        match in_flow_type {
                            InFlowType::None => {}
                            InFlowType::Wac => {
                                let (total_quantity, total_amount) =
                                    sum_inventory(inventory_records);

                                if total_quantity != 0.0 {
                                    let expected_amount =
                                        single.quantity * price(total_amount, total_quantity);

                                    if single.amount != expected_amount {
                                        errr.single_entry_errors[i].amount_mismatch =
                                            Some(AmountMismatch {
                                                expected_amount,
                                            });
                                    }
                                }
                            }
                            InFlowType::QuantityEqualAmount => {
                                if single.quantity != single.amount {
                                    errr.single_entry_errors[i].quantity_not_equal_amount = true;
                                }
                            }
                            InFlowType::QuantityEqualZero => {
                                if single.quantity != 0.0 {
                                    errr.single_entry_errors[i].quantity_not_equal_zero = true;
                                }
                            }
                        }
                    }
                    CostFlowType::OutFlow(out_flow_type) => {
                        let (total_quantity, total_amount) = sum_inventory(inventory_records);

                        if total_amount + single.amount < 0.0 {
                            errr.single_entry_errors[i].insufficient_amount_in_inventory =
                                Some(InsufficientAmountInInventory {
                                    total_amount,
                                });
                        }

                        if total_quantity + single.quantity < 0.0 {
                            errr.single_entry_errors[i].insufficient_quantity_in_inventory =
                                Some(InsufficientQuantityInInventory {
                                    total_quantity,
                                });
                        }

                        sort_inventory(out_flow_type, inventory_records);

                        match out_flow_type {
                            OutFlowType::Wac
                            | OutFlowType::Fifo
                            | OutFlowType::Lifo
                            | OutFlowType::Hifo
                            | OutFlowType::Lofo
                            | OutFlowType::None => {
                                let expected_amount =
                                    get_amount(single.quantity, &inventory_records);

                                if expected_amount != single.amount {
                                    errr.single_entry_errors[i].amount_mismatch =
                                        Some(AmountMismatch {
                                            expected_amount,
                                        });
                                }
                            }
                            OutFlowType::QuantityEqualAmount => {
                                if single.quantity != single.amount {
                                    errr.single_entry_errors[i].quantity_not_equal_amount = true;
                                }
                            }
                            OutFlowType::QuantityEqualZero => {
                                if single.quantity != 0.0 {
                                    errr.single_entry_errors[i].quantity_not_equal_zero = true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if total_debit != total_credit {
        errr.debit_not_equal_credit = Some(DebitNotEqualCreditError {
            total_debit,
            total_credit,
        });
    }

    errr
}

fn apply_entry_on_inventory<AccountID: Eq + Hash>(
    time_unix: u64,
    entry: &AccountingEntry<AccountID>,
    account_info: &mut AccountInfoM<AccountID>,
) {
    for single in &entry.double_entry {
        let inventory = match account_info.get_mut(&single.account_id) {
            Some(inventory) => &mut inventory.inventory_records,
            None => &mut Vec::with_capacity(1),
        };

        let (amt, qty) = match single.cost_flow_type {
            CostFlowType::InFlow(_) => (single.amount, single.quantity),
            CostFlowType::OutFlow(_) => (-single.amount, -single.quantity),
        };

        if amt > 0.0 && qty > 0.0 {
            inventory.push(InventoryRecord {
                time_unix,
                quantity: single.quantity,
                amount: single.amount,
            });
        } else if (amt == 0.0) != (qty == 0.0) {
            let (total_quantity, total_amount) = sum_inventory(&inventory);

            if total_quantity + qty == 0.0 && total_amount + amt == 0.0 {
                inventory.clear();
            } else {
                inventory.clear();
                inventory.push(InventoryRecord {
                    time_unix,
                    quantity: total_quantity + qty,
                    amount: total_amount + amt,
                });
            }
        } else if amt < 0.0 && qty < 0.0 {
            decrease_inventory(single.quantity, inventory);
        } else {
            unreachable!();
        }
    }
}

fn sum_inventory(inventory: &Vec<InventoryRecord>) -> (f64, f64) {
    let mut total_quantity = 0.0;
    let mut total_amount = 0.0;

    for record in inventory {
        total_quantity += record.quantity;
        total_amount += record.amount;
    }

    (total_quantity, total_amount)
}

fn combine_all_inventory_record_in_one_record(inventory: &mut Vec<InventoryRecord>) {
    let mut total = InventoryRecord {
        time_unix: 0,
        quantity:  0.0,
        amount:    0.0,
    };

    for record in inventory.iter() {
        total.quantity += record.quantity;
        total.amount += record.amount;

        if record.time_unix > total.time_unix {
            total.time_unix = record.time_unix;
        }
    }

    inventory.clear();
    inventory.push(total);
    inventory.shrink_to_fit();
}

fn sort_inventory(cost_flow_type: &OutFlowType, inventory: &mut Vec<InventoryRecord>) {
    match cost_flow_type {
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
                price(b.amount, b.quantity).total_cmp(&price(a.amount, a.quantity))
            });
        }
        OutFlowType::Lofo => {
            inventory.sort_by(|a, b| {
                price(a.amount, a.quantity).total_cmp(&price(b.amount, b.quantity))
            });
        }
    }
}

fn price(amount: f64, quantity: f64) -> f64 {
    amount / quantity
}

fn get_amount(quantity: f64, inventory: &Vec<InventoryRecord>) -> f64 {
    let mut amount_accumulator = 0.0;
    let mut remaining_quantity = quantity;

    // Process FIFO
    for record in inventory {
        if record.quantity <= remaining_quantity {
            // Take entire record
            remaining_quantity -= record.quantity;
            amount_accumulator += record.amount;
        } else {
            // Take partial record
            amount_accumulator += remaining_quantity * price(record.amount, record.quantity);
            break;
        }
    }

    amount_accumulator
}

fn decrease_inventory(quantity: f64, inventory: &mut Vec<InventoryRecord>) {
    let mut remaining_quantity = quantity;
    let mut indexes = 0;

    // Process FIFO
    for record in inventory.iter_mut() {
        if record.quantity <= remaining_quantity {
            // Take entire record
            remaining_quantity -= record.quantity;
            indexes += 1;
        } else {
            // Take partial record
            record.quantity -= remaining_quantity;
            record.amount = record.quantity * price(record.amount, record.quantity);
            break;
        }
    }

    for _ in 0..indexes {
        inventory.pop();
    }
}
