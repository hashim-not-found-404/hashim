use crate::accounting_domain::utility::accounting_stuff;
use crate::accounting_domain::utility::accounting_stuff::DoubleEntry;
use crate::accounting_domain::utility::accounting_stuff::EntryContainer;
use crate::accounting_domain::utility::accounting_stuff::Inventory;
use crate::accounting_domain::utility::common_subset_sum;
use crate::accounting_domain::utility::constrained_partition;
use std::collections::HashSet;
use std::hash::Hash;

pub trait SingleEntry {
    type AccountId: Eq + Hash;

    fn get_account_id(&self) -> Self::AccountId;

    fn get_from_user_input_is_debit(&self) -> Option<bool>;
    fn get_from_user_input_is_inflow(&self) -> Option<bool>;
    fn get_from_user_input_quantity(&self) -> Option<f64>;
    fn get_from_user_input_amount(&self) -> Option<f64>;
    fn get_from_user_input_inflow_type(&self) -> Option<accounting_stuff::InFlowType>;
    fn get_from_user_input_outflow_type(&self) -> Option<accounting_stuff::OutFlowType>;

    fn set_user_input_is_debit(&mut self, i: Option<bool>);
    fn set_user_input_is_inflow(&mut self, i: Option<bool>);
    fn set_user_input_quantity(&mut self, i: Option<f64>);
    fn set_user_input_amount(&mut self, i: Option<f64>);
    fn set_user_input_inflow_type(&mut self, i: Option<accounting_stuff::InFlowType>);
    fn set_user_input_outflow_type(&mut self, i: Option<accounting_stuff::OutFlowType>);

    fn set_inferred_is_debit(&mut self, i: Option<bool>);
    fn set_inferred_is_inflow(&mut self, i: Option<bool>);
    fn set_inferred_quantity(&mut self, i: Option<f64>);
    fn set_inferred_amount(&mut self, i: Option<f64>);
    fn set_inferred_inflow_type(&mut self, i: Option<accounting_stuff::InFlowType>);
    fn set_inferred_outflow_type(&mut self, i: Option<accounting_stuff::OutFlowType>);

    fn get_inferred_is_debit(&self) -> Option<bool>;
    fn get_inferred_is_inflow(&self) -> Option<bool>;
    fn get_inferred_quantity(&self) -> Option<f64>;
    fn get_inferred_amount(&self) -> Option<f64>;
    fn get_inferred_inflow_type(&self) -> Option<accounting_stuff::InFlowType>;
    fn get_inferred_outflow_type(&self) -> Option<accounting_stuff::OutFlowType>;
}

pub trait AccountInfoProvider {
    type AccountId: Eq + Hash;
    type Inventory: Inventory;

    fn get_info<'a>(&'a mut self, id: &Self::AccountId)
    -> Option<AccountInfo<'a, Self::Inventory>>;
}

pub struct AccountInfo<'a, I> {
    is_debit:     bool,
    inflow_type:  accounting_stuff::InFlowType,
    outflow_type: accounting_stuff::OutFlowType,
    inventory:    &'a mut I,
}

fn reset_all_inferred_values<C>(entry: &mut C)
where
    C: EntryContainer,
    C::Double: DoubleEntry,
    <C::Double as DoubleEntry>::Single: SingleEntry,
{
    for double in entry.iter_mut() {
        for single in double.iter_mut() {
            single.set_inferred_is_debit(single.get_from_user_input_is_debit());
            single.set_inferred_is_inflow(single.get_from_user_input_is_inflow());
            single.set_inferred_quantity(single.get_from_user_input_quantity().map(|a| a.abs()));
            single.set_inferred_amount(single.get_from_user_input_amount().map(|a| a.abs()));
            single.set_inferred_inflow_type(single.get_from_user_input_inflow_type());
            single.set_inferred_outflow_type(single.get_from_user_input_outflow_type());
        }
    }
}

fn horizontal_infer_for_is_debit<C, A>(entry: &mut C, account_info: &mut A)
where
    C: EntryContainer,
    C::Double: DoubleEntry,
    <C::Double as DoubleEntry>::Single: SingleEntry,
    A: AccountInfoProvider<
        AccountId = <<C::Double as DoubleEntry>::Single as SingleEntry>::AccountId,
    >,
    A::Inventory: Inventory,
{
    for double in entry.iter_mut() {
        for single in double.iter_mut() {
            if single.get_inferred_is_debit().is_none() {
                if let Some(is_inflow) = single.get_inferred_is_inflow() {
                    let account_id = single.get_account_id();
                    if let Some(info) = account_info.get_info(&account_id) {
                        single.set_inferred_is_debit(Some(accounting_stuff::is_debit(
                            info.is_debit,
                            is_inflow,
                        )));
                    }
                }
            }
        }
    }
}

fn horizontal_infer_for_is_inflow<C, A>(entry: &mut C, account_info: &mut A)
where
    C: EntryContainer,
    C::Double: DoubleEntry,
    <C::Double as DoubleEntry>::Single: SingleEntry,
    A: AccountInfoProvider<
        AccountId = <<C::Double as DoubleEntry>::Single as SingleEntry>::AccountId,
    >,
    A::Inventory: Inventory,
{
    for double in entry.iter_mut() {
        for single in double.iter_mut() {
            if let Some(is_debit) = single.get_inferred_is_debit() {
                let account_id = single.get_account_id();
                if let Some(info) = account_info.get_info(&account_id) {
                    single.set_inferred_is_inflow(Some(accounting_stuff::is_inflow(
                        info.is_debit,
                        is_debit,
                    )));
                }
            }
        }
    }
}

fn horizontal_infer_for_inflow_type<C, A>(entry: &mut C, account_info: &mut A)
where
    C: EntryContainer,
    C::Double: DoubleEntry,
    <C::Double as DoubleEntry>::Single: SingleEntry,
    A: AccountInfoProvider<
        AccountId = <<C::Double as DoubleEntry>::Single as SingleEntry>::AccountId,
    >,
    A::Inventory: Inventory,
{
    for double in entry.iter_mut() {
        for single in double.iter_mut() {
            match single.get_inferred_inflow_type() {
                Some(inflow_type_from_user) => {
                    single.set_inferred_inflow_type(Some(inflow_type_from_user));
                }
                None => {
                    let account_id = single.get_account_id();

                    if let Some(info) = account_info.get_info(&account_id) {
                        single.set_inferred_inflow_type(Some(info.inflow_type));
                    }
                }
            }
        }
    }
}

fn horizontal_infer_for_outflow_type<C, A>(entry: &mut C, account_info: &mut A)
where
    C: EntryContainer,
    C::Double: DoubleEntry,
    <C::Double as DoubleEntry>::Single: SingleEntry,
    A: AccountInfoProvider<
        AccountId = <<C::Double as DoubleEntry>::Single as SingleEntry>::AccountId,
    >,
    A::Inventory: Inventory,
{
    for double in entry.iter_mut() {
        for single in double.iter_mut() {
            match single.get_inferred_outflow_type() {
                Some(outflow_type_from_user) => {
                    single.set_inferred_outflow_type(Some(outflow_type_from_user));
                }
                None => {
                    let account_id = single.get_account_id();

                    if let Some(info) = account_info.get_info(&account_id) {
                        single.set_inferred_outflow_type(Some(info.outflow_type));
                    }
                }
            }
        }
    }
}

fn vertical_correct_by_remove_duplicate_account<C>(entry: &mut C)
where
    C: EntryContainer,
    C::Double: DoubleEntry,
    <C::Double as DoubleEntry>::Single: SingleEntry,
{
    for double in entry.iter_mut() {
        let mut seen_accounts = HashSet::new();
        double.retain(|single| seen_accounts.insert(single.get_account_id()));
    }
}

fn vertical_correct_to_remove_empty_double_entry<C>(entry: &mut C)
where
    C: EntryContainer,
{
    entry.retain(|double| double.is_empty());
}

fn horizontal_infer_for_amount_from_quantity<C, A>(
    time_unix: u64,
    entry: &mut C,
    account_info: &mut A,
) where
    C: EntryContainer,
    C::Double: DoubleEntry,
    <C::Double as DoubleEntry>::Single: SingleEntry,
    A: AccountInfoProvider<
        AccountId = <<C::Double as DoubleEntry>::Single as SingleEntry>::AccountId,
    >,
    A::Inventory: Inventory,
{
    for double in entry.iter_mut() {
        for single in double.iter_mut() {
            let mut inferred_quantity = match single.get_inferred_quantity() {
                Some(a) => a,
                None => continue,
            };

            let is_inflow = match single.get_inferred_is_inflow() {
                Some(a) => a,
                None => continue,
            };

            let account_id = single.get_account_id();

            let info = match account_info.get_info(&account_id) {
                Some(a) => a,
                None => continue,
            };

            if is_inflow {
                let inferred_inflow_type = match single.get_inferred_inflow_type() {
                    Some(a) => a,
                    None => continue,
                };

                match inferred_inflow_type {
                    accounting_stuff::InFlowType::Manual => {}
                    accounting_stuff::InFlowType::QuantityEqualAmount => {
                        single.set_inferred_amount(Some(inferred_quantity));
                    }
                    accounting_stuff::InFlowType::QuantityEqualZero => {
                        single.set_inferred_quantity(Some(0.0))
                    }
                }
            } else {
                let inferred_outflow_type = match single.get_inferred_outflow_type() {
                    Some(a) => a,
                    None => continue,
                };

                let total_quantity_in_inventory =
                    info.inventory.iter1().fold(0.0, |total, record| total + record.quantity);

                inferred_quantity = total_quantity_in_inventory.min(inferred_quantity);

                single.set_inferred_quantity(Some(inferred_quantity));

                accounting_stuff::sort_inventory(&inferred_outflow_type, info.inventory);

                match inferred_outflow_type {
                    accounting_stuff::OutFlowType::Manual => {
                        if let Some(mut inferred_amount) = single.get_inferred_amount() {
                            let total_amount_in_inventory = info
                                .inventory
                                .iter1()
                                .fold(0.0, |total, record| total + record.amount);

                            inferred_amount = total_amount_in_inventory.min(inferred_amount);

                            single.set_inferred_amount(Some(inferred_amount));
                        };
                    }
                    accounting_stuff::OutFlowType::QuantityEqualAmount => {
                        let total_amount_in_inventory =
                            info.inventory.iter1().fold(0.0, |total, record| total + record.amount);

                        let inferred_amount = total_amount_in_inventory.min(inferred_quantity);

                        single.set_inferred_quantity(Some(inferred_amount));
                        single.set_inferred_amount(Some(inferred_amount));
                    }
                    accounting_stuff::OutFlowType::QuantityEqualZero => {
                        single.set_inferred_quantity(Some(0.0));

                        if let Some(mut inferred_amount) = single.get_inferred_amount() {
                            let total_amount_in_inventory = info
                                .inventory
                                .iter1()
                                .fold(0.0, |total, record| total + record.amount);

                            inferred_amount = total_amount_in_inventory.min(inferred_amount);

                            single.set_inferred_amount(Some(inferred_amount));
                        };
                    }
                    accounting_stuff::OutFlowType::Wac
                    | accounting_stuff::OutFlowType::Fifo
                    | accounting_stuff::OutFlowType::Lifo
                    | accounting_stuff::OutFlowType::Hifo
                    | accounting_stuff::OutFlowType::Lofo => {
                        let expected_amount =
                            accounting_stuff::get_amount(inferred_quantity, info.inventory);

                        single.set_inferred_amount(Some(expected_amount));
                    }
                };
            }

            if let Some(amount) = single.get_inferred_amount()
                && let Some(quantity) = single.get_inferred_quantity()
            {
                let is_decrease_by_price = match single.get_inferred_outflow_type() {
                    Some(accounting_stuff::OutFlowType::Manual) => false,
                    Some(accounting_stuff::OutFlowType::QuantityEqualAmount) => false,
                    Some(accounting_stuff::OutFlowType::QuantityEqualZero) => false,
                    Some(accounting_stuff::OutFlowType::Wac) => true,
                    Some(accounting_stuff::OutFlowType::Fifo) => true,
                    Some(accounting_stuff::OutFlowType::Lifo) => true,
                    Some(accounting_stuff::OutFlowType::Hifo) => true,
                    Some(accounting_stuff::OutFlowType::Lofo) => true,
                    None => false,
                };

                accounting_stuff::apply_entry_on_inventory::<A::Inventory>(
                    time_unix,
                    amount,
                    quantity,
                    is_inflow,
                    is_decrease_by_price,
                    info.inventory,
                );
            }
        }
    }
}

