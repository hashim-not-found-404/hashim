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

use derive_more::{Add, AddAssign, Display, From, Into, Neg, Sub, SubAssign};
use std::collections::HashMap;
use thiserror::Error;

mod error;
mod state_pattern;

#[derive(Error, Debug, PartialEq)]
enum ErrorCode {
    #[error("your entry is empty")]
    YourEntryIsEmpty,

    #[error("the current time should be bigger than the last entry time")]
    TimeShouldBeBigger,

    #[error(
        "you can't enter both quantity and amount as zeros for account ID {}",
        0
    )]
    QuantityAndAmountAreZero(AccountID),

    #[error("duplicate account ID {} in entry", 0)]
    DuplicateAccountInEntry(AccountID),

    #[error("debit not equal credit and debit = {} , credit = {} and debit-credit = {}", .total_debit, .total_credit, *.total_debit - *.total_credit)]
    DebitNotEqualCredit {
        total_debit: Amount,
        total_credit: Amount,
    },

    #[error("account info not found for account ID {}", 0)]
    AccountInfoNotFoundForAccountID(AccountID),

    #[error("inventory is empty for account ID {}", 0)]
    InventoryIsEmpty(AccountID),

    #[error("You want to withdraw quantity = {} but you do not have enough quantity because your total quantity = {} for account ID {}",.quantity.0.abs(),.total_quantity,.account_id)]
    InsufficientQuantityInInventory {
        account_id: AccountID,
        quantity: Quantity,
        total_quantity: Quantity,
    },

    #[error("amount mismatch: expected to enter amount = {} but got = {} for account ID {}",.expected_amount,.actual_amount,.account_id)]
    AmountMismatch {
        account_id: AccountID,
        expected_amount: Amount,
        actual_amount: Amount,
    },

    #[error("You want to withdraw amount = {} but you do not have enough amount because your total amount = {} for account ID {}",.amount.0.abs(),.total_amount,.account_id)]
    InsufficientAmountInInventory {
        account_id: AccountID,
        amount: Amount,
        total_amount: Amount,
    },

    #[error("the quantity and amount should be both positive for account ID {}", 0)]
    TheQuantityAndAmountShouldBeBothPositive(AccountID),

    // #[error(
    //     "you should to use cost flow type JustOutFlow because your quantity or amount is zero for account ID {}",
    //     0
    // )]
    // YouShouldUseCostFlowTypeJustOutFlowIfYouHaveQuantityOrAmountZero(AccountID),
    #[error("quantity not equal amount for account ID {} the quantity is {} and the amount is {}",.account_id,.quantity,.amount)]
    QuantityNotEqualAmount {
        account_id: AccountID,
        quantity: Quantity,
        amount: Amount,
    },

    #[error("quantity not equal amount for account ID {}", 0)]
    QuantityNotEqualZero(AccountID),
}

#[derive(PartialEq, Debug)]
enum CostFlowType {
    InFlow(InFlowType),
    OutFlow(OutFlowType),
}

#[derive(PartialEq, Debug)]
enum OutFlowType {
    None, // reorderable
    QuantityEqualAmount,
    QuantityEqualZero,
    Wac,  // reorderable
    Fifo, // sortable
    Lifo, // sortable
    Hifo, // sortable
    Lofo, // sortable
}

#[derive(PartialEq, Debug)]
enum InFlowType {
    None,
    QuantityEqualAmount,
    QuantityEqualZero,
    Wac,
}

#[derive(PartialEq, Debug)]
enum Nature {
    Debit,
    Credit,
}

#[derive(Display, Debug, PartialEq, Clone, Copy, Hash, Eq)]
struct AccountID<T = u64>(T); // this line should be changed to "struct AccountID<T>(T);" just for using multiple types for the ID like uid or uuid or guid or strings

#[derive(Display, AddAssign, SubAssign, Debug, PartialOrd, PartialEq, Clone, Copy, Neg, Add)]
struct Quantity(f64);

#[derive(Display, Sub, AddAssign, Debug, PartialEq, Clone, Copy, Add, PartialOrd, Neg)]
struct Amount(f64);

#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Copy, Default)]
struct TimeUnix(i64); // the time in UnixMicro()

#[derive(PartialEq, Into, From, Debug, Default)]
struct AccountingEntryS {
    time_unix: TimeUnix,
    double_entry: Vec<SingleEntry>,
}

