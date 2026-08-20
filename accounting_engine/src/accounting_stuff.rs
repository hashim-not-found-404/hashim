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
use std::cmp::Ordering;
use std::str::FromStr;

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
pub enum CostFlowType {
    InFlow(InFlowType),
    OutFlow(OutFlowType),
}

#[derive(PartialEq, Debug, Deserialize, Serialize, Clone, Copy, Default)]
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

impl FromStr for OutFlowType {
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

#[derive(PartialEq, Debug, Deserialize, Serialize, Clone, Copy, Default)]
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

impl FromStr for InFlowType {
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
    pub time_unix: u64,
    pub quantity:  f64,
    pub amount:    f64,
}

pub trait DoubleEntry {
    type Single;

    type Iter<'a>: IntoIterator<Item = Self::Single> + ExactSizeIterator
    where
        Self: 'a;

    type IterRef<'a>: Iterator<Item = &'a Self::Single> + ExactSizeIterator
    where
        Self: 'a;

    type IterMut<'a>: Iterator<Item = &'a mut Self::Single> + ExactSizeIterator
    where
        Self: 'a;

    fn into_iter<'a>(self) -> Self::Iter<'a>;
    fn iter_ref(&self) -> Self::IterRef<'_>;
    fn iter_mut(&mut self) -> Self::IterMut<'_>;

    fn set_singles(&mut self, singles: Vec<Self::Single>);
    fn is_empty(&self) -> bool;
    fn len(&self) -> usize;
    fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&Self::Single) -> bool;
}

pub trait EntryContainer {
    type Double<'a>: DoubleEntry + 'a
    where
        Self: 'a;

    type Iter<'a>: IntoIterator<Item = Self::Double<'a>> + ExactSizeIterator
    where
        Self: 'a;

    type IterRef<'a>: Iterator<Item = &'a Self::Double<'a>> + ExactSizeIterator
    where
        Self: 'a;

    type IterMut<'a>: Iterator<Item = &'a mut Self::Double<'a>> + ExactSizeIterator
    where
        Self: 'a;

    fn iter<'a>(self) -> Self::Iter<'a>;
    fn iter_ref(&self) -> Self::IterRef<'_>;
    fn iter_mut(&mut self) -> Self::IterMut<'_>;

    fn set_doubles(&mut self, doubles: Vec<Self::Double<'_>>);
    fn is_empty(&self) -> bool;
    fn len(&self) -> usize;
    fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&Self::Double<'_>) -> bool;
}

pub trait Inventory {
    fn push(&mut self, record: InventoryRecord);
    fn clear(&mut self);
    fn is_empty(&self) -> bool;
    fn iter1(&self) -> impl Iterator<Item = &InventoryRecord>;
    fn iter_mut1(&mut self) -> impl Iterator<Item = &mut InventoryRecord>;
    fn sort_by1<F>(&mut self, compare: F)
    where
        F: FnMut(&InventoryRecord, &InventoryRecord) -> Ordering;
    fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&InventoryRecord) -> bool;
    fn pop(&mut self) -> Option<InventoryRecord>;
}

pub(crate) fn is_debit(is_debit_nature: bool, is_inflow: bool) -> bool {
    is_debit_nature == is_inflow
}

pub(crate) fn is_inflow(is_debit_nature: bool, is_debit_state: bool) -> bool {
    is_debit_nature == is_debit_state
}

fn price(amount: f64, quantity: f64) -> f64 {
    amount / quantity
}

pub(crate) fn is_decrease_by_price(out_flow_type: OutFlowType) -> bool {
    match out_flow_type {
        OutFlowType::Manual | OutFlowType::QuantityEqualAmount | OutFlowType::QuantityEqualZero => {
            false
        }
        OutFlowType::Wac
        | OutFlowType::Fifo
        | OutFlowType::Lifo
        | OutFlowType::Hifo
        | OutFlowType::Lofo => true,
    }
}