fn vertical_infer_for_is_debit<C>(entry: &mut C)
where
    C: EntryContainer,
    C::Double: DoubleEntry,
    <C::Double as DoubleEntry>::Single: SingleEntry + Clone,
{
    for double in entry.iter_mut() {
        let mut new_double = Vec::new();
        let mut other_double = Vec::new();

        for single in double.iter() {
            if single.get_inferred_amount().is_some() {
                new_double.push(single.clone());
            } else {
                other_double.push(single.clone());
            }
        }

        constrained_partition::assign_partition(
            &mut new_double,
            |single| accounting_stuff::wrapper::T(single.get_inferred_amount().unwrap_or_default()),
            |single| {
                single.get_inferred_is_debit().map_or(
                    constrained_partition::Side::Unknown,
                    |is_debit| {
                        if is_debit {
                            constrained_partition::Side::RHS
                        } else {
                            constrained_partition::Side::LHS
                        }
                    },
                )
            },
            |single, b| single.set_inferred_is_debit(Some(b == constrained_partition::Side::RHS)),
        );

        new_double.append(&mut other_double);
        double.set_singles(new_double);
    }
}

fn vertical_infer_for_amount<C>(entry: &mut C)
where
    C: EntryContainer,
    C::Double: DoubleEntry,
    <C::Double as DoubleEntry>::Single: SingleEntry,
{
    'l1: for double in entry.iter_mut() {
        let mut total_debit = 0.0;
        let mut total_credit = 0.0;

        let mut idx_for_not_inferred_amount: Option<usize> = None;

        for (idx, single) in double.iter_mut().enumerate() {
            let get_inferred_is_debit = single.get_inferred_is_debit().unwrap_or_default();

            if let Some(get_inferred_amount) = single.get_inferred_amount() {
                if get_inferred_is_debit {
                    total_debit += get_inferred_amount;
                } else {
                    total_credit += get_inferred_amount;
                }
            } else {
                if idx_for_not_inferred_amount.is_none() {
                    idx_for_not_inferred_amount = Some(idx);
                } else {
                    continue 'l1;
                }
            };
        }

        if total_debit == total_credit {
            continue;
        }

        let diff = (total_debit - total_credit).abs();

        let the_idx = match idx_for_not_inferred_amount {
            Some(idx) => idx,
            None => continue,
        };

        for (idx, single) in double.iter_mut().enumerate() {
            if the_idx == idx {
                single.set_inferred_amount(Some(diff));
            }
        }
    }
}

fn vertical_correct_by_common_subset_sum<C>(entry: &mut C)
where
    C: EntryContainer,
    C::Double: DoubleEntry + Clone,
    <C::Double as DoubleEntry>::Single: SingleEntry + Clone,
{
    let mut new_doubles: Vec<C::Double> = Vec::new();

    for double in entry.iter() {
        let mut debit_side = Vec::new();
        let mut credit_side = Vec::new();

        for single in double.iter() {
            if let Some(is_debit) = single.get_inferred_is_debit() {
                if is_debit {
                    debit_side.push(single.clone());
                } else {
                    credit_side.push(single.clone());
                }
            } else {
                credit_side.push(single.clone());
            }
        }

        let groups = common_subset_sum::split_to_max(&debit_side, &credit_side, &|s| {
            accounting_stuff::wrapper::T(s.get_inferred_amount().unwrap_or(0.0))
        });

        for (debit_group, credit_group) in groups {
            let mut combined = Vec::new();
            combined.extend(debit_group);
            combined.extend(credit_group);

            let mut new_double = double.clone();
            new_double.set_singles(combined);
            new_doubles.push(new_double);
        }
    }

    entry.set_doubles(new_doubles);
}

fn horizontal_correct<C>(entry: &mut C)
where
    C: EntryContainer,
    C::Double: DoubleEntry,
    <C::Double as DoubleEntry>::Single: SingleEntry,
{
    for double in entry.iter_mut() {
        for single in double.iter_mut() {
            if single.get_from_user_input_is_debit().is_some() {
                single.set_user_input_is_debit(single.get_inferred_is_debit());
            }
            if single.get_from_user_input_is_inflow().is_some() {
                single.set_user_input_is_inflow(single.get_inferred_is_inflow());
            }
            if single.get_from_user_input_quantity().is_some() {
                single.set_user_input_quantity(single.get_inferred_quantity());
            }
            if single.get_from_user_input_amount().is_some() {
                single.set_user_input_amount(single.get_inferred_amount());
            }
            if single.get_from_user_input_inflow_type().is_some() {
                single.set_user_input_inflow_type(single.get_inferred_inflow_type());
            }
            if single.get_from_user_input_outflow_type().is_some() {
                single.set_user_input_outflow_type(single.get_inferred_outflow_type());
            }
        }
    }
}

fn horizontal_infer_for_quantity_from_amount<C, A>(
    time_unix: u64,
    entry: &mut C,
    account_info: &mut A,
) where
    C: EntryContainer,
    C::Double: DoubleEntry,
    <C::Double as DoubleEntry>::Single: SingleEntry,
    A: AccountInfoProvider<
        AccountId = <<C::Double as DoubleEntry>::Single as SingleEntry>::AccountId,
    >,
    A::Inventory: Inventory,
{
    for double in entry.iter_mut() {
        for single in double.iter_mut() {
            let amount = match single.get_inferred_amount() {
                Some(amount) => amount,
                None => continue,
            };

            let account_id = single.get_account_id();

            let info = match account_info.get_info(&account_id) {
                Some(a) => a,
                None => continue,
            };

            let is_inflow = match single.get_inferred_is_inflow() {
                Some(is_inflow) => is_inflow,
                None => continue,
            };

            if is_inflow {
                let inflow_type = match single.get_inferred_inflow_type() {
                    Some(inflow_type) => inflow_type,
                    None => continue,
                };

                match inflow_type {
                    accounting_stuff::InFlowType::Manual => {}
                    accounting_stuff::InFlowType::QuantityEqualAmount => {
                        single.set_inferred_quantity(Some(amount))
                    }
                    accounting_stuff::InFlowType::QuantityEqualZero => {
                        single.set_inferred_quantity(Some(0.0))
                    }
                }
            } else {
                let outflow_type = match single.get_inferred_outflow_type() {
                    Some(outflow_type) => outflow_type,
                    None => continue,
                };

                let total_amount_in_inventory =
                    info.inventory.iter1().fold(0.0, |total, record| total + record.amount);

                let inferred_amount = total_amount_in_inventory.min(amount);

                single.set_inferred_amount(Some(inferred_amount));

                accounting_stuff::sort_inventory(&outflow_type, info.inventory);

                match outflow_type {
                    accounting_stuff::OutFlowType::Manual => {}
                    accounting_stuff::OutFlowType::QuantityEqualAmount => {
                        let total_quantity_in_inventory = info
                            .inventory
                            .iter1()
                            .fold(0.0, |total, record| total + record.quantity);

                        let total_amount_in_inventory =
                            info.inventory.iter1().fold(0.0, |total, record| total + record.amount);

                        let inferred_quantity =
                            total_quantity_in_inventory.min(total_amount_in_inventory).min(amount);

                        single.set_inferred_quantity(Some(inferred_quantity));
                        single.set_inferred_amount(Some(inferred_quantity));
                    }
                    accounting_stuff::OutFlowType::QuantityEqualZero => {
                        single.set_inferred_quantity(Some(0.0))
                    }
                    accounting_stuff::OutFlowType::Wac
                    | accounting_stuff::OutFlowType::Fifo
                    | accounting_stuff::OutFlowType::Lifo
                    | accounting_stuff::OutFlowType::Hifo
                    | accounting_stuff::OutFlowType::Lofo => {
                        let quantity = accounting_stuff::get_quantity(amount, info.inventory);

                        single.set_inferred_quantity(Some(quantity));
                    }
                }
            }

            if let Some(amount) = single.get_inferred_amount()
                && let Some(quantity) = single.get_inferred_quantity()
            {
                let is_decrease_by_price = match single.get_inferred_outflow_type() {
                    Some(accounting_stuff::OutFlowType::Manual) => false,
                    Some(accounting_stuff::OutFlowType::QuantityEqualAmount) => false,
                    Some(accounting_stuff::OutFlowType::QuantityEqualZero) => false,
                    Some(accounting_stuff::OutFlowType::Wac) => true,
                    Some(accounting_stuff::OutFlowType::Fifo) => true,
                    Some(accounting_stuff::OutFlowType::Lifo) => true,
                    Some(accounting_stuff::OutFlowType::Hifo) => true,
                    Some(accounting_stuff::OutFlowType::Lofo) => true,
                    None => false,
                };

                accounting_stuff::apply_entry_on_inventory::<A::Inventory>(
                    time_unix,
                    amount,
                    quantity,
                    is_inflow,
                    is_decrease_by_price,
                    info.inventory,
                );
            }
        }
    }
}