#[derive(PartialEq, Debug)]
struct SingleEntry {
    cost_flow_type: CostFlowType,
    account_id: AccountID,
    quantity: Quantity,
    amount: Amount,
}

#[derive(Debug, PartialEq)]
struct InventoryRecord {
    time_unix: TimeUnix,
    quantity: Quantity,
    amount: Amount,
}

#[derive(Debug)]
struct Sorted;
#[derive(Debug)]
struct UnSorted;
#[derive(Debug)]
struct Decreased;

type Inventory<State = state_pattern::MutableState> =
    state_pattern::Wrapper<Vec<InventoryRecord>, State>;

type UnSortedInventory = Inventory<UnSorted>;
type SortedInventory = Inventory<Sorted>;
type DecreasedInventory = Inventory<Decreased>;
type StorableInventory = Inventory<Storable>;

#[derive(Debug)]
struct Checked;
#[derive(Debug)]
struct Storable;

type AccountingEntry<State = state_pattern::MutableState> =
    state_pattern::Wrapper<AccountingEntryS, State>;

type CheckedAccountingEntry = AccountingEntry<Checked>;
type StorableAccountingEntry = AccountingEntry<Storable>;

type AccountInfo<T1 = state_pattern::MutableState, T2 = state_pattern::MutableState> =
    state_pattern::Wrapper<HashMap<AccountID, (Nature, Inventory<T2>)>, T1>;

type CheckedAccountInfo = AccountInfo<Checked, Sorted>;
type StorableAccountInfo = AccountInfo<Storable, Storable>;

fn check_entry(
    last_time_unix: TimeUnix,
    entry: AccountingEntry,
    mut account_info: AccountInfo,
) -> Result<
    (CheckedAccountingEntry, CheckedAccountInfo),
    error::Error<(AccountingEntry, AccountInfo), ErrorCode>,