pub(crate) fn apply_entry_on_inventory<I>(
    time_unix: u64,
    amount: f64,
    quantity: f64,
    is_inflow: bool,
    is_decrease_by_price: bool,
    inventory: &mut I,
) where
    I: Inventory,
{
    let (amt, qty) = if is_inflow {
        (amount.abs(), quantity.abs())
    } else {
        (-amount.abs(), -quantity.abs())
    };

    if amt > 0.0 && qty > 0.0 {
        inventory.push(InventoryRecord {
            time_unix,
            quantity,
            amount,
        });
    } else if (amt == 0.0) != (qty == 0.0) {
        decrease_inventory_by_manual(time_unix, inventory, amt, qty);
    } else if amt < 0.0 && qty < 0.0 {
        if is_decrease_by_price {
            decrease_inventory_by_price(quantity, inventory);
        } else {
            decrease_inventory_by_manual(time_unix, inventory, amt, qty);
        }
    } else {
        unreachable!();
    }
}

fn decrease_inventory_by_manual<I: Inventory>(
    time_unix: u64,
    inventory: &mut I,
    amt: f64,
    qty: f64,
) {
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

pub(crate) fn sum_inventory<I: Inventory>(inventory: &I) -> (f64, f64) {
    let mut total_qty = 0.0;
    let mut total_amt = 0.0;
    for record in inventory.iter1() {
        total_qty += record.quantity;
        total_amt += record.amount;
    }
    (total_qty, total_amt)
}

fn combine_all_inventory_record_in_one_record<I: Inventory>(inventory: &mut I) {
    let mut total = InventoryRecord {
        time_unix: 0,
        quantity:  0.0,
        amount:    0.0,
    };

    for record in inventory.iter1() {
        total.quantity += record.quantity;
        total.amount += record.amount;
        if record.time_unix > total.time_unix {
            total.time_unix = record.time_unix;
        }
    }
    inventory.clear();
    if !(total.quantity == 0.0 && total.amount == 0.0) {
        inventory.push(total);
    }
}

pub(crate) fn sort_inventory<I: Inventory>(out_flow_type: OutFlowType, inventory: &mut I) {
    match out_flow_type {
        OutFlowType::QuantityEqualAmount
        | OutFlowType::QuantityEqualZero
        | OutFlowType::Manual
        | OutFlowType::Wac => {
            combine_all_inventory_record_in_one_record(inventory);
        }
        OutFlowType::Fifo => {
            inventory.sort_by1(|a, b| a.time_unix.cmp(&b.time_unix));
        }
        OutFlowType::Lifo => {
            inventory.sort_by1(|a, b| b.time_unix.cmp(&a.time_unix));
        }
        OutFlowType::Hifo => {
            inventory.sort_by1(|a, b| {
                price(b.amount, b.quantity)
                    .partial_cmp(&price(a.amount, a.quantity))
                    .unwrap_or(Ordering::Equal)
            });
        }
        OutFlowType::Lofo => {
            inventory.sort_by1(|a, b| {
                price(a.amount, a.quantity)
                    .partial_cmp(&price(b.amount, b.quantity))
                    .unwrap_or(Ordering::Equal)
            });
        }
    }
}

pub(crate) fn get_amount<I: Inventory>(quantity: f64, inventory: &I) -> f64 {
    if quantity <= 0.0 {
        return 0.0;
    }

    let mut remaining = quantity;
    let mut accumulator = 0.0;

    for record in inventory.iter1() {
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

pub(crate) fn get_quantity<I: Inventory>(amount: f64, inventory: &I) -> f64 {
    if amount <= 0.0 {
        return 0.0;
    }

    let mut remaining = amount;
    let mut accumulator = 0.0;

    for record in inventory.iter1() {
        if record.amount <= remaining {
            remaining -= record.amount;
            accumulator += record.quantity;
        } else {
            if record.quantity == 0.0 {
            } else {
                let price = record.amount / record.quantity;
                accumulator += remaining / price;
            }

            break;
        }
    }

    accumulator
}

fn decrease_inventory_by_price<I: Inventory>(quantity: f64, inventory: &mut I) {
    let mut remaining = quantity;
    let mut to_remove = 0;

    for record in inventory.iter_mut1() {
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
    for record in inventory.iter1() {
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

#[cfg(test)]
mod tests {
    use super::*;

    type TestInventory = Vec<InventoryRecord>;

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

        decrease_inventory_by_price(4.0, &mut inv);

        let (qty, _amt) = sum_inventory(&inv);
        assert_eq!(qty, 6.0);
        let records: Vec<_> = inv.iter1().collect();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].quantity, 1.0);
        assert_eq!(records[0].amount, 5.0);
        assert_eq!(records[1].quantity, 5.0);
        assert_eq!(records[1].amount, 25.0);
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
        assert_eq!(inv.iter1().count(), 1);
        let rec = inv.iter1().next().unwrap();
        assert_eq!(rec.time_unix, 20);
        assert_eq!(rec.quantity, 5.0);
        assert_eq!(rec.amount, 12.0);
    }

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
        decrease_inventory_by_price(2.0, &mut inv);
        let records: Vec<_> = inv.iter1().collect();
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
        decrease_inventory_by_price(5.0, &mut inv);
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
        decrease_inventory_by_price(7.0, &mut inv);
        let records: Vec<_> = inv.iter1().collect();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].quantity, 1.0);
        assert_eq!(records[0].amount, 3.0);
    }

    #[test]
    fn test_decrease_inventory_zero_quantity() {
        let mut inv = TestInventory::default();
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        });
        decrease_inventory_by_price(0.0, &mut inv);
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

    #[test]
    fn test_get_quantity_empty_inventory() {
        let inv = TestInventory::default();
        assert_eq!(get_quantity(10.0, &inv), 0.0);
        assert_eq!(get_quantity(0.0, &inv), 0.0);
    }

    #[test]
    fn test_get_quantity_single_record_exact_match() {
        let mut inv = TestInventory::default();
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  2.0,
            amount:    4.0,
        });
        assert_eq!(get_quantity(4.0, &inv), 2.0);
    }

    #[test]
    fn test_get_quantity_single_record_partial() {
        let mut inv = TestInventory::default();
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        });
        assert_eq!(get_quantity(3.0, &inv), 1.5);
        assert_eq!(get_quantity(0.0, &inv), 0.0);
        assert_eq!(get_quantity(10.0, &inv), 5.0);
        assert_eq!(get_quantity(12.0, &inv), 5.0);
    }

    #[test]
    fn test_get_quantity_multiple_records() {
        let mut inv = TestInventory::default();
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  2.0,
            amount:    4.0,
        });
        inv.push(InventoryRecord {
            time_unix: 2,
            quantity:  3.0,
            amount:    6.0,
        });
        assert_eq!(get_quantity(5.0, &inv), 2.5);
        assert_eq!(get_quantity(4.0, &inv), 2.0);
        assert_eq!(get_quantity(6.0, &inv), 3.0);
        assert_eq!(get_quantity(10.0, &inv), 5.0);
        assert_eq!(get_quantity(20.0, &inv), 5.0);
    }

    #[test]
    fn test_get_quantity_with_zero_quantity_record() {
        let mut inv = TestInventory::default();
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  0.0,
            amount:    5.0,
        });
        inv.push(InventoryRecord {
            time_unix: 2,
            quantity:  2.0,
            amount:    4.0,
        });
        assert_eq!(get_quantity(4.0, &inv), 0.0);
        assert_eq!(get_quantity(6.0, &inv), 0.5);
        assert_eq!(get_quantity(0.0, &inv), 0.0);
    }

    #[test]
    fn test_get_quantity_negative_amount() {
        let mut inv = TestInventory::default();
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        });
        assert_eq!(get_quantity(-1.0, &inv), 0.0);
        assert_eq!(get_quantity(-5.0, &inv), 0.0);
    }

    #[test]
    fn test_get_quantity_with_wac_combined() {
        let mut inv = TestInventory::default();
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
        sort_inventory(OutFlowType::Wac, &mut inv);
        assert_eq!(get_quantity(8.0, &inv), 4.0);
        assert_eq!(get_quantity(16.0, &inv), 8.0);
        assert_eq!(get_quantity(4.0, &inv), 2.0);
        assert_eq!(get_quantity(0.0, &inv), 0.0);
    }

    #[test]
    fn test_get_quantity_with_only_zero_quantity_records() {
        let mut inv = TestInventory::default();
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  0.0,
            amount:    5.0,
        });
        inv.push(InventoryRecord {
            time_unix: 2,
            quantity:  0.0,
            amount:    10.0,
        });
        assert_eq!(get_quantity(5.0, &inv), 0.0);
        assert_eq!(get_quantity(0.0, &inv), 0.0);
    }
}