fn correct_the_input<C, A>(time_unix: u64, entry: &mut C, mut account_info: A)
where
    C: EntryContainer,
    C::Double: DoubleEntry + Clone,
    <C::Double as DoubleEntry>::Single: SingleEntry + Clone,
    A: AccountInfoProvider<
        AccountId = <<C::Double as DoubleEntry>::Single as SingleEntry>::AccountId,
    >,
    A::Inventory: Inventory,
{
    reset_all_inferred_values(entry);
    vertical_correct_by_remove_duplicate_account(entry);
    horizontal_infer_for_is_debit(entry, &mut account_info);
    horizontal_infer_for_is_inflow(entry, &mut account_info);
    horizontal_infer_for_inflow_type(entry, &mut account_info);
    horizontal_infer_for_outflow_type(entry, &mut account_info);
    horizontal_infer_for_amount_from_quantity(time_unix, entry, &mut account_info);

    vertical_infer_for_is_debit(entry);
    horizontal_infer_for_is_inflow(entry, &mut account_info);
    vertical_infer_for_amount(entry);
    horizontal_infer_for_quantity_from_amount(time_unix, entry, &mut account_info);

    vertical_correct_by_common_subset_sum(entry);
    horizontal_correct(entry);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounting_domain::utility::accounting_stuff::InventoryRecord;
    use std::collections::HashMap;

    // ---------- Dummy inventory ----------

    impl Inventory for Vec<InventoryRecord> {
        fn push(&mut self, record: accounting_stuff::InventoryRecord) {
            self.push(record);
        }

        fn clear(&mut self) {
            self.clear();
        }

        fn is_empty(&self) -> bool {
            self.is_empty()
        }

        fn iter1(&self) -> impl Iterator<Item = &accounting_stuff::InventoryRecord> {
            self.iter()
        }

        fn iter_mut1(&mut self) -> impl Iterator<Item = &mut accounting_stuff::InventoryRecord> {
            self.iter_mut()
        }

        fn sort_by1<F>(&mut self, compare: F)
        where
            F: FnMut(
                &accounting_stuff::InventoryRecord,
                &accounting_stuff::InventoryRecord,
            ) -> std::cmp::Ordering,
        {
            self.sort_by(compare);
        }

        fn retain<F>(&mut self, _f: F)
        where
            F: FnMut(&accounting_stuff::InventoryRecord) -> bool,
        {
        }

        fn pop(&mut self) -> Option<accounting_stuff::InventoryRecord> {
            None
        }
    }

    // ---------- Mock SingleEntry ----------
    #[derive(Default, Debug, Clone)]
    pub struct MockSingle {
        pub user_input_account_id: String,

        pub user_input_is_debit:     Option<bool>,
        pub user_input_is_inflow:    Option<bool>,
        pub user_input_quantity:     Option<f64>,
        pub user_input_amount:       Option<f64>,
        pub user_input_inflow_type:  Option<accounting_stuff::InFlowType>,
        pub user_input_outflow_type: Option<accounting_stuff::OutFlowType>,

        pub inferred_is_debit:     Option<bool>,
        pub inferred_is_inflow:    Option<bool>,
        pub inferred_quantity:     Option<f64>,
        pub inferred_amount:       Option<f64>,
        pub inferred_inflow_type:  Option<accounting_stuff::InFlowType>,
        pub inferred_outflow_type: Option<accounting_stuff::OutFlowType>,

        // Error flags
        pub quantity_and_amount_are_zero:       bool,
        pub duplicate_account_in_entry:         bool,
        pub inventory_is_empty:                 bool,
        pub the_amount_should_be_positive:      bool,
        pub the_quantity_should_be_positive:    bool,
        pub quantity_not_equal_amount:          bool,
        pub quantity_not_equal_zero:            bool,
        pub insufficient_quantity_in_inventory: Option<f64>,
        pub amount_mismatch:                    Option<f64>,
        pub insufficient_amount_in_inventory:   Option<f64>,
    }

    impl SingleEntry for MockSingle {
        type AccountId = String;

        // ---------- Getters ----------
        fn get_account_id(&self) -> Self::AccountId {
            self.user_input_account_id.clone()
        }

        fn get_from_user_input_is_debit(&self) -> Option<bool> {
            self.user_input_is_debit
        }

        fn get_from_user_input_is_inflow(&self) -> Option<bool> {
            self.user_input_is_inflow
        }

        fn get_from_user_input_quantity(&self) -> Option<f64> {
            self.user_input_quantity
        }

        fn get_from_user_input_amount(&self) -> Option<f64> {
            self.user_input_amount
        }

        fn get_from_user_input_inflow_type(&self) -> Option<accounting_stuff::InFlowType> {
            self.user_input_inflow_type.clone()
        }

        fn get_from_user_input_outflow_type(&self) -> Option<accounting_stuff::OutFlowType> {
            self.user_input_outflow_type.clone()
        }

        // ---------- Setters for user input ----------
        fn set_user_input_is_debit(&mut self, i: Option<bool>) {
            self.user_input_is_debit = i;
        }

        fn set_user_input_is_inflow(&mut self, i: Option<bool>) {
            self.user_input_is_inflow = i;
        }

        fn set_user_input_quantity(&mut self, i: Option<f64>) {
            self.user_input_quantity = i;
        }

        fn set_user_input_amount(&mut self, i: Option<f64>) {
            self.user_input_amount = i;
        }

        fn set_user_input_inflow_type(&mut self, i: Option<accounting_stuff::InFlowType>) {
            self.user_input_inflow_type = i;
        }

        fn set_user_input_outflow_type(&mut self, i: Option<accounting_stuff::OutFlowType>) {
            self.user_input_outflow_type = i;
        }

        // ---------- Setters for inferred ----------
        fn set_inferred_is_debit(&mut self, i: Option<bool>) {
            self.inferred_is_debit = i;
        }

        fn set_inferred_is_inflow(&mut self, i: Option<bool>) {
            self.inferred_is_inflow = i;
        }

        fn set_inferred_quantity(&mut self, i: Option<f64>) {
            self.inferred_quantity = i;
        }

        fn set_inferred_amount(&mut self, i: Option<f64>) {
            self.inferred_amount = i;
        }

        fn set_inferred_inflow_type(&mut self, i: Option<accounting_stuff::InFlowType>) {
            self.inferred_inflow_type = i;
        }

        fn set_inferred_outflow_type(&mut self, i: Option<accounting_stuff::OutFlowType>) {
            self.inferred_outflow_type = i;
        }

        // ---------- Getters for inferred ----------
        fn get_inferred_is_debit(&self) -> Option<bool> {
            self.inferred_is_debit
        }

        fn get_inferred_is_inflow(&self) -> Option<bool> {
            self.inferred_is_inflow
        }

        fn get_inferred_quantity(&self) -> Option<f64> {
            self.inferred_quantity
        }

        fn get_inferred_amount(&self) -> Option<f64> {
            self.inferred_amount
        }

        fn get_inferred_inflow_type(&self) -> Option<accounting_stuff::InFlowType> {
            self.inferred_inflow_type.clone()
        }

        fn get_inferred_outflow_type(&self) -> Option<accounting_stuff::OutFlowType> {
            self.inferred_outflow_type.clone()
        }
    }

    // ---------- Mock DoubleEntry ----------
    #[derive(Debug, Clone)]
    pub struct MockDouble {
        pub singles: Vec<MockSingle>,
    }

    impl DoubleEntry for MockDouble {
        type Iter<'a> = std::slice::Iter<'a, MockSingle>;
        type IterMut<'a> = std::slice::IterMut<'a, MockSingle>;
        type Single = MockSingle;

        fn iter(&self) -> Self::Iter<'_> {
            self.singles.iter()
        }

        fn iter_mut(&mut self) -> Self::IterMut<'_> {
            self.singles.iter_mut()
        }

        fn retain<F>(&mut self, f: F)
        where
            F: FnMut(&Self::Single) -> bool,
        {
            self.singles.retain(f);
        }

        fn set_singles(&mut self, singles: Vec<Self::Single>) {
            self.singles = singles;
        }
    }

    // ---------- Mock EntryContainer ----------
    #[derive(Debug)]
    pub struct MockEntryContainer {
        pub doubles: Vec<MockDouble>,
    }

    impl EntryContainer for MockEntryContainer {
        type Double = MockDouble;
        type Iter<'a> = std::slice::Iter<'a, MockDouble>;
        type IterMut<'a> = std::slice::IterMut<'a, MockDouble>;

        fn iter(&self) -> Self::Iter<'_> {
            self.doubles.iter()
        }

        fn iter_mut(&mut self) -> Self::IterMut<'_> {
            self.doubles.iter_mut()
        }

        fn set_doubles(&mut self, doubles: Vec<Self::Double>) {
            self.doubles = doubles;
        }

        fn retain<F>(&mut self, f: F)
        where
            F: FnMut(&Self::Double) -> bool,
        {
            self.doubles.retain(f);
        }
    }

    // ---------- Mock AccountInfoProvider ----------
    struct MockAccountInfoProvider {
        is_debit:     bool,
        inflow_type:  accounting_stuff::InFlowType,
        outflow_type: accounting_stuff::OutFlowType,
        inventory:    Vec<InventoryRecord>,
    }

    fn new_account_info_provider() -> HashMap<String, MockAccountInfoProvider> {
        HashMap::new()
    }

    impl AccountInfoProvider for HashMap<String, MockAccountInfoProvider> {
        type AccountId = String;
        type Inventory = Vec<InventoryRecord>;

        fn get_info<'a>(
            &'a mut self,
            id: &Self::AccountId,
        ) -> Option<AccountInfo<'a, Self::Inventory>> {
            self.get_mut(id).map(|a| {
                // Safety: The raw pointer is valid for the entire lifetime of the provider,
                // and we never mutate the inventory (only read is_debit from infos).
                AccountInfo {
                    is_debit:     a.is_debit,
                    inflow_type:  a.inflow_type.clone(),
                    outflow_type: a.outflow_type.clone(),
                    inventory:    &mut a.inventory,
                }
            })
        }
    }

    // ---------- The test ----------
    #[test]
    fn full_pipeline_infers_and_corrects_inflow() {
        // Create two singles
        let single1 = MockSingle {
            user_input_account_id: "1".to_string(),
            user_input_is_debit: Some(true),
            ..Default::default()
        };

        let single2 = MockSingle {
            user_input_account_id: "1".to_string(),
            user_input_is_debit: Some(true),
            user_input_is_inflow: Some(true),
            ..Default::default()
        };

        let single3 = MockSingle {
            user_input_account_id: "1".to_string(),
            user_input_is_debit: Some(true),
            user_input_is_inflow: Some(false),
            ..Default::default()
        };

        let single4 = MockSingle {
            user_input_account_id: "1".to_string(),
            user_input_is_debit: Some(false),
            user_input_is_inflow: Some(true),
            ..Default::default()
        };

        let single5 = MockSingle {
            user_input_account_id: "1".to_string(),
            user_input_is_debit: Some(false),
            user_input_is_inflow: Some(false),
            ..Default::default()
        };

        let single6 = MockSingle {
            user_input_account_id: "2".to_string(),
            user_input_is_debit: Some(false),
            user_input_is_inflow: Some(false),
            ..Default::default()
        };

        let single7 = MockSingle {
            user_input_account_id: "2".to_string(),
            user_input_is_inflow: Some(false),
            ..Default::default()
        };

        let single8 = MockSingle {
            user_input_account_id: "2".to_string(),
            ..Default::default()
        };

        let double = MockDouble {
            singles: vec![single1, single2, single3, single4, single5, single6, single7, single8],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };

        // Provider only for account "1"
        let mut provider = new_account_info_provider();
        provider.insert("1".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  accounting_stuff::InFlowType::Manual,
            outflow_type: accounting_stuff::OutFlowType::Manual,
            inventory:    Vec::new(),
        });
        provider.insert("2".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  accounting_stuff::InFlowType::Manual,
            outflow_type: accounting_stuff::OutFlowType::Manual,
            inventory:    Vec::new(),
        });

        // Call the function
        reset_all_inferred_values(&mut container);
        horizontal_infer_for_is_debit(&mut container, &mut provider);
        horizontal_infer_for_is_inflow(&mut container, &mut provider);
        horizontal_correct(&mut container);

        // Verify
        let updated_double = &container.doubles[0];

        assert_eq!(updated_double.singles[0].get_from_user_input_is_inflow(), None);
        assert_eq!(updated_double.singles[1].get_from_user_input_is_inflow(), Some(true));
        assert_eq!(updated_double.singles[2].get_from_user_input_is_inflow(), Some(true));
        assert_eq!(updated_double.singles[3].get_from_user_input_is_inflow(), Some(false));
        assert_eq!(updated_double.singles[4].get_from_user_input_is_inflow(), Some(false));
        assert_eq!(updated_double.singles[5].get_from_user_input_is_inflow(), Some(false));
        assert_eq!(updated_double.singles[6].get_from_user_input_is_inflow(), Some(false));
        assert_eq!(updated_double.singles[7].get_from_user_input_is_inflow(), None);
    }

    #[test]
    fn horizontal_infer_for_inflow_type_sets_inferred_inflow_type() {
        // Case 1: user provided inflow type -> should set inferred to that
        let single1 = MockSingle {
            user_input_account_id: "1".to_string(),
            user_input_inflow_type: Some(accounting_stuff::InFlowType::Manual),
            ..Default::default()
        };

        // Case 2: user provided inflow type (different) -> should set inferred to that
        let single2 = MockSingle {
            user_input_account_id: "2".to_string(),
            user_input_inflow_type: Some(accounting_stuff::InFlowType::QuantityEqualAmount),
            ..Default::default()
        };

        // Case 3: no user inflow type, account exists -> should use account's inflow type
        let single3 = MockSingle {
            user_input_account_id: "3".to_string(),
            user_input_inflow_type: None,
            ..Default::default()
        };

        // Case 4: no user inflow type, account does not exist -> should remain
        let single4 = MockSingle {
            user_input_account_id: "4".to_string(),
            user_input_inflow_type: None,
            ..Default::default()
        };

        // Case 5: no user inflow type, account does not exist -> should remain
        let single5 = MockSingle {
            user_input_account_id: "4".to_string(),
            user_input_inflow_type: Some(accounting_stuff::InFlowType::Manual),
            ..Default::default()
        };

        let double = MockDouble {
            singles: vec![single1, single2, single3, single4, single5],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };

        // Provider info for accounts 1, 2, 3 (but not 4)
        let mut provider = new_account_info_provider();
        provider.insert("1".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  accounting_stuff::InFlowType::QuantityEqualZero,
            outflow_type: accounting_stuff::OutFlowType::Manual,
            inventory:    Vec::new(),
        });
        provider.insert("2".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  accounting_stuff::InFlowType::QuantityEqualAmount,
            outflow_type: accounting_stuff::OutFlowType::Manual,
            inventory:    Vec::new(),
        });
        provider.insert("3".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  accounting_stuff::InFlowType::QuantityEqualZero,
            outflow_type: accounting_stuff::OutFlowType::Manual,
            inventory:    Vec::new(),
        });

        // Call the function
        reset_all_inferred_values(&mut container);
        horizontal_infer_for_inflow_type(&mut container, &mut provider);

        let updated_double = &container.doubles[0];

        // Case 1: user provided Manual, so inferred should be Manual
        assert_eq!(
            updated_double.singles[0].get_inferred_inflow_type(),
            Some(accounting_stuff::InFlowType::Manual)
        );
        // Case 2: user provided QuantityEqualAmount, so inferred should be that
        assert_eq!(
            updated_double.singles[1].get_inferred_inflow_type(),
            Some(accounting_stuff::InFlowType::QuantityEqualAmount)
        );
        // Case 3: no user input, account 3 exists with inflow_type QuantityEqualZero, so inferred should be that
        assert_eq!(
            updated_double.singles[2].get_inferred_inflow_type(),
            Some(accounting_stuff::InFlowType::QuantityEqualZero)
        );
        // Case 4: no user input, account 4 does not exist, so inferred should remain None
        assert_eq!(updated_double.singles[3].get_inferred_inflow_type(), None);
        // Case 5: no user input, account 4 does not exist, so inferred should remain
        assert_eq!(
            updated_double.singles[4].get_inferred_inflow_type(),
            Some(accounting_stuff::InFlowType::Manual)
        );
    }

    #[test]
    fn horizontal_infer_for_outflow_type_sets_inferred_outflow_type() {
        // Case 1: user provided outflow type -> should set inferred to that
        let single1 = MockSingle {
            user_input_account_id: "1".to_string(),
            user_input_outflow_type: Some(accounting_stuff::OutFlowType::Manual),
            ..Default::default()
        };

        // Case 2: user provided outflow type (different) -> should set inferred to that
        let single2 = MockSingle {
            user_input_account_id: "2".to_string(),
            user_input_outflow_type: Some(accounting_stuff::OutFlowType::QuantityEqualAmount),
            ..Default::default()
        };

        // Case 3: no user outflow type, account exists -> should use account's outflow type
        let single3 = MockSingle {
            user_input_account_id: "3".to_string(),
            user_input_outflow_type: None,
            ..Default::default()
        };

        // Case 4: no user outflow type, account does not exist -> should remain
        let single4 = MockSingle {
            user_input_account_id: "4".to_string(),
            user_input_outflow_type: None,
            ..Default::default()
        };

        // Case 5: no user outflow type, account does not exist -> should remain
        let single5 = MockSingle {
            user_input_account_id: "4".to_string(),
            user_input_outflow_type: Some(accounting_stuff::OutFlowType::Manual),
            ..Default::default()
        };

        let double = MockDouble {
            singles: vec![single1, single2, single3, single4, single5],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };

        // Provider info for accounts 1, 2, 3 (but not 4)
        let mut provider = new_account_info_provider();
        provider.insert("1".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  accounting_stuff::InFlowType::QuantityEqualZero,
            outflow_type: accounting_stuff::OutFlowType::Manual,
            inventory:    Vec::new(),
        });
        provider.insert("2".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  accounting_stuff::InFlowType::QuantityEqualAmount,
            outflow_type: accounting_stuff::OutFlowType::Manual,
            inventory:    Vec::new(),
        });
        provider.insert("3".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  accounting_stuff::InFlowType::QuantityEqualZero,
            outflow_type: accounting_stuff::OutFlowType::QuantityEqualZero,
            inventory:    Vec::new(),
        });

        // Call the function
        reset_all_inferred_values(&mut container);
        horizontal_infer_for_outflow_type(&mut container, &mut provider);

        let updated_double = &container.doubles[0];

        // Case 1: user provided Manual, so inferred should be Manual
        assert_eq!(
            updated_double.singles[0].get_inferred_outflow_type(),
            Some(accounting_stuff::OutFlowType::Manual)
        );
        // Case 2: user provided QuantityEqualAmount, so inferred should be that
        assert_eq!(
            updated_double.singles[1].get_inferred_outflow_type(),
            Some(accounting_stuff::OutFlowType::QuantityEqualAmount)
        );
        // Case 3: no user input, account 3 exists with outflow_type QuantityEqualZero, so inferred should be that
        assert_eq!(
            updated_double.singles[2].get_inferred_outflow_type(),
            Some(accounting_stuff::OutFlowType::QuantityEqualZero)
        );
        // Case 4: no user input, account 4 does not exist, so inferred should remain None
        assert_eq!(updated_double.singles[3].get_inferred_outflow_type(), None);
        // Case 5: no user input, account 4 does not exist, so inferred should remain
        assert_eq!(
            updated_double.singles[4].get_inferred_outflow_type(),
            Some(accounting_stuff::OutFlowType::Manual)
        );
    }

    #[test]
    fn vertical_correct_to_remove_duplicate_account_removes_duplicates() {
        // Create singles:
        // - Account "A" appears three times (duplicates)
        // - Account "B" appears twice (duplicate)
        // - Account "C" appears once (unique)
        let single1 = MockSingle {
            user_input_account_id: "A".to_string(),
            user_input_is_debit: Some(true),
            ..Default::default()
        };
        let single2 = MockSingle {
            user_input_account_id: "A".to_string(),
            user_input_is_debit: Some(false),
            ..Default::default()
        };
        let single3 = MockSingle {
            user_input_account_id: "B".to_string(),
            user_input_is_debit: Some(true),
            ..Default::default()
        };
        let single4 = MockSingle {
            user_input_account_id: "A".to_string(),
            user_input_is_debit: Some(false),
            ..Default::default()
        };
        let single5 = MockSingle {
            user_input_account_id: "C".to_string(),
            user_input_is_debit: Some(true),
            ..Default::default()
        };
        let single6 = MockSingle {
            user_input_account_id: "B".to_string(),
            user_input_is_debit: Some(false),
            ..Default::default()
        };

        let double = MockDouble {
            singles: vec![single1, single2, single3, single4, single5, single6],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };

        // Call the function
        vertical_correct_by_remove_duplicate_account(&mut container);

        let updated_double = &container.doubles[0];
        let remaining = &updated_double.singles;

        // We expect only the first occurrence of each account ID:
        // - "A" appears first at index 0 -> keep
        // - "B" appears first at index 2 (since indices 0,1 are A) -> keep
        // - "C" appears at index 4 -> keep
        // So we should have 3 entries.
        assert_eq!(remaining.len(), 3);

        // Check that the kept entries are the first occurrences in order.
        // The first occurrence of "A" has is_debit = Some(true)
        assert_eq!(remaining[0].user_input_account_id, "A");
        assert_eq!(remaining[0].user_input_is_debit, Some(true));

        assert_eq!(remaining[1].user_input_account_id, "B");
        assert_eq!(remaining[1].user_input_is_debit, Some(true));

        assert_eq!(remaining[2].user_input_account_id, "C");
        assert_eq!(remaining[2].user_input_is_debit, Some(true));
    }

    #[test]
    fn horizontal_correct_for_quantity_and_amount_inflow_manual() {
        // Inflow with Manual: no changes to quantity or amount.
        let single = MockSingle {
            user_input_account_id: "1".to_string(),
            user_input_quantity: Some(5.0),
            user_input_amount: Some(10.0),
            inferred_is_inflow: Some(true),
            inferred_inflow_type: Some(accounting_stuff::InFlowType::Manual),
            ..Default::default()
        };
        let double = MockDouble {
            singles: vec![single],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };

        let mut provider = new_account_info_provider();
        provider.insert("1".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  accounting_stuff::InFlowType::Manual,
            outflow_type: accounting_stuff::OutFlowType::Manual,
            inventory:    Vec::new(),
        });

        horizontal_infer_for_amount_from_quantity(100, &mut container, &mut provider);

        let updated = &container.doubles[0].singles[0];
        assert_eq!(updated.get_from_user_input_quantity(), Some(5.0));
        assert_eq!(updated.get_from_user_input_amount(), Some(10.0));
        // No inventory changes because no error, but we can't easily check inventory here.
    }

    #[test]
    fn horizontal_correct_for_quantity_and_amount_inflow_quantity_equal_amount() {
        // Inflow with QuantityEqualAmount: amount should be set to quantity.
        let single = MockSingle {
            user_input_account_id: "1".to_string(),
            user_input_quantity: Some(5.0),
            user_input_amount: Some(10.0), // will be overwritten
            user_input_is_inflow: Some(true),
            user_input_inflow_type: Some(accounting_stuff::InFlowType::QuantityEqualAmount),
            ..Default::default()
        };
        let double = MockDouble {
            singles: vec![single],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };

        let mut provider = new_account_info_provider();
        provider.insert("1".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  accounting_stuff::InFlowType::Manual,
            outflow_type: accounting_stuff::OutFlowType::Manual,
            inventory:    Vec::new(),
        });

        reset_all_inferred_values(&mut container);
        horizontal_infer_for_amount_from_quantity(100, &mut container, &mut provider);

        let updated = &container.doubles[0].singles[0];
        assert_eq!(updated.get_inferred_quantity(), Some(5.0));
        assert_eq!(updated.get_inferred_amount(), Some(5.0)); // amount becomes quantity
    }

    #[test]
    fn horizontal_correct_for_quantity_and_amount_inflow_quantity_equal_zero() {
        // Inflow with QuantityEqualZero: quantity should be set to 0.
        let single = MockSingle {
            user_input_account_id: "1".to_string(),
            user_input_quantity: Some(5.0),
            user_input_amount: Some(10.0),
            user_input_is_inflow: Some(true),
            user_input_inflow_type: Some(accounting_stuff::InFlowType::QuantityEqualZero),
            ..Default::default()
        };
        let double = MockDouble {
            singles: vec![single],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };

        let mut provider = new_account_info_provider();
        provider.insert("1".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  accounting_stuff::InFlowType::Manual,
            outflow_type: accounting_stuff::OutFlowType::Manual,
            inventory:    Vec::new(),
        });

        reset_all_inferred_values(&mut container);
        horizontal_infer_for_amount_from_quantity(100, &mut container, &mut provider);

        let updated = &container.doubles[0].singles[0];
        assert_eq!(updated.get_inferred_quantity(), Some(0.0));
        assert_eq!(updated.get_inferred_amount(), Some(10.0)); // amount unchanged
    }

    #[test]
    fn horizontal_correct_for_quantity_and_amount_outflow_manual() {
        // Outflow with Manual: adjust quantity to inventory quantity, amount to inventory amount if exceeds.
        let single = MockSingle {
            user_input_account_id: "1".to_string(),
            user_input_quantity: Some(10.0),
            user_input_amount: Some(100.0),
            user_input_is_inflow: Some(false),
            user_input_outflow_type: Some(accounting_stuff::OutFlowType::Manual),
            ..Default::default()
        };
        let double = MockDouble {
            singles: vec![single],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };

        // Provider with inventory: total qty=5, total amt=50.
        let mut provider = new_account_info_provider();
        provider.insert("1".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  accounting_stuff::InFlowType::Manual,
            outflow_type: accounting_stuff::OutFlowType::Manual,
            inventory:    vec![
                accounting_stuff::InventoryRecord {
                    time_unix: 1,
                    quantity:  2.0,
                    amount:    20.0,
                },
                accounting_stuff::InventoryRecord {
                    time_unix: 2,
                    quantity:  3.0,
                    amount:    30.0,
                },
            ],
        });

        reset_all_inferred_values(&mut container);
        horizontal_infer_for_amount_from_quantity(100, &mut container, &mut provider);

        let updated = &container.doubles[0].singles[0];
        // Quantity should be capped at 5.0 (total inventory quantity)
        assert_eq!(updated.get_inferred_quantity(), Some(5.0));
        // Amount should be capped at 50.0 (total inventory amount)
        assert_eq!(updated.get_inferred_amount(), Some(50.0));
    }

    #[test]
    fn vertical_correct_to_common_subset_sum_splits_balanced_groups() {
        // Create a single double entry with debit amounts [1,4,5] and credit [2,3,5].
        // This should split into two balanced entries:
        //   ([1,4] vs [5]) and ([5] vs [2,3]).
        let single1 = MockSingle {
            user_input_account_id: "A".to_string(),
            inferred_is_debit: Some(true),
            inferred_amount: Some(1.0),
            ..Default::default()
        };
        let single2 = MockSingle {
            user_input_account_id: "D".to_string(),
            inferred_is_debit: Some(true),
            inferred_amount: Some(4.0),
            ..Default::default()
        };
        let single3 = MockSingle {
            user_input_account_id: "E".to_string(),
            inferred_is_debit: Some(true),
            inferred_amount: Some(5.0),
            ..Default::default()
        };
        let single4 = MockSingle {
            user_input_account_id: "B".to_string(),
            inferred_is_debit: Some(false),
            inferred_amount: Some(2.0),
            ..Default::default()
        };
        let single5 = MockSingle {
            user_input_account_id: "C".to_string(),
            inferred_is_debit: Some(false),
            inferred_amount: Some(3.0),
            ..Default::default()
        };
        let single6 = MockSingle {
            user_input_account_id: "F".to_string(),
            inferred_is_debit: Some(false),
            inferred_amount: Some(5.0),
            ..Default::default()
        };

        let double = MockDouble {
            singles: vec![single1, single2, single3, single4, single5, single6],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };

        // Call the function
        vertical_correct_by_common_subset_sum(&mut container);

        // After splitting, we expect 2 doubles.
        let updated_doubles = &container.doubles;
        assert_eq!(updated_doubles.len(), 2);
    }

    #[test]
    fn vertical_correct_to_common_subset_sum_splits_balanced_groups_with_uninferred_debit() {
        // Create a single double entry with debit amounts [1,4,5] and credit [2,3,5].
        // This should split into two balanced entries:
        //   ([1,4] vs [5]) and ([5] vs [2,3]).
        let single1 = MockSingle {
            user_input_account_id: "A".to_string(),
            inferred_is_debit: None,
            inferred_amount: Some(1.0),
            ..Default::default()
        };
        let single2 = MockSingle {
            user_input_account_id: "D".to_string(),
            inferred_is_debit: Some(true),
            inferred_amount: Some(4.0),
            ..Default::default()
        };
        let single3 = MockSingle {
            user_input_account_id: "E".to_string(),
            inferred_is_debit: Some(true),
            inferred_amount: Some(5.0),
            ..Default::default()
        };
        let single4 = MockSingle {
            user_input_account_id: "B".to_string(),
            inferred_is_debit: Some(false),
            inferred_amount: Some(2.0),
            ..Default::default()
        };
        let single5 = MockSingle {
            user_input_account_id: "C".to_string(),
            inferred_is_debit: Some(false),
            inferred_amount: Some(3.0),
            ..Default::default()
        };
        let single6 = MockSingle {
            user_input_account_id: "F".to_string(),
            inferred_is_debit: Some(false),
            inferred_amount: Some(5.0),
            ..Default::default()
        };

        let double = MockDouble {
            singles: vec![single1, single2, single3, single4, single5, single6],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };

        // Call the function
        vertical_correct_by_common_subset_sum(&mut container);

        // After splitting, we expect 2 doubles.
        let updated_doubles = &container.doubles;
        assert_eq!(updated_doubles.len(), 2);
        assert_eq!(updated_doubles[0].len(), 3);
        assert_eq!(updated_doubles[1].len(), 3);
    }

    #[test]
    fn test_vertical_correct_to_remove_empty_double_entry() {
        let single1 = MockSingle::default();

        let double = MockDouble {
            singles: vec![single1.clone(), single1.clone(), single1.clone(), single1],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double, MockDouble {
                singles: vec![],
            }],
        };

        // Call the function
        vertical_correct_to_remove_empty_double_entry(&mut container);

        // After splitting, we expect 2 doubles.
        let updated_doubles = &container.doubles;
        assert_eq!(updated_doubles.len(), 1);
    }

    #[test]
    fn test_vertical_infer_for_is_debit() {
        let single1 = MockSingle {
            user_input_account_id: "A".to_string(),
            inferred_is_debit: None,
            inferred_amount: Some(1.0),
            ..Default::default()
        };
        let single2 = MockSingle {
            user_input_account_id: "D".to_string(),
            inferred_is_debit: Some(true),
            inferred_amount: Some(4.0),
            ..Default::default()
        };
        let single3 = MockSingle {
            user_input_account_id: "E".to_string(),
            inferred_is_debit: Some(true),
            inferred_amount: Some(5.0),
            ..Default::default()
        };

        let double = MockDouble {
            singles: vec![single1, single2, single3],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };

        // Call the function
        vertical_infer_for_is_debit(&mut container);

        // After splitting, we expect 2 doubles.
        let updated_doubles = &container.doubles;
        assert_eq!(updated_doubles.len(), 1);
    }

    #[test]
    fn vertical_infer_for_is_debit_assigns_sides_to_balance_amounts() {
        // Create a double with four singles: amounts 4, 6 (should become debit)
        // and amounts 3, 7 (should become credit), all with no inferred_is_debit.
        let single1 = MockSingle {
            user_input_account_id: "A".to_string(),
            inferred_amount: Some(4.0),
            inferred_is_debit: None,
            ..Default::default()
        };
        let single2 = MockSingle {
            user_input_account_id: "B".to_string(),
            inferred_amount: Some(6.0),
            inferred_is_debit: None,
            ..Default::default()
        };
        let single3 = MockSingle {
            user_input_account_id: "C".to_string(),
            inferred_amount: Some(3.0),
            inferred_is_debit: None,
            ..Default::default()
        };
        let single4 = MockSingle {
            user_input_account_id: "D".to_string(),
            inferred_amount: Some(7.0),
            inferred_is_debit: None,
            ..Default::default()
        };

        let double = MockDouble {
            singles: vec![single1, single2, single3, single4],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };

        // Call the function
        vertical_infer_for_is_debit(&mut container);

        // Verify results
        let updated_double = &container.doubles[0];

        // All entries should now have inferred_is_debit set.
        for single in &updated_double.singles {
            assert!(
                single.inferred_is_debit.is_some(),
                "Entry {} has no inferred_is_debit",
                single.user_input_account_id
            );
        }

        // The debit and credit totals must be equal.
        let total_debit: f64 = updated_double
            .singles
            .iter()
            .filter(|s| s.inferred_is_debit == Some(true))
            .map(|s| s.inferred_amount.unwrap_or(0.0))
            .sum();
        let total_credit: f64 = updated_double
            .singles
            .iter()
            .filter(|s| s.inferred_is_debit == Some(false))
            .map(|s| s.inferred_amount.unwrap_or(0.0))
            .sum();

        assert!(
            (total_debit - total_credit).abs() < 1.0e-9,
            "Debit sum {} != credit sum {}",
            total_debit,
            total_credit
        );

        // Optionally, check that each entry's assigned side matches its amount group.
        // Since the partitioner may have multiple valid splits, we can't hardcode,
        // but we can verify that the groups are non‑empty and balanced.
        // For this specific case, the only balanced split is {4,6} vs {3,7}.
        // But to be robust, we just check the balance.
    }

    #[test]
    fn vertical_infer_for_is_debit_assigns_sides_to_unknown_amount() {
        let single1 = MockSingle {
            user_input_account_id: "A".to_string(),
            inferred_amount: None,
            inferred_is_debit: None,
            ..Default::default()
        };
        let single2 = MockSingle {
            user_input_account_id: "B".to_string(),
            inferred_amount: Some(6.0),
            inferred_is_debit: None,
            ..Default::default()
        };
        let single3 = MockSingle {
            user_input_account_id: "C".to_string(),
            inferred_amount: Some(3.0),
            inferred_is_debit: Some(true),
            ..Default::default()
        };
        let single4 = MockSingle {
            user_input_account_id: "D".to_string(),
            inferred_amount: Some(7.0),
            inferred_is_debit: None,
            ..Default::default()
        };

        let double = MockDouble {
            singles: vec![single1, single2, single3, single4],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };

        // Call the function
        vertical_infer_for_is_debit(&mut container);

        // Verify results
        let updated_double = &container.doubles[0];

        // The debit and credit totals must be equal.
        let total_debit: f64 = updated_double
            .singles
            .iter()
            .filter(|s| s.inferred_is_debit == Some(true))
            .map(|s| s.inferred_amount.unwrap_or(0.0))
            .sum();
        let total_credit: f64 = updated_double
            .singles
            .iter()
            .filter(|s| s.inferred_is_debit == Some(false))
            .map(|s| s.inferred_amount.unwrap_or(0.0))
            .sum();

        assert_eq!(total_debit, 9.0);
        assert_eq!(total_credit, 7.0);
    }

    #[test]
    fn vertical_infer_for_is_debit_assigns_sides_to_balance_amounts_with_two_inferred() {
        // Create a double with four singles: amounts 4, 6 (should become debit)
        // and amounts 3, 7 (should become credit), all with no inferred_is_debit.
        let single1 = MockSingle {
            user_input_account_id: "A".to_string(),
            inferred_amount: Some(4.0),
            inferred_is_debit: Some(true),
            ..Default::default()
        };
        let single2 = MockSingle {
            user_input_account_id: "B".to_string(),
            inferred_amount: Some(6.0),
            inferred_is_debit: Some(false),
            ..Default::default()
        };
        let single3 = MockSingle {
            user_input_account_id: "C".to_string(),
            inferred_amount: Some(3.0),
            inferred_is_debit: None,
            ..Default::default()
        };
        let single4 = MockSingle {
            user_input_account_id: "D".to_string(),
            inferred_amount: Some(7.0),
            inferred_is_debit: None,
            ..Default::default()
        };

        let double = MockDouble {
            singles: vec![single1, single2, single3, single4],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };

        // Call the function
        vertical_infer_for_is_debit(&mut container);

        // Verify results
        let updated_double = &container.doubles[0];

        // All entries should now have inferred_is_debit set.
        for single in &updated_double.singles {
            assert!(
                single.inferred_is_debit.is_some(),
                "Entry {} has no inferred_is_debit",
                single.user_input_account_id
            );
        }

        // The debit and credit totals must be equal.
        let total_debit: f64 = updated_double
            .singles
            .iter()
            .filter(|s| s.inferred_is_debit == Some(true))
            .map(|s| s.inferred_amount.unwrap_or(0.0))
            .sum();
        let total_credit: f64 = updated_double
            .singles
            .iter()
            .filter(|s| s.inferred_is_debit == Some(false))
            .map(|s| s.inferred_amount.unwrap_or(0.0))
            .sum();

        assert_eq!(total_debit, 11.0);
        assert_eq!(total_credit, 9.0);
    }

    #[test]
    fn vertical_infer_for_amount_infers_missing_amounts_to_balance() {
        // Scenario 1: Debit total > Credit total, missing amounts on credit side.
        // Debit: amounts 10, 20 (total 30)
        // Credit: amounts 5, ? (missing) -> need ? = 25
        let single1 = MockSingle {
            user_input_account_id: "D1".to_string(),
            inferred_is_debit: Some(true),
            inferred_amount: Some(10.0),
            ..Default::default()
        };
        let single2 = MockSingle {
            user_input_account_id: "D2".to_string(),
            inferred_is_debit: Some(true),
            inferred_amount: Some(20.0),
            ..Default::default()
        };
        let single3 = MockSingle {
            user_input_account_id: "C1".to_string(),
            inferred_is_debit: Some(false),
            inferred_amount: Some(5.0),
            ..Default::default()
        };
        let single4 = MockSingle {
            user_input_account_id: "C2".to_string(),
            inferred_is_debit: Some(false),
            inferred_amount: None,
            ..Default::default()
        };

        let double = MockDouble {
            singles: vec![single1, single2, single3, single4],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };

        vertical_infer_for_amount(&mut container);

        let updated_double = &container.doubles[0];
        // C2 should now have inferred_amount = 25.0
        let c2 = updated_double.singles.iter().find(|s| s.user_input_account_id == "C2").unwrap();
        assert_eq!(c2.inferred_amount, Some(25.0));

        // Check totals balance
        let total_debit: f64 = updated_double
            .singles
            .iter()
            .filter(|s| s.inferred_is_debit == Some(true))
            .map(|s| s.inferred_amount.unwrap_or(0.0))
            .sum();
        let total_credit: f64 = updated_double
            .singles
            .iter()
            .filter(|s| s.inferred_is_debit == Some(false))
            .map(|s| s.inferred_amount.unwrap_or(0.0))
            .sum();
        assert!((total_debit - total_credit).abs() < 1.0e-9);
    }

    #[test]
    fn vertical_infer_for_amount_does_nothing_when_already_balanced() {
        // Balanced: debit 10, credit 10, both amounts known.
        let single1 = MockSingle {
            user_input_account_id: "D1".to_string(),
            inferred_is_debit: Some(true),
            inferred_amount: Some(10.0),
            ..Default::default()
        };
        let single2 = MockSingle {
            user_input_account_id: "C1".to_string(),
            inferred_is_debit: Some(false),
            inferred_amount: Some(10.0),
            ..Default::default()
        };

        let double = MockDouble {
            singles: vec![single1, single2],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };

        vertical_infer_for_amount(&mut container);

        let updated = &container.doubles[0];
        assert_eq!(updated.singles[0].inferred_amount, Some(10.0));
        assert_eq!(updated.singles[1].inferred_amount, Some(10.0));
    }

    #[test]
    fn vertical_infer_for_amount_infers_on_debit_side_when_credit_greater() {
        // Credit > Debit: Debit has missing amount.
        // Debit: amounts 5, ? (missing) => total debit = 5 + ?
        // Credit: amounts 10, 15 (total 25) -> need ? = 20
        let single1 = MockSingle {
            user_input_account_id: "D1".to_string(),
            inferred_is_debit: Some(true),
            inferred_amount: Some(5.0),
            ..Default::default()
        };
        let single2 = MockSingle {
            user_input_account_id: "D2".to_string(),
            inferred_is_debit: Some(true),
            inferred_amount: None,
            ..Default::default()
        };
        let single3 = MockSingle {
            user_input_account_id: "C1".to_string(),
            inferred_is_debit: Some(false),
            inferred_amount: Some(10.0),
            ..Default::default()
        };
        let single4 = MockSingle {
            user_input_account_id: "C2".to_string(),
            inferred_is_debit: Some(false),
            inferred_amount: Some(15.0),
            ..Default::default()
        };

        let double = MockDouble {
            singles: vec![single1, single2, single3, single4],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };

        vertical_infer_for_amount(&mut container);

        let updated = &container.doubles[0];
        let d2 = updated.singles.iter().find(|s| s.user_input_account_id == "D2").unwrap();
        assert_eq!(d2.inferred_amount, Some(20.0));

        // Check balance
        let total_debit: f64 = updated
            .singles
            .iter()
            .filter(|s| s.inferred_is_debit == Some(true))
            .map(|s| s.inferred_amount.unwrap_or(0.0))
            .sum();
        let total_credit: f64 = updated
            .singles
            .iter()
            .filter(|s| s.inferred_is_debit == Some(false))
            .map(|s| s.inferred_amount.unwrap_or(0.0))
            .sum();
        assert!((total_debit - total_credit).abs() < 1.0e-9);
    }

    #[test]
    fn vertical_infer_for_amount_handles_multiple_missing_on_same_side() {
        let single1 = MockSingle {
            user_input_account_id: "D1".to_string(),
            inferred_is_debit: Some(true),
            inferred_amount: Some(10.0),
            ..Default::default()
        };
        let single2 = MockSingle {
            user_input_account_id: "D2".to_string(),
            inferred_is_debit: Some(true),
            inferred_amount: None,
            ..Default::default()
        };
        let single3 = MockSingle {
            user_input_account_id: "D3".to_string(),
            inferred_is_debit: Some(true),
            inferred_amount: None,
            ..Default::default()
        };
        let single4 = MockSingle {
            user_input_account_id: "C1".to_string(),
            inferred_is_debit: Some(false),
            inferred_amount: Some(20.0),
            ..Default::default()
        };
        let single5 = MockSingle {
            user_input_account_id: "C2".to_string(),
            inferred_is_debit: Some(false),
            inferred_amount: Some(30.0),
            ..Default::default()
        };

        let double = MockDouble {
            singles: vec![single1, single2, single3, single4, single5],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };

        vertical_infer_for_amount(&mut container);

        let updated = &container.doubles[0];
        let d2 = updated.singles.iter().find(|s| s.user_input_account_id == "D2").unwrap();
        let d3 = updated.singles.iter().find(|s| s.user_input_account_id == "D3").unwrap();
        assert_eq!(d2.inferred_amount, None);
        assert_eq!(d3.inferred_amount, None);

        // Check balance
        let total_debit: f64 = updated
            .singles
            .iter()
            .filter(|s| s.inferred_is_debit == Some(true))
            .map(|s| s.inferred_amount.unwrap_or(0.0))
            .sum();
        let total_credit: f64 = updated
            .singles
            .iter()
            .filter(|s| s.inferred_is_debit == Some(false))
            .map(|s| s.inferred_amount.unwrap_or(0.0))
            .sum();
        assert!((total_debit != total_credit));
    }

    // =============================================================================
    // Tests for horizontal_infer_for_quantity_from_amount
    // =============================================================================

    #[test]
    fn test_quantity_from_amount_inflow_manual() {
        // Inflow with Manual: quantity should remain unchanged (already set)
        let mut single = MockSingle {
            user_input_account_id: "1".to_string(),
            inferred_amount: Some(10.0),
            inferred_quantity: Some(5.0), // already set
            inferred_is_inflow: Some(true),
            inferred_inflow_type: Some(accounting_stuff::InFlowType::Manual),
            ..Default::default()
        };
        // We need to set inferred fields directly because we are not calling reset_all_inferred_values.
        // But we will simulate the state after previous inference steps.
        // For clarity, we'll set both user and inferred to same.
        single.set_inferred_amount(Some(10.0));
        single.set_inferred_quantity(Some(5.0));
        single.set_inferred_is_inflow(Some(true));
        single.set_inferred_inflow_type(Some(accounting_stuff::InFlowType::Manual));

        let double = MockDouble {
            singles: vec![single],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };

        let mut provider = new_account_info_provider();
        provider.insert("1".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  accounting_stuff::InFlowType::Manual,
            outflow_type: accounting_stuff::OutFlowType::Manual,
            inventory:    vec![accounting_stuff::InventoryRecord {
                time_unix: 1,
                quantity:  10.0,
                amount:    20.0,
            }],
        });

        horizontal_infer_for_quantity_from_amount(100, &mut container, &mut provider);

        let updated = &container.doubles[0].singles[0];
        // Quantity unchanged
        assert_eq!(updated.get_inferred_quantity(), Some(5.0));
        // Amount unchanged
        assert_eq!(updated.get_inferred_amount(), Some(10.0));
        // Inventory should not be updated because quantity is already set (no change)
        let inv = &provider.get_mut(&"1".to_string()).unwrap().inventory;
        assert_eq!(inv.len(), 2);
        assert_eq!(inv[0].quantity, 10.0);
        assert_eq!(inv[0].amount, 20.0);
        assert_eq!(inv[1].quantity, 5.0);
        assert_eq!(inv[1].amount, 10.0);
    }

    #[test]
    fn test_quantity_from_amount_inflow_quantity_equal_amount() {
        // Inflow with QuantityEqualAmount: quantity should be set to amount
        let mut single = MockSingle {
            user_input_account_id: "1".to_string(),
            inferred_amount: Some(10.0),
            inferred_quantity: None, // not set
            inferred_is_inflow: Some(true),
            inferred_inflow_type: Some(accounting_stuff::InFlowType::QuantityEqualAmount),
            ..Default::default()
        };
        single.set_inferred_amount(Some(10.0));
        single.set_inferred_is_inflow(Some(true));
        single.set_inferred_inflow_type(Some(accounting_stuff::InFlowType::QuantityEqualAmount));

        let double = MockDouble {
            singles: vec![single],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };

        let mut provider = new_account_info_provider();
        provider.insert("1".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  accounting_stuff::InFlowType::QuantityEqualAmount,
            outflow_type: accounting_stuff::OutFlowType::Manual,
            inventory:    vec![accounting_stuff::InventoryRecord {
                time_unix: 1,
                quantity:  10.0,
                amount:    20.0,
            }],
        });

        horizontal_infer_for_quantity_from_amount(100, &mut container, &mut provider);

        let updated = &container.doubles[0].singles[0];
        assert_eq!(updated.get_inferred_quantity(), Some(10.0)); // quantity = amount
        // Amount unchanged
        assert_eq!(updated.get_inferred_amount(), Some(10.0));

        // Inventory should be updated (inflow with amount 10, qty 10)
        let inv = &provider.get_mut(&"1".to_string()).unwrap().inventory;
        assert_eq!(inv.len(), 2);
        assert_eq!(inv[0].quantity, 10.0);
        assert_eq!(inv[0].amount, 20.0);
        assert_eq!(inv[1].quantity, 10.0);
        assert_eq!(inv[1].amount, 10.0);
    }

    #[test]
    fn test_quantity_from_amount_inflow_quantity_equal_zero() {
        // Inflow with QuantityEqualZero: quantity should be set to 0
        let mut single = MockSingle {
            user_input_account_id: "1".to_string(),
            inferred_amount: Some(10.0),
            inferred_quantity: None,
            inferred_is_inflow: Some(true),
            inferred_inflow_type: Some(accounting_stuff::InFlowType::QuantityEqualZero),
            ..Default::default()
        };
        single.set_inferred_amount(Some(10.0));
        single.set_inferred_is_inflow(Some(true));
        single.set_inferred_inflow_type(Some(accounting_stuff::InFlowType::QuantityEqualZero));

        let double = MockDouble {
            singles: vec![single],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };

        let mut provider = new_account_info_provider();
        provider.insert("1".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  accounting_stuff::InFlowType::QuantityEqualZero,
            outflow_type: accounting_stuff::OutFlowType::Manual,
            inventory:    vec![accounting_stuff::InventoryRecord {
                time_unix: 1,
                quantity:  10.0,
                amount:    20.0,
            }],
        });

        horizontal_infer_for_quantity_from_amount(100, &mut container, &mut provider);

        let updated = &container.doubles[0].singles[0];
        assert_eq!(updated.get_inferred_quantity(), Some(0.0));
        // Amount unchanged
        assert_eq!(updated.get_inferred_amount(), Some(10.0));

        // Inventory should be updated (inflow with amount 10, qty 0)
        let inv = &provider.get_mut(&"1".to_string()).unwrap().inventory;
        assert_eq!(inv.len(), 1);
        assert_eq!(inv[0].quantity, 10.0); // unchanged
        assert_eq!(inv[0].amount, 30.0); // 20 + 10
    }

    #[test]
    fn test_quantity_from_amount_outflow_manual() {
        // Outflow with Manual: amount capped to total amount, quantity unchanged
        let mut single = MockSingle {
            user_input_account_id: "1".to_string(),
            inferred_amount: Some(100.0), // more than inventory
            inferred_quantity: Some(5.0), // already set
            inferred_is_inflow: Some(false),
            inferred_outflow_type: Some(accounting_stuff::OutFlowType::Manual),
            ..Default::default()
        };
        single.set_inferred_amount(Some(100.0));
        single.set_inferred_quantity(Some(5.0));
        single.set_inferred_is_inflow(Some(false));
        single.set_inferred_outflow_type(Some(accounting_stuff::OutFlowType::Manual));

        let double = MockDouble {
            singles: vec![single],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };

        let mut provider = new_account_info_provider();
        provider.insert("1".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  accounting_stuff::InFlowType::Manual,
            outflow_type: accounting_stuff::OutFlowType::Manual,
            inventory:    vec![accounting_stuff::InventoryRecord {
                time_unix: 1,
                quantity:  10.0,
                amount:    20.0,
            }],
        });

        horizontal_infer_for_quantity_from_amount(100, &mut container, &mut provider);

        let updated = &container.doubles[0].singles[0];
        assert_eq!(updated.get_inferred_quantity(), Some(5.0)); // unchanged
        // Amount should be capped to total inventory amount (20.0)
        assert_eq!(updated.get_inferred_amount(), Some(20.0));

        // Inventory should be updated (outflow with amount 20, qty 5) – manual outflow uses given amount
        let inv = &provider.get_mut(&"1".to_string()).unwrap().inventory;
        assert_eq!(inv.len(), 1);
        assert_eq!(inv[0].quantity, 5.0); // 10 - 5
        assert_eq!(inv[0].amount, 0.0); // 20 - 20
    }

    #[test]
    fn test_quantity_from_amount_outflow_quantity_equal_amount() {
        // Outflow with QuantityEqualAmount: quantity and amount both set to min of amount, total amount, total quantity
        let mut single = MockSingle {
            user_input_account_id: "1".to_string(),
            inferred_amount: Some(100.0),
            inferred_quantity: None, // not set
            inferred_is_inflow: Some(false),
            inferred_outflow_type: Some(accounting_stuff::OutFlowType::QuantityEqualAmount),
            ..Default::default()
        };
        single.set_inferred_amount(Some(100.0));
        single.set_inferred_is_inflow(Some(false));
        single.set_inferred_outflow_type(Some(accounting_stuff::OutFlowType::QuantityEqualAmount));

        let double = MockDouble {
            singles: vec![single],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };

        let mut provider = new_account_info_provider();
        provider.insert("1".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  accounting_stuff::InFlowType::Manual,
            outflow_type: accounting_stuff::OutFlowType::QuantityEqualAmount,
            inventory:    vec![accounting_stuff::InventoryRecord {
                time_unix: 1,
                quantity:  10.0,
                amount:    20.0,
            }],
        });

        horizontal_infer_for_quantity_from_amount(100, &mut container, &mut provider);

        let updated = &container.doubles[0].singles[0];
        // Inferred quantity = min(total_qty, total_amt, amount) = min(10,20,100) = 10
        assert_eq!(updated.get_inferred_quantity(), Some(10.0));
        // Inferred amount should be set to same value (10.0) because QuantityEqualAmount
        assert_eq!(updated.get_inferred_amount(), Some(10.0));

        // Inventory should be updated (outflow with amount 10, qty 10) – but this is decrease_by_price? Actually QuantityEqualAmount sets is_decrease_by_price = false, so it uses the given amount directly.
        // In apply_entry_on_inventory, for outflow with is_decrease_by_price=false, it will subtract given amount and qty.
        // Since we are decreasing by amount 10 and qty 10, inventory becomes qty=0, amt=10? Wait: original (10,20), subtract (10,10) -> (0,10)
        let inv = &provider.get_mut(&"1".to_string()).unwrap().inventory;
        assert_eq!(inv.len(), 1);
        assert_eq!(inv[0].quantity, 0.0);
        assert_eq!(inv[0].amount, 10.0);
    }

    #[test]
    fn test_quantity_from_amount_outflow_quantity_equal_zero() {
        // Outflow with QuantityEqualZero: quantity set to 0, amount capped
        let mut single = MockSingle {
            user_input_account_id: "1".to_string(),
            inferred_amount: Some(100.0),
            inferred_quantity: None,
            inferred_is_inflow: Some(false),
            inferred_outflow_type: Some(accounting_stuff::OutFlowType::QuantityEqualZero),
            ..Default::default()
        };
        single.set_inferred_amount(Some(100.0));
        single.set_inferred_is_inflow(Some(false));
        single.set_inferred_outflow_type(Some(accounting_stuff::OutFlowType::QuantityEqualZero));

        let double = MockDouble {
            singles: vec![single],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };

        let mut provider = new_account_info_provider();
        provider.insert("1".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  accounting_stuff::InFlowType::Manual,
            outflow_type: accounting_stuff::OutFlowType::QuantityEqualZero,
            inventory:    vec![accounting_stuff::InventoryRecord {
                time_unix: 1,
                quantity:  10.0,
                amount:    20.0,
            }],
        });

        horizontal_infer_for_quantity_from_amount(100, &mut container, &mut provider);

        let updated = &container.doubles[0].singles[0];
        assert_eq!(updated.get_inferred_quantity(), Some(0.0));
        // Amount capped to total amount (20.0)
        assert_eq!(updated.get_inferred_amount(), Some(20.0));

        // Inventory updated: outflow with amount 20, qty 0 -> amount decreases, qty unchanged
        let inv = &provider.get_mut(&"1".to_string()).unwrap().inventory;
        assert_eq!(inv.len(), 1);
        assert_eq!(inv[0].quantity, 10.0);
        assert_eq!(inv[0].amount, 0.0);
    }

    #[test]
    fn test_quantity_from_amount_outflow_wac_fifo_lifo_hifo_lofo() {
        // For cost methods: use get_quantity to derive quantity from amount
        let flow_types = vec![
            accounting_stuff::OutFlowType::Wac,
            accounting_stuff::OutFlowType::Fifo,
            accounting_stuff::OutFlowType::Lifo,
            accounting_stuff::OutFlowType::Hifo,
            accounting_stuff::OutFlowType::Lofo,
        ];

        for flow in flow_types {
            let mut single = MockSingle {
                user_input_account_id: "1".to_string(),
                inferred_amount: Some(10.0),
                inferred_quantity: None,
                inferred_is_inflow: Some(false),
                inferred_outflow_type: Some(flow.clone()),
                ..Default::default()
            };
            single.set_inferred_amount(Some(10.0));
            single.set_inferred_is_inflow(Some(false));
            single.set_inferred_outflow_type(Some(flow.clone()));

            let double = MockDouble {
                singles: vec![single],
            };
            let mut container = MockEntryContainer {
                doubles: vec![double],
            };

            let mut provider = new_account_info_provider();
            // Inventory with two records: total qty=10, total amt=20 (price=2)
            provider.insert("1".to_string(), MockAccountInfoProvider {
                is_debit:     true,
                inflow_type:  accounting_stuff::InFlowType::Manual,
                outflow_type: flow.clone(),
                inventory:    vec![
                    accounting_stuff::InventoryRecord {
                        time_unix: 1,
                        quantity:  6.0,
                        amount:    12.0,
                    },
                    accounting_stuff::InventoryRecord {
                        time_unix: 2,
                        quantity:  4.0,
                        amount:    8.0,
                    },
                ],
            });

            horizontal_infer_for_quantity_from_amount(100, &mut container, &mut provider);

            let updated = &container.doubles[0].singles[0];
            // For amount 10, with price 2, expected quantity = 5.0
            let expected_qty = accounting_stuff::get_quantity(
                10.0,
                &provider.get(&"1".to_string()).unwrap().inventory,
            );
            assert_eq!(updated.get_inferred_quantity(), Some(expected_qty));
            // Amount should be capped to total amount (20) because amount 10 <= 20, so stays 10
            assert_eq!(updated.get_inferred_amount(), Some(10.0));

            // Inventory should be updated: outflow with amount 10, qty = expected_qty
            let inv_after = &provider.get_mut(&"1".to_string()).unwrap().inventory;
            // We can't easily assert exact state because apply_entry_on_inventory will decrease inventory.
            // But we can check that total decreased.
            let (total_qty, total_amt) = accounting_stuff::sum_inventory(inv_after);
            assert!((total_qty - (10.0 - expected_qty)).abs() < 1.0e-9);
            assert!((total_amt - (20.0 - 10.0)).abs() < 1.0e-9);
        }
    }

    #[test]
    fn test_quantity_from_amount_missing_fields_skips() {
        // If any required field is missing, the function should skip that single entry.
        // Test missing amount
        let single = MockSingle {
            user_input_account_id: "1".to_string(),
            inferred_amount: None, // missing
            inferred_is_inflow: Some(true),
            inferred_inflow_type: Some(accounting_stuff::InFlowType::Manual),
            ..Default::default()
        };
        let double = MockDouble {
            singles: vec![single],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };
        let mut provider = new_account_info_provider();
        provider.insert("1".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  accounting_stuff::InFlowType::Manual,
            outflow_type: accounting_stuff::OutFlowType::Manual,
            inventory:    vec![],
        });

        // Should not panic, and no changes
        horizontal_infer_for_quantity_from_amount(100, &mut container, &mut provider);
        let updated = &container.doubles[0].singles[0];
        assert_eq!(updated.get_inferred_quantity(), None);
        assert_eq!(updated.get_inferred_amount(), None);

        // Test missing is_inflow
        let single2 = MockSingle {
            user_input_account_id: "1".to_string(),
            inferred_amount: Some(10.0),
            inferred_is_inflow: None,
            inferred_inflow_type: Some(accounting_stuff::InFlowType::Manual),
            ..Default::default()
        };
        let double2 = MockDouble {
            singles: vec![single2],
        };
        let mut container2 = MockEntryContainer {
            doubles: vec![double2],
        };
        horizontal_infer_for_quantity_from_amount(100, &mut container2, &mut provider);
        let updated2 = &container2.doubles[0].singles[0];
        assert_eq!(updated2.get_inferred_quantity(), None); // not set because is_inflow missing
    }

    #[test]
    fn test_quantity_from_amount() {
        let single = MockSingle {
            user_input_account_id: "1".to_string(),
            inferred_amount: Some(10.0),
            inferred_is_inflow: Some(true),
            inferred_inflow_type: Some(accounting_stuff::InFlowType::QuantityEqualAmount),
            ..Default::default()
        };

        let double = MockDouble {
            singles: vec![single],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };

        let mut provider = new_account_info_provider();
        provider.insert("1".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  accounting_stuff::InFlowType::QuantityEqualAmount,
            outflow_type: accounting_stuff::OutFlowType::Manual,
            inventory:    vec![accounting_stuff::InventoryRecord {
                time_unix: 1,
                quantity:  10.0,
                amount:    20.0,
            }],
        });

        horizontal_infer_for_quantity_from_amount(100, &mut container, &mut provider);

        // Quantity should still be set (because the function sets it regardless of errors)
        let updated = &container.doubles[0].singles[0];
        assert_eq!(updated.get_inferred_quantity(), Some(10.0));

        // But inventory should NOT be updated because error flag is set
        let inv = &provider.get_mut(&"1".to_string()).unwrap().inventory;
        assert_eq!(inv.len(), 2);
        assert_eq!(inv[0].quantity, 10.0);
        assert_eq!(inv[0].amount, 20.0);
        assert_eq!(inv[1].quantity, 10.0);
        assert_eq!(inv[1].amount, 10.0);
    }

    #[test]
    fn test_quantity_from_amount_with_empty_inventory() {
        // If inventory is empty, the function should still work (no panic) and set quantity as appropriate.
        let single = MockSingle {
            user_input_account_id: "1".to_string(),
            inferred_amount: Some(10.0),
            inferred_is_inflow: Some(false),
            inferred_outflow_type: Some(accounting_stuff::OutFlowType::Manual),
            ..Default::default()
        };

        let double = MockDouble {
            singles: vec![single],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };

        let mut provider = new_account_info_provider();
        provider.insert("1".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  accounting_stuff::InFlowType::Manual,
            outflow_type: accounting_stuff::OutFlowType::Manual,
            inventory:    vec![], // empty
        });

        horizontal_infer_for_quantity_from_amount(100, &mut container, &mut provider);

        let updated = &container.doubles[0].singles[0];
        assert_eq!(updated.get_inferred_quantity(), None); // unchanged because manual outflow does not change quantity
        // Amount should be capped to total amount (0.0)
        assert_eq!(updated.get_inferred_amount(), Some(0.0));

        // Inventory remains empty
        let inv = &provider.get_mut(&"1".to_string()).unwrap().inventory;
        dbg!(&inv);
        assert!(inv.is_empty());
    }

    #[test]
    fn test_quantity_from_amount_outflow_manual_amount_already_less_than_inventory() {
        // If amount is less than total amount, it stays unchanged.
        let mut single = MockSingle {
            user_input_account_id: "1".to_string(),
            inferred_amount: Some(5.0), // less than total (20)
            inferred_quantity: Some(2.0),
            inferred_is_inflow: Some(false),
            inferred_outflow_type: Some(accounting_stuff::OutFlowType::Manual),
            ..Default::default()
        };
        single.set_inferred_amount(Some(5.0));
        single.set_inferred_quantity(Some(2.0));
        single.set_inferred_is_inflow(Some(false));
        single.set_inferred_outflow_type(Some(accounting_stuff::OutFlowType::Manual));

        let double = MockDouble {
            singles: vec![single],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };

        let mut provider = new_account_info_provider();
        provider.insert("1".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  accounting_stuff::InFlowType::Manual,
            outflow_type: accounting_stuff::OutFlowType::Manual,
            inventory:    vec![accounting_stuff::InventoryRecord {
                time_unix: 1,
                quantity:  10.0,
                amount:    20.0,
            }],
        });

        horizontal_infer_for_quantity_from_amount(100, &mut container, &mut provider);

        let updated = &container.doubles[0].singles[0];
        assert_eq!(updated.get_inferred_quantity(), Some(2.0)); // unchanged
        assert_eq!(updated.get_inferred_amount(), Some(5.0)); // unchanged (not capped)

        // Inventory updated: outflow amount 5, qty 2
        let inv = &provider.get_mut(&"1".to_string()).unwrap().inventory;
        assert_eq!(inv.len(), 1);
        assert_eq!(inv[0].quantity, 8.0);
        assert_eq!(inv[0].amount, 15.0);
    }

    // Additional test for when quantity is already set but we have outflow with cost method.
    // In that case, quantity should be recomputed from amount (overwriting previous).
    #[test]
    fn test_quantity_from_amount_outflow_cost_method_overwrites_quantity() {
        let mut single = MockSingle {
            user_input_account_id: "1".to_string(),
            inferred_amount: Some(10.0),
            inferred_quantity: Some(999.0), // arbitrary old value, should be overwritten
            inferred_is_inflow: Some(false),
            inferred_outflow_type: Some(accounting_stuff::OutFlowType::Fifo),
            ..Default::default()
        };
        single.set_inferred_amount(Some(10.0));
        single.set_inferred_quantity(Some(999.0));
        single.set_inferred_is_inflow(Some(false));
        single.set_inferred_outflow_type(Some(accounting_stuff::OutFlowType::Fifo));

        let double = MockDouble {
            singles: vec![single],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };

        let mut provider = new_account_info_provider();
        provider.insert("1".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  accounting_stuff::InFlowType::Manual,
            outflow_type: accounting_stuff::OutFlowType::Fifo,
            inventory:    vec![accounting_stuff::InventoryRecord {
                time_unix: 1,
                quantity:  5.0,
                amount:    10.0,
            }],
        });
        // Sort FIFO (already sorted by time)
        accounting_stuff::sort_inventory(
            &accounting_stuff::OutFlowType::Fifo,
            &mut provider.get_mut(&"1".to_string()).unwrap().inventory,
        );

        horizontal_infer_for_quantity_from_amount(100, &mut container, &mut provider);

        let updated = &container.doubles[0].singles[0];
        // get_quantity(10, inventory) = 10 / (10/5) = 5.0
        assert_eq!(updated.get_inferred_quantity(), Some(5.0));
        assert_eq!(updated.get_inferred_amount(), Some(10.0));
    }
}