> {
    let e = entry.get_value();

    if e.double_entry.len() == 0 {
        bail!((entry, account_info), ErrorCode::YourEntryIsEmpty);
    }

    if e.time_unix < last_time_unix {
        bail!((entry, account_info), ErrorCode::TimeShouldBeBigger);
    }

    let mut total_debit: Amount = Amount(0.0);
    let mut total_credit: Amount = Amount(0.0);
    let mut accounts: HashMap<AccountID, ()> = HashMap::with_capacity(e.double_entry.len());

    for single in &e.double_entry {
        let single_account_id = single.account_id;
        let single_amount = single.amount;
        let single_quantity = single.quantity;

        if single.amount == Amount(0.0) && single.quantity == Quantity(0.0) {
            bail!(
                (entry, account_info),
                ErrorCode::QuantityAndAmountAreZero(single_account_id)
            );
        }

        if single.amount < Amount(0.0) || single.quantity < Quantity(0.0) {
            bail!(
                (entry, account_info),
                ErrorCode::TheQuantityAndAmountShouldBeBothPositive(single_account_id)
            );
        }

        if let Some(_) = accounts.insert(single_account_id, ()) {
            bail!(
                (entry, account_info),
                ErrorCode::DuplicateAccountInEntry(single_account_id)
            );
        }

        match account_info.get_value_mut().get_mut(&single_account_id) {
            None => {
                bail!(
                    (entry, account_info),
                    ErrorCode::AccountInfoNotFoundForAccountID(single_account_id)
                );
            }
            Some((nature, inventory)) => {
                if inventory.get_value().len() == 0 {
                    bail!(
                        (entry, account_info),
                        ErrorCode::InventoryIsEmpty(single_account_id)
                    );
                }

                let qty = -single.quantity;
                let amt = -single.amount;

                // i think it is not correct and should be removed
                // if single.cost_flow_type != CostFlowType::OutFlow(OutFlowType::None) {
                //     //CostFlowType::OutFlow(OutFlowType::zero)//here is bug
                //     if (amt == Amount(0.0) && qty < Quantity(0.0))
                //         || (amt < Amount(0.0) && qty == Quantity(0.0))
                //     {
                //         bail!(
                //             (entry, account_info),
                //             ErrorCode::YouShouldUseCostFlowTypeJustOutFlowIfYouHaveQuantityOrAmountZero(single_account_id)
                //         );
                //     }
                // }

                match &single.cost_flow_type {
                    CostFlowType::OutFlow(cost_out_flow_type) => {
                        let (total_quantity, total_amount) = sum_inventory(inventory);

                        if total_amount + amt < Amount(0.0) {
                            bail!(
                                (entry, account_info),
                                ErrorCode::InsufficientAmountInInventory {
                                    account_id: single_account_id,
                                    amount: single_amount,
                                    total_amount: total_amount
                                }
                            );
                        }

                        if total_quantity + qty < Quantity(0.0) {
                            bail!(
                                (entry, account_info),
                                ErrorCode::InsufficientQuantityInInventory {
                                    account_id: single_account_id,
                                    quantity: single_quantity,
                                    total_quantity: total_quantity
                                }
                            )
                        }

                        let inventory =
                            sort_inventory(cost_out_flow_type, inventory.transmute_ref_mut());

                        match cost_out_flow_type {
                            OutFlowType::Wac
                            | OutFlowType::Fifo
                            | OutFlowType::Lifo
                            | OutFlowType::Hifo
                            | OutFlowType::Lofo
                            | OutFlowType::None => {
                                let expected_amount = get_amount(qty, &inventory);
                                if expected_amount != amt {
                                    bail!(
                                        (entry, account_info),
                                        ErrorCode::AmountMismatch {
                                            account_id: single_account_id,
                                            expected_amount: expected_amount,
                                            actual_amount: amt
                                        }
                                    );
                                }
                            }
                            OutFlowType::QuantityEqualAmount => {
                                if single.quantity.0 != single.amount.0 {
                                    bail!(
                                        (entry, account_info),
                                        ErrorCode::QuantityNotEqualAmount {
                                            account_id: single_account_id,
                                            quantity: single_quantity,
                                            amount: single_amount
                                        }
                                    );
                                }
                            }
                            OutFlowType::QuantityEqualZero => {
                                if single.quantity != Quantity(0.0) {
                                    bail!(
                                        (entry, account_info),
                                        ErrorCode::QuantityNotEqualZero(single_account_id)
                                    );
                                }
                            }
                        }
                        match nature {
                            Nature::Debit => total_credit += single.amount,
                            Nature::Credit => total_debit += single.amount,
                        }
                    }
                    CostFlowType::InFlow(cost_in_flow_type) => {
                        match cost_in_flow_type {
                            InFlowType::None => {}
                            InFlowType::Wac => {
                                let (total_quantity, total_amount) = sum_inventory(inventory);

                                if total_quantity != Quantity(0.0) {
                                    let expected_amount = Amount(
                                        single.quantity.0 * price(total_amount, total_quantity),
                                    );
                                    if single.amount != expected_amount {
                                        bail!(
                                            (entry, account_info),
                                            ErrorCode::AmountMismatch {
                                                account_id: single_account_id,
                                                expected_amount: expected_amount,
                                                actual_amount: single_amount
                                            }
                                        );
                                    }
                                }
                            }
                            InFlowType::QuantityEqualAmount => {
                                if single.quantity.0 != single.amount.0 {
                                    bail!(
                                        (entry, account_info),
                                        ErrorCode::QuantityNotEqualAmount {
                                            account_id: single_account_id,
                                            quantity: single_quantity,
                                            amount: single_amount
                                        }
                                    );
                                }
                            }
                            InFlowType::QuantityEqualZero => {
                                if single.quantity != Quantity(0.0) {
                                    bail!(
                                        (entry, account_info),
                                        ErrorCode::QuantityNotEqualZero(single_account_id)
                                    );
                                }
                            }
                        }

                        match nature {
                            Nature::Debit => total_debit += single.amount,
                            Nature::Credit => total_credit += single.amount,
                        }
                    }
                }
            }
        }
    }

    if total_debit != total_credit {
        bail!(
            (entry, account_info),
            ErrorCode::DebitNotEqualCredit {
                total_debit: total_debit,
                total_credit: total_credit
            }
        );
    }

    Ok((
        entry.transmute(),
        transmute!(CheckedAccountInfo, account_info),
    ))
}

fn apply_entry_on_inventory(
    entry: CheckedAccountingEntry,
    mut account_info: CheckedAccountInfo,
) -> (StorableAccountingEntry, StorableAccountInfo) {
    let e = entry.get_value();
    let mut account_info = transmute!(AccountInfo, account_info);

    for single in &e.double_entry {
        let mut inventory = match account_info.get_value_mut().get_mut(&single.account_id) {
            Some(inventory_hh) => &mut inventory_hh.1,
            None => transmute!(
                &mut Inventory,
                &mut Vec::<InventoryRecord>::with_capacity(1)
            ),
        };

        let (amt, qty) = match single.cost_flow_type {
            CostFlowType::InFlow(_) => (single.amount, single.quantity),
            CostFlowType::OutFlow(_) => (-single.amount, -single.quantity),
        };

        if amt > Amount(0.0) && qty > Quantity(0.0) {
            inventory.get_value_mut().push(InventoryRecord {
                time_unix: e.time_unix,
                quantity: single.quantity,
                amount: single.amount,
            });
        } else if (amt == Amount(0.0)) != (qty == Quantity(0.0)) {
            let (total_quantity, total_amount) = sum_inventory(&inventory);

            if total_quantity + qty == Quantity(0.0) && total_amount + amt == Amount(0.0) {
                inventory.get_value_mut().clear();
            } else {
                inventory.set_value(vec![InventoryRecord {
                    time_unix: e.time_unix,
                    quantity: total_quantity + qty,
                    amount: total_amount + amt,
                }]);
            }
        } else if amt < Amount(0.0) && qty < Quantity(0.0) {
            inventory = decrease_inventory(single.quantity, inventory.transmute_ref_mut())
                .transmute_ref_mut();
        } else {
            unreachable!();
        }
    }

    (
        entry.transmute(),
        transmute!(StorableAccountInfo, account_info),
    )
}

fn sum_inventory<T>(inventory: &Inventory<T>) -> (Quantity, Amount) {
    let mut total_quantity = Quantity(0.0);
    let mut total_amount = Amount(0.0);

    for record in inventory.get_value() {
        total_quantity += record.quantity;
        total_amount += record.amount;
    }

    (total_quantity, total_amount)
}

fn combine_all_inventory_record_in_one_record(
    inventory: &mut UnSortedInventory,
) -> &mut SortedInventory {
    let mut total = InventoryRecord {
        time_unix: TimeUnix(0),
        quantity: Quantity(0.0),
        amount: Amount(0.0),
    };

    for record in inventory.get_value().iter().rev() {
        total.quantity += record.quantity;
        total.amount += record.amount;

        if record.time_unix > total.time_unix {
            total.time_unix = record.time_unix;
        }
    }

    inventory
        .transmute_ref_mut()
        .set_value(vec![total])
        .transmute_ref_mut()
}

fn sort_inventory<'a>(
    cost_flow_type: &OutFlowType,
    inventory: &'a mut UnSortedInventory,
) -> &'a mut SortedInventory {
    let inventory = inventory.transmute_ref_mut();

    match cost_flow_type {
        OutFlowType::QuantityEqualAmount
        | OutFlowType::QuantityEqualZero
        | OutFlowType::None
        | OutFlowType::Wac => {
            return combine_all_inventory_record_in_one_record(inventory.transmute_ref_mut());
        }
        OutFlowType::Fifo => {
            inventory
                .get_value_mut()
                .sort_by(|a, b| b.time_unix.cmp(&a.time_unix));
        }
        OutFlowType::Lifo => {
            inventory
                .get_value_mut()
                .sort_by(|a, b| a.time_unix.cmp(&b.time_unix));
        }
        OutFlowType::Hifo => {
            inventory.get_value_mut().sort_by(|a, b| {
                price(b.amount, b.quantity).total_cmp(&price(a.amount, a.quantity))
            });
        }
        OutFlowType::Lofo => {
            inventory.get_value_mut().sort_by(|a, b| {
                price(a.amount, a.quantity).total_cmp(&price(b.amount, b.quantity))
            });
        }
    }

    inventory.transmute_ref_mut()
}

fn price(amount: Amount, quantity: Quantity) -> f64 {
    amount.0 / quantity.0
}

fn get_amount(quantity: Quantity, inventory: &SortedInventory) -> Amount {
    let mut amount_accumulator = Amount(0.0);
    let mut remaining_quantity = quantity;

    // Process FIFO
    for record in inventory.get_value().iter().rev() {
        if record.quantity <= remaining_quantity {
            // Take entire record
            remaining_quantity -= record.quantity;
            amount_accumulator += record.amount;
        } else {
            // Take partial record
            amount_accumulator.0 += remaining_quantity.0 * price(record.amount, record.quantity);
            break;
        }
    }

    amount_accumulator
}

fn decrease_inventory(
    quantity: Quantity,
    inventory: &mut SortedInventory,
) -> &mut DecreasedInventory {
    let mut remaining_quantity = quantity;
    let mut indexes = 0;
    let inventory = inventory.transmute_ref_mut();

    // Process FIFO
    for record in inventory.get_value_mut().iter_mut().rev() {
        if record.quantity <= remaining_quantity {
            // Take entire record
            remaining_quantity -= record.quantity;
            indexes += 1;
        } else {
            // Take partial record
            record.quantity -= remaining_quantity;
            record.amount = Amount(record.quantity.0 * price(record.amount, record.quantity));
            break;
        }
    }

    for _ in 0..indexes {
        inventory.get_value_mut().pop();
    }

    inventory.transmute_ref_mut()
}

impl Inventory {
    fn push(mut self, time_unix: i64, quantity: f64, amount: f64) -> Self {
        self.get_value_mut().push(InventoryRecord {
            time_unix: TimeUnix(time_unix),
            quantity: Quantity(quantity),
            amount: Amount(amount),
        });
        self
    }
}

impl AccountingEntry {
    const fn set_time(mut self, time_unix: i64) -> Self {
        self.get_value_mut().time_unix = TimeUnix(time_unix);
        self
    }

    fn push(
        mut self,
        cost_flow_type: CostFlowType,
        account_id: u64,
        quantity: f64,
        amount: f64,
    ) -> Self {
        self.get_value_mut().double_entry.push(SingleEntry {
            cost_flow_type: (cost_flow_type),
            account_id: AccountID(account_id),
            quantity: Quantity(quantity),
            amount: Amount(amount),
        });
        self
    }
}

impl AccountInfo {
    fn push(mut self, account_id: u64, nature: Nature, inventory: Inventory) -> Self {
        self.get_value_mut()
            .insert(AccountID(account_id), (nature, inventory));
        self
    }

    fn add(
        mut self,
        account_id: u64,
        nature: Nature,
        time_unix: i64,
        quantity: f64,
        amount: f64,
    ) -> Self {
        let x = self.get_value_mut().get_mut(&AccountID(account_id));
        match x {
            None => {
                self.get_value_mut().insert(
                    AccountID(account_id),
                    (
                        nature,
                        Inventory::default().push(time_unix, quantity, amount),
                    ),
                );
            }
            Some(a) => {
                a.0 = nature;
                let x = a.1.get_value_mut();
                x.push(InventoryRecord {
                    time_unix: TimeUnix(time_unix),
                    quantity: Quantity(quantity),
                    amount: Amount(amount),
                });
            }
        };

        self
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_check_entry() {
        let mut account_info: AccountInfo = AccountInfo::default();

        ////////////////////////////////////////////////////////////////////////////////////////////////////////
        let r = check_entry(TimeUnix(0), AccountingEntry::default(), account_info);

        let r = r.unwrap_err();
        let mut account_info = r.moved_in_variables.1;
        assert_eq!(r.error_code, ErrorCode::YourEntryIsEmpty);

        ////////////////////////////////////////////////////////////////////////////////////////////////////////
        let r = check_entry(
            TimeUnix(0),
            AccountingEntry::default().set_time(-1).push(
                CostFlowType::InFlow(InFlowType::None),
                0,
                0.0,
                0.0,
            ),
            account_info,
        );

        let r = r.unwrap_err();
        let mut account_info = r.moved_in_variables.1;
        assert_eq!(r.error_code, ErrorCode::TimeShouldBeBigger);

        ////////////////////////////////////////////////////////////////////////////////////////////////////////
        let r = check_entry(
            TimeUnix(0),
            AccountingEntry::default().set_time(0).push(
                CostFlowType::InFlow(InFlowType::None),
                0,
                0.0,
                0.0,
            ),
            account_info,
        );

        let r = r.unwrap_err();
        let mut account_info = r.moved_in_variables.1;
        assert_eq!(
            r.error_code,
            ErrorCode::QuantityAndAmountAreZero(AccountID(0))
        );

        ////////////////////////////////////////////////////////////////////////////////////////////////////////
        let r = check_entry(
            TimeUnix(0),
            AccountingEntry::default().set_time(0).push(
                CostFlowType::InFlow(InFlowType::None),
                0,
                -10.0,
                0.0,
            ),
            account_info,
        );

        let r = r.unwrap_err();
        let mut account_info = r.moved_in_variables.1;
        assert_eq!(
            r.error_code,
            ErrorCode::TheQuantityAndAmountShouldBeBothPositive(AccountID(0))
        );

        ////////////////////////////////////////////////////////////////////////////////////////////////////////
        let r = check_entry(
            TimeUnix(0),
            AccountingEntry::default().set_time(0).push(
                CostFlowType::InFlow(InFlowType::None),
                0,
                10.0,
                0.0,
            ),
            account_info,
        );

        let r = r.unwrap_err();
        let mut account_info = r.moved_in_variables.1;
        assert_eq!(
            r.error_code,
            ErrorCode::AccountInfoNotFoundForAccountID(AccountID(0)),
        );

        ////////////////////////////////////////////////////////////////////////////////////////////////////////
        account_info = account_info.push(0, Nature::Credit, Inventory::default());

        let r = check_entry(
            TimeUnix(0),
            AccountingEntry::default().set_time(0).push(
                CostFlowType::InFlow(InFlowType::None),
                0,
                10.0,
                0.0,
            ),
            account_info,
        );

        let r = r.unwrap_err();
        let mut account_info = r.moved_in_variables.1;
        assert_eq!(r.error_code, ErrorCode::InventoryIsEmpty(AccountID(0)));

        ////////////////////////////////////////////////////////////////////////////////////////////////////////
        let mut account_info = account_info.add(0, Nature::Credit, 0, 0.0, 0.0);

        let r = check_entry(
            TimeUnix(0),
            AccountingEntry::default().set_time(0).push(
                CostFlowType::InFlow(InFlowType::QuantityEqualAmount),
                0,
                0.0,
                10.0,
            ),
            account_info,
        );

        let r = r.unwrap_err();
        let mut account_info = r.moved_in_variables.1;
        assert_eq!(
            r.error_code,
            ErrorCode::QuantityNotEqualAmount {
                account_id: AccountID(0),
                quantity: Quantity(0.0),
                amount: Amount(10.0)
            },
        );

        ////////////////////////////////////////////////////////////////////////////////////////////////////////
        let r = check_entry(
            TimeUnix(0),
            AccountingEntry::default().set_time(0).push(
                CostFlowType::InFlow(InFlowType::QuantityEqualZero),
                0,
                10.0,
                10.0,
            ),
            account_info,
        );

        let r = r.unwrap_err();
        let mut account_info = r.moved_in_variables.1;
        assert_eq!(r.error_code, ErrorCode::QuantityNotEqualZero(AccountID(0)));

        ////////////////////////////////////////////////////////////////////////////////////////////////////////
        let r = check_entry(
            TimeUnix(0),
            AccountingEntry::default().set_time(0).push(
                CostFlowType::InFlow(InFlowType::None),
                0,
                10.0,
                10.0,
            ),
            account_info,
        );

        let r = r.unwrap_err();
        let mut account_info = r.moved_in_variables.1;
        assert_eq!(
            r.error_code,
            ErrorCode::DebitNotEqualCredit {
                total_debit: Amount(0.0),
                total_credit: Amount(10.0)
            }
        );

        ////////////////////////////////////////////////////////////////////////////////////////////////////////
        let r = check_entry(
            TimeUnix(0),
            AccountingEntry::default().set_time(0).push(
                CostFlowType::InFlow(InFlowType::None),
                0,
                10.0,
                0.0,
            ),
            account_info,
        );

        let (entry, account_info) = r.unwrap();
        let (entry, account_info) = apply_entry_on_inventory(entry, account_info);

        todo!("convert all this boiler plate code to one function for mocking all crate")
    }
}

#[macro_export]
macro_rules! case_name {
    ($name:expr) => {
        format!(
            "line : {} case name : {}",
            std::panic::Location::caller(),
            $name
        )
    };
    () => {
        format!("line : {}", std::panic::Location::caller())
    };
}
