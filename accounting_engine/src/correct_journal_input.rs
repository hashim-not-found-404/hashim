use crate::accounting_stuff::DoubleEntry;
use crate::accounting_stuff::EntryContainer;
use crate::accounting_stuff::InFlowType;
use crate::accounting_stuff::Inventory;
use crate::accounting_stuff::OutFlowType;
use crate::accounting_stuff::apply_entry_on_inventory;
use crate::accounting_stuff::get_amount;
use crate::accounting_stuff::get_quantity;
use crate::accounting_stuff::is_debit;
use crate::accounting_stuff::is_decrease_by_price;
use crate::accounting_stuff::is_inflow;
use crate::accounting_stuff::sort_inventory;
use crate::common_subset_sum::split_to_max;
use crate::constrained_partition::Side;
use crate::constrained_partition::assign_partition;
use crate::number_type::Num;
use std::collections::HashSet;
use std::hash::Hash;

pub trait SingleEntry {
    type AccountId: Eq + Hash;

    fn get_account_id(&self) -> Self::AccountId;

    fn get_from_user_input_is_debit(&self) -> Option<bool>;
    fn get_from_user_input_is_inflow(&self) -> Option<bool>;
    fn get_from_user_input_quantity(&self) -> Option<f64>;
    fn get_from_user_input_amount(&self) -> Option<f64>;
    fn get_from_user_input_inflow_type(&self) -> Option<InFlowType>;
    fn get_from_user_input_outflow_type(&self) -> Option<OutFlowType>;

    fn set_user_input_is_debit(&mut self, i: Option<bool>);
    fn set_user_input_is_inflow(&mut self, i: Option<bool>);
    fn set_user_input_quantity(&mut self, i: Option<f64>);
    fn set_user_input_amount(&mut self, i: Option<f64>);
    fn set_user_input_inflow_type(&mut self, i: Option<InFlowType>);
    fn set_user_input_outflow_type(&mut self, i: Option<OutFlowType>);

    fn set_inferred_is_debit(&mut self, i: Option<bool>);
    fn set_inferred_is_inflow(&mut self, i: Option<bool>);
    fn set_inferred_quantity(&mut self, i: Option<f64>);
    fn set_inferred_amount(&mut self, i: Option<f64>);
    fn set_inferred_inflow_type(&mut self, i: Option<InFlowType>);
    fn set_inferred_outflow_type(&mut self, i: Option<OutFlowType>);

    fn get_inferred_is_debit(&self) -> Option<bool>;
    fn get_inferred_is_inflow(&self) -> Option<bool>;
    fn get_inferred_quantity(&self) -> Option<f64>;
    fn get_inferred_amount(&self) -> Option<f64>;
    fn get_inferred_inflow_type(&self) -> Option<InFlowType>;
    fn get_inferred_outflow_type(&self) -> Option<OutFlowType>;
}

pub trait AccountInfoProvider {
    type AccountId: Eq + Hash;
    type Inventory: Inventory;

    fn get_info<'a>(&'a self, id: &Self::AccountId) -> Option<AccountInfo<&'a Self::Inventory>>;

    fn get_info_mut<'a>(
        &'a mut self,
        id: &Self::AccountId,
    ) -> Option<AccountInfo<&'a mut Self::Inventory>>;
}

pub struct AccountInfo<I> {
    pub is_debit:      bool,
    pub in_flow_type:  InFlowType,
    pub out_flow_type: OutFlowType,
    pub inventory:     I,
}

fn reset_all_inferred_values<C, AId>(entry: &mut C)
where
    C: EntryContainer,
    for<'a> C::Double<'a>: DoubleEntry,
    for<'a> <C::Double<'a> as DoubleEntry>::Single: SingleEntry<AccountId = AId>,
{
    for double in entry.iter_mut() {
        for single in double.iter_mut() {
            single.set_inferred_is_debit(single.get_from_user_input_is_debit());
            single.set_inferred_is_inflow(single.get_from_user_input_is_inflow());
            single.set_inferred_quantity(single.get_from_user_input_quantity().map(f64::abs));
            single.set_inferred_amount(single.get_from_user_input_amount().map(f64::abs));
            single.set_inferred_inflow_type(single.get_from_user_input_inflow_type());
            single.set_inferred_outflow_type(single.get_from_user_input_outflow_type());
        }
    }
}

fn horizontal_infer_for_is_debit<C, A, AId>(entry: &mut C, account_info: &A)
where
    C: EntryContainer,
    for<'a> C::Double<'a>: DoubleEntry,
    for<'a> <C::Double<'a> as DoubleEntry>::Single: SingleEntry<AccountId = AId>,
    A: AccountInfoProvider<AccountId = AId>,
    A::Inventory: Inventory,
{
    for double in entry.iter_mut() {
        for single in double.iter_mut() {
            if single.get_inferred_is_debit().is_none()
                && let Some(is_inflow) = single.get_inferred_is_inflow()
            {
                let account_id = single.get_account_id();
                if let Some(info) = account_info.get_info(&account_id) {
                    single.set_inferred_is_debit(Some(is_debit(info.is_debit, is_inflow)));
                }
            }
        }
    }
}

fn horizontal_infer_for_is_inflow<C, A, AId>(entry: &mut C, account_info: &A)
where
    C: EntryContainer,
    for<'a> C::Double<'a>: DoubleEntry,
    for<'a> <C::Double<'a> as DoubleEntry>::Single: SingleEntry<AccountId = AId>,
    A: AccountInfoProvider<AccountId = AId>,
    A::Inventory: Inventory,
{
    for double in entry.iter_mut() {
        for single in double.iter_mut() {
            if let Some(is_debit) = single.get_inferred_is_debit() {
                let account_id = single.get_account_id();
                if let Some(info) = account_info.get_info(&account_id) {
                    single.set_inferred_is_inflow(Some(is_inflow(info.is_debit, is_debit)));
                }
            }
        }
    }
}

fn horizontal_infer_for_inflow_type<C, A, AId>(entry: &mut C, account_info: &A)
where
    C: EntryContainer,
    for<'a> C::Double<'a>: DoubleEntry,
    for<'a> <C::Double<'a> as DoubleEntry>::Single: SingleEntry<AccountId = AId>,
    A: AccountInfoProvider<AccountId = AId>,
    A::Inventory: Inventory,
{
    for double in entry.iter_mut() {
        for single in double.iter_mut() {
            if let Some(inflow_type_from_user) = single.get_inferred_inflow_type() {
                single.set_inferred_inflow_type(Some(inflow_type_from_user));
            } else {
                let account_id = single.get_account_id();

                if let Some(info) = account_info.get_info(&account_id) {
                    single.set_inferred_inflow_type(Some(info.in_flow_type));
                }
            }
        }
    }
}

fn horizontal_infer_for_outflow_type<C, A, AId>(entry: &mut C, account_info: &A)
where
    C: EntryContainer,
    for<'a> C::Double<'a>: DoubleEntry,
    for<'a> <C::Double<'a> as DoubleEntry>::Single: SingleEntry<AccountId = AId>,
    A: AccountInfoProvider<AccountId = AId>,
    A::Inventory: Inventory,
{
    for double in entry.iter_mut() {
        for single in double.iter_mut() {
            if let Some(outflow_type_from_user) = single.get_inferred_outflow_type() {
                single.set_inferred_outflow_type(Some(outflow_type_from_user));
            } else {
                let account_id = single.get_account_id();

                if let Some(info) = account_info.get_info(&account_id) {
                    single.set_inferred_outflow_type(Some(info.out_flow_type));
                }
            }
        }
    }
}

fn vertical_correct_by_remove_duplicate_account<C>(entry: &mut C)
where
    C: EntryContainer,
    for<'a> C::Double<'a>: DoubleEntry,
    for<'a> <C::Double<'a> as DoubleEntry>::Single: SingleEntry,
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

fn horizontal_infer_for_amount_from_quantity<C, A, AId>(
    time_unix: u64,
    entry: &mut C,
    mut account_info: A,
) where
    C: EntryContainer,
    for<'a> C::Double<'a>: DoubleEntry,
    for<'a> <C::Double<'a> as DoubleEntry>::Single: SingleEntry<AccountId = AId>,
    A: AccountInfoProvider<AccountId = AId>,
    A::Inventory: Inventory,
{
    for double in entry.iter_mut() {
        for single in double.iter_mut() {
            let Some(mut inferred_quantity) = single.get_inferred_quantity() else {
                continue;
            };

            let Some(is_inflow) = single.get_inferred_is_inflow() else {
                continue;
            };

            let account_id = single.get_account_id();

            let Some(info) = account_info.get_info_mut(&account_id) else {
                continue;
            };

            if is_inflow {
                let Some(inferred_inflow_type) = single.get_inferred_inflow_type() else {
                    continue;
                };

                match inferred_inflow_type {
                    InFlowType::Manual => {}
                    InFlowType::QuantityEqualAmount => {
                        single.set_inferred_amount(Some(inferred_quantity));
                    }
                    InFlowType::QuantityEqualZero => single.set_inferred_quantity(Some(0.0)),
                }
            } else {
                let Some(inferred_outflow_type) = single.get_inferred_outflow_type() else {
                    continue;
                };

                let total_quantity_in_inventory =
                    info.inventory.iter1().fold(0.0, |total, record| total + record.quantity);

                inferred_quantity = total_quantity_in_inventory.min(inferred_quantity);

                single.set_inferred_quantity(Some(inferred_quantity));

                sort_inventory(inferred_outflow_type, info.inventory);

                match inferred_outflow_type {
                    OutFlowType::Manual => {
                        if let Some(mut inferred_amount) = single.get_inferred_amount() {
                            let total_amount_in_inventory = info
                                .inventory
                                .iter1()
                                .fold(0.0, |total, record| total + record.amount);

                            inferred_amount = total_amount_in_inventory.min(inferred_amount);

                            single.set_inferred_amount(Some(inferred_amount));
                        }
                    }
                    OutFlowType::QuantityEqualAmount => {
                        let total_amount_in_inventory =
                            info.inventory.iter1().fold(0.0, |total, record| total + record.amount);

                        let inferred_amount = total_amount_in_inventory.min(inferred_quantity);

                        single.set_inferred_quantity(Some(inferred_amount));
                        single.set_inferred_amount(Some(inferred_amount));
                    }
                    OutFlowType::QuantityEqualZero => {
                        single.set_inferred_quantity(Some(0.0));

                        if let Some(mut inferred_amount) = single.get_inferred_amount() {
                            let total_amount_in_inventory = info
                                .inventory
                                .iter1()
                                .fold(0.0, |total, record| total + record.amount);

                            inferred_amount = total_amount_in_inventory.min(inferred_amount);

                            single.set_inferred_amount(Some(inferred_amount));
                        }
                    }
                    OutFlowType::Wac
                    | OutFlowType::Fifo
                    | OutFlowType::Lifo
                    | OutFlowType::Hifo
                    | OutFlowType::Lofo => {
                        let expected_amount = get_amount(inferred_quantity, info.inventory);

                        single.set_inferred_amount(Some(expected_amount));
                    }
                }
            }

            if let Some(amount) = single.get_inferred_amount()
                && let Some(quantity) = single.get_inferred_quantity()
            {
                let is_decrease_by_price = match single.get_inferred_outflow_type() {
                    Some(OutFlowType::Manual) => false,
                    Some(OutFlowType::QuantityEqualAmount) => false,
                    Some(OutFlowType::QuantityEqualZero) => false,
                    Some(OutFlowType::Wac) => true,
                    Some(OutFlowType::Fifo) => true,
                    Some(OutFlowType::Lifo) => true,
                    Some(OutFlowType::Hifo) => true,
                    Some(OutFlowType::Lofo) => true,
                    None => false,
                };

                apply_entry_on_inventory::<A::Inventory>(
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
    for<'a> C::Double<'a>: DoubleEntry,
    for<'a> <C::Double<'a> as DoubleEntry>::Single: SingleEntry + Clone,
{
    for double in entry.iter_mut() {
        let mut new_double = Vec::new();
        let mut other_double = Vec::new();

        for single in double.iter_ref() {
            if single.get_inferred_amount().is_some() {
                new_double.push(single.clone());
            } else {
                other_double.push(single.clone());
            }
        }

        assign_partition(
            &mut new_double,
            |single| Num(single.get_inferred_amount().unwrap_or_default()),
            |single| {
                single.get_inferred_is_debit().map_or(Side::Unknown, |is_debit| {
                    if is_debit {
                        Side::Rhs
                    } else {
                        Side::Lhs
                    }
                })
            },
            |single, b| single.set_inferred_is_debit(Some(b == Side::Rhs)),
        );

        new_double.append(&mut other_double);
        double.set_singles(new_double);
    }
}

fn vertical_infer_for_amount<C>(entry: &mut C)
where
    C: EntryContainer,
    for<'a> C::Double<'a>: DoubleEntry,
    for<'a> <C::Double<'a> as DoubleEntry>::Single: SingleEntry,
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
            }
        }

        if total_debit == total_credit {
            continue;
        }

        let diff = (total_debit - total_credit).abs();

        let Some(the_idx) = idx_for_not_inferred_amount else {
            continue;
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
    C: EntryContainer + Clone,
    for<'a> C::Double<'a>: DoubleEntry + Clone,
    for<'a> <C::Double<'a> as DoubleEntry>::Single: SingleEntry + Clone,
{
    let mut new_doubles = Vec::new();

    for double in entry.clone().iter() {
        let mut debit_side = Vec::new();
        let mut credit_side = Vec::new();

        for single in double.clone().into_iter() {
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

        let groups = split_to_max(&debit_side, &credit_side, &|s| {
            Num(s.get_inferred_amount().unwrap_or_default())
        });

        for (debit_group, credit_group) in groups {
            let mut combined = Vec::new();
            combined.extend(debit_group);
            combined.extend(credit_group);

            let mut new_double: <C as EntryContainer>::Double<'_> = double.clone();
            new_double.set_singles(combined);
            new_doubles.push(new_double);
        }
    }

    entry.set_doubles(new_doubles);
}

fn horizontal_correct<C>(entry: &mut C)
where
    C: EntryContainer,
    for<'a> C::Double<'a>: DoubleEntry,
    for<'a> <C::Double<'a> as DoubleEntry>::Single: SingleEntry,
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

fn horizontal_infer_for_quantity_from_amount<C, A, AId>(
    time_unix: u64,
    entry: &mut C,
    mut account_info: A,
) where
    C: EntryContainer,
    for<'a> C::Double<'a>: DoubleEntry,
    for<'a> <C::Double<'a> as DoubleEntry>::Single: SingleEntry<AccountId = AId>,
    A: AccountInfoProvider<AccountId = AId>,
    A::Inventory: Inventory,
{
    for double in entry.iter_mut() {
        for single in double.iter_mut() {
            let Some(amount) = single.get_inferred_amount() else {
                continue;
            };

            let account_id = single.get_account_id();

            let Some(info) = account_info.get_info_mut(&account_id) else {
                continue;
            };

            let Some(is_inflow) = single.get_inferred_is_inflow() else {
                continue;
            };

            if is_inflow {
                let Some(inflow_type) = single.get_inferred_inflow_type() else {
                    continue;
                };

                match inflow_type {
                    InFlowType::Manual => {}
                    InFlowType::QuantityEqualAmount => single.set_inferred_quantity(Some(amount)),
                    InFlowType::QuantityEqualZero => single.set_inferred_quantity(Some(0.0)),
                }
            } else {
                let Some(outflow_type) = single.get_inferred_outflow_type() else {
                    continue;
                };

                let total_amount_in_inventory =
                    info.inventory.iter1().fold(0.0, |total, record| total + record.amount);

                let inferred_amount = total_amount_in_inventory.min(amount);

                single.set_inferred_amount(Some(inferred_amount));

                sort_inventory(outflow_type, info.inventory);

                match outflow_type {
                    OutFlowType::Manual => {}
                    OutFlowType::QuantityEqualAmount => {
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
                    OutFlowType::QuantityEqualZero => single.set_inferred_quantity(Some(0.0)),
                    OutFlowType::Wac
                    | OutFlowType::Fifo
                    | OutFlowType::Lifo
                    | OutFlowType::Hifo
                    | OutFlowType::Lofo => {
                        let quantity = get_quantity(amount, info.inventory);

                        single.set_inferred_quantity(Some(quantity));
                    }
                }
            }

            if let Some(amount) = single.get_inferred_amount()
                && let Some(quantity) = single.get_inferred_quantity()
            {
                let is_decrease_by_price = match single.get_inferred_outflow_type() {
                    Some(out_flow_type) => is_decrease_by_price(out_flow_type),
                    None => false,
                };

                apply_entry_on_inventory::<A::Inventory>(
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

pub fn correct_the_input<C, A, AId>(time_unix: u64, entry: &mut C, account_info: &A)
where
    C: EntryContainer + Clone,
    for<'a> C::Double<'a>: DoubleEntry + Clone,
    for<'a> <C::Double<'a> as DoubleEntry>::Single: SingleEntry<AccountId = AId> + Clone,
    A: AccountInfoProvider<AccountId = AId> + Clone,
    A::Inventory: Inventory,
{
    reset_all_inferred_values(entry);
    vertical_correct_to_remove_empty_double_entry(entry);
    vertical_correct_by_remove_duplicate_account(entry);
    horizontal_infer_for_is_debit(entry, account_info);
    horizontal_infer_for_is_inflow(entry, account_info);
    horizontal_infer_for_inflow_type(entry, account_info);
    horizontal_infer_for_outflow_type(entry, account_info);
    horizontal_infer_for_amount_from_quantity(time_unix, entry, account_info.clone());

    vertical_infer_for_is_debit(entry);
    horizontal_infer_for_is_inflow(entry, account_info);
    vertical_infer_for_amount(entry);
    horizontal_infer_for_quantity_from_amount(time_unix, entry, account_info.clone());

    vertical_correct_by_common_subset_sum(entry);
    horizontal_correct(entry);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounting_stuff::InventoryRecord;
    use std::collections::HashMap;

    impl Inventory for Vec<InventoryRecord> {
        fn push(&mut self, record: InventoryRecord) {
            self.push(record);
        }

        fn clear(&mut self) {
            self.clear();
        }

        fn is_empty(&self) -> bool {
            self.is_empty()
        }

        fn iter1(&self) -> impl Iterator<Item = &InventoryRecord> {
            self.iter()
        }

        fn iter_mut1(&mut self) -> impl Iterator<Item = &mut InventoryRecord> {
            self.iter_mut()
        }

        fn sort_by1<F>(&mut self, compare: F)
        where
            F: FnMut(&InventoryRecord, &InventoryRecord) -> std::cmp::Ordering,
        {
            self.sort_by(compare);
        }

        fn retain<F>(&mut self, _f: F)
        where
            F: FnMut(&InventoryRecord) -> bool,
        {
        }

        fn pop(&mut self) -> Option<InventoryRecord> {
            None
        }
    }

    #[derive(Default, Debug, Clone)]
    pub struct MockSingle {
        pub user_input_account_id: String,

        pub user_input_is_debit:     Option<bool>,
        pub user_input_is_inflow:    Option<bool>,
        pub user_input_quantity:     Option<f64>,
        pub user_input_amount:       Option<f64>,
        pub user_input_inflow_type:  Option<InFlowType>,
        pub user_input_outflow_type: Option<OutFlowType>,

        pub inferred_is_debit:     Option<bool>,
        pub inferred_is_inflow:    Option<bool>,
        pub inferred_quantity:     Option<f64>,
        pub inferred_amount:       Option<f64>,
        pub inferred_inflow_type:  Option<InFlowType>,
        pub inferred_outflow_type: Option<OutFlowType>,
    }

    impl SingleEntry for MockSingle {
        type AccountId = String;

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

        fn get_from_user_input_inflow_type(&self) -> Option<InFlowType> {
            self.user_input_inflow_type.clone()
        }

        fn get_from_user_input_outflow_type(&self) -> Option<OutFlowType> {
            self.user_input_outflow_type.clone()
        }

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

        fn set_user_input_inflow_type(&mut self, i: Option<InFlowType>) {
            self.user_input_inflow_type = i;
        }

        fn set_user_input_outflow_type(&mut self, i: Option<OutFlowType>) {
            self.user_input_outflow_type = i;
        }

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

        fn set_inferred_inflow_type(&mut self, i: Option<InFlowType>) {
            self.inferred_inflow_type = i;
        }

        fn set_inferred_outflow_type(&mut self, i: Option<OutFlowType>) {
            self.inferred_outflow_type = i;
        }

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

        fn get_inferred_inflow_type(&self) -> Option<InFlowType> {
            self.inferred_inflow_type.clone()
        }

        fn get_inferred_outflow_type(&self) -> Option<OutFlowType> {
            self.inferred_outflow_type.clone()
        }
    }

    #[derive(Debug, Clone)]
    pub struct MockDouble {
        pub singles: Vec<MockSingle>,
    }

    impl DoubleEntry for MockDouble {
        type Iter<'a> = std::vec::IntoIter<MockSingle>;
        type IterMut<'a> = std::slice::IterMut<'a, MockSingle>;
        type IterRef<'a> = std::slice::Iter<'a, MockSingle>;
        type Single = MockSingle;

        fn into_iter<'a>(self) -> Self::Iter<'a> {
            self.singles.into_iter()
        }

        fn iter_ref(&self) -> Self::IterRef<'_> {
            self.singles.iter()
        }

        fn iter_mut(&mut self) -> Self::IterMut<'_> {
            self.singles.iter_mut()
        }

        fn set_singles(&mut self, singles: Vec<Self::Single>) {
            self.singles = singles;
        }

        fn is_empty(&self) -> bool {
            self.singles.is_empty()
        }

        fn len(&self) -> usize {
            self.singles.len()
        }

        fn retain<F>(&mut self, f: F)
        where
            F: FnMut(&Self::Single) -> bool,
        {
            self.singles.retain(f);
        }
    }

    #[derive(Debug, Clone)]
    pub struct MockEntryContainer {
        pub doubles: Vec<MockDouble>,
    }

    impl EntryContainer for MockEntryContainer {
        type Double<'a> = MockDouble;
        type Iter<'a> = std::vec::IntoIter<MockDouble>;
        type IterMut<'a> = std::slice::IterMut<'a, MockDouble>;
        type IterRef<'a> = std::slice::Iter<'a, MockDouble>;

        fn iter<'a>(self) -> Self::Iter<'a> {
            self.doubles.into_iter()
        }

        fn iter_ref(&self) -> Self::IterRef<'_> {
            self.doubles.iter()
        }

        fn iter_mut(&mut self) -> Self::IterMut<'_> {
            self.doubles.iter_mut()
        }

        fn set_doubles(&mut self, doubles: Vec<Self::Double<'_>>) {
            self.doubles = doubles;
        }

        fn is_empty(&self) -> bool {
            self.doubles.is_empty()
        }

        fn len(&self) -> usize {
            self.doubles.len()
        }

        fn retain<F>(&mut self, f: F)
        where
            F: FnMut(&Self::Double<'_>) -> bool,
        {
            self.doubles.retain(f);
        }
    }

    struct MockAccountInfoProvider {
        is_debit:     bool,
        inflow_type:  InFlowType,
        outflow_type: OutFlowType,
        inventory:    Vec<InventoryRecord>,
    }

    fn new_account_info_provider() -> HashMap<String, MockAccountInfoProvider> {
        HashMap::new()
    }

    impl AccountInfoProvider for HashMap<String, MockAccountInfoProvider> {
        type AccountId = String;
        type Inventory = Vec<InventoryRecord>;

        fn get_info<'a>(
            &'a self,
            id: &Self::AccountId,
        ) -> Option<AccountInfo<&'a Self::Inventory>> {
            self.get(id).map(|a| {
                AccountInfo {
                    is_debit:      a.is_debit,
                    in_flow_type:  a.inflow_type.clone(),
                    out_flow_type: a.outflow_type.clone(),
                    inventory:     &a.inventory,
                }
            })
        }

        fn get_info_mut<'a>(
            &'a mut self,
            id: &Self::AccountId,
        ) -> Option<AccountInfo<&'a mut Self::Inventory>> {
            self.get_mut(id).map(|a| {
                AccountInfo {
                    is_debit:      a.is_debit,
                    in_flow_type:  a.inflow_type.clone(),
                    out_flow_type: a.outflow_type.clone(),
                    inventory:     &mut a.inventory,
                }
            })
        }
    }

    #[test]
    fn full_pipeline_infers_and_corrects_inflow() {
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

        let mut provider = new_account_info_provider();
        provider.insert("1".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  InFlowType::Manual,
            outflow_type: OutFlowType::Manual,
            inventory:    Vec::new(),
        });
        provider.insert("2".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  InFlowType::Manual,
            outflow_type: OutFlowType::Manual,
            inventory:    Vec::new(),
        });

        reset_all_inferred_values(&mut container);
        horizontal_infer_for_is_debit(&mut container, &mut provider);
        horizontal_infer_for_is_inflow(&mut container, &mut provider);
        horizontal_correct(&mut container);

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
        let single1 = MockSingle {
            user_input_account_id: "1".to_string(),
            user_input_inflow_type: Some(InFlowType::Manual),
            ..Default::default()
        };

        let single2 = MockSingle {
            user_input_account_id: "2".to_string(),
            user_input_inflow_type: Some(InFlowType::QuantityEqualAmount),
            ..Default::default()
        };

        let single3 = MockSingle {
            user_input_account_id: "3".to_string(),
            user_input_inflow_type: None,
            ..Default::default()
        };

        let single4 = MockSingle {
            user_input_account_id: "4".to_string(),
            user_input_inflow_type: None,
            ..Default::default()
        };

        let single5 = MockSingle {
            user_input_account_id: "4".to_string(),
            user_input_inflow_type: Some(InFlowType::Manual),
            ..Default::default()
        };

        let double = MockDouble {
            singles: vec![single1, single2, single3, single4, single5],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };

        let mut provider = new_account_info_provider();
        provider.insert("1".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  InFlowType::QuantityEqualZero,
            outflow_type: OutFlowType::Manual,
            inventory:    Vec::new(),
        });
        provider.insert("2".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  InFlowType::QuantityEqualAmount,
            outflow_type: OutFlowType::Manual,
            inventory:    Vec::new(),
        });
        provider.insert("3".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  InFlowType::QuantityEqualZero,
            outflow_type: OutFlowType::Manual,
            inventory:    Vec::new(),
        });

        reset_all_inferred_values(&mut container);
        horizontal_infer_for_inflow_type(&mut container, &mut provider);

        let updated_double = &container.doubles[0];

        assert_eq!(updated_double.singles[0].get_inferred_inflow_type(), Some(InFlowType::Manual));
        assert_eq!(
            updated_double.singles[1].get_inferred_inflow_type(),
            Some(InFlowType::QuantityEqualAmount)
        );
        assert_eq!(
            updated_double.singles[2].get_inferred_inflow_type(),
            Some(InFlowType::QuantityEqualZero)
        );
        assert_eq!(updated_double.singles[3].get_inferred_inflow_type(), None);
        assert_eq!(updated_double.singles[4].get_inferred_inflow_type(), Some(InFlowType::Manual));
    }

    #[test]
    fn horizontal_infer_for_outflow_type_sets_inferred_outflow_type() {
        let single1 = MockSingle {
            user_input_account_id: "1".to_string(),
            user_input_outflow_type: Some(OutFlowType::Manual),
            ..Default::default()
        };

        let single2 = MockSingle {
            user_input_account_id: "2".to_string(),
            user_input_outflow_type: Some(OutFlowType::QuantityEqualAmount),
            ..Default::default()
        };

        let single3 = MockSingle {
            user_input_account_id: "3".to_string(),
            user_input_outflow_type: None,
            ..Default::default()
        };

        let single4 = MockSingle {
            user_input_account_id: "4".to_string(),
            user_input_outflow_type: None,
            ..Default::default()
        };

        let single5 = MockSingle {
            user_input_account_id: "4".to_string(),
            user_input_outflow_type: Some(OutFlowType::Manual),
            ..Default::default()
        };

        let double = MockDouble {
            singles: vec![single1, single2, single3, single4, single5],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };

        let mut provider = new_account_info_provider();
        provider.insert("1".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  InFlowType::QuantityEqualZero,
            outflow_type: OutFlowType::Manual,
            inventory:    Vec::new(),
        });
        provider.insert("2".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  InFlowType::QuantityEqualAmount,
            outflow_type: OutFlowType::Manual,
            inventory:    Vec::new(),
        });
        provider.insert("3".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  InFlowType::QuantityEqualZero,
            outflow_type: OutFlowType::QuantityEqualZero,
            inventory:    Vec::new(),
        });

        reset_all_inferred_values(&mut container);
        horizontal_infer_for_outflow_type(&mut container, &mut provider);

        let updated_double = &container.doubles[0];

        assert_eq!(
            updated_double.singles[0].get_inferred_outflow_type(),
            Some(OutFlowType::Manual)
        );
        assert_eq!(
            updated_double.singles[1].get_inferred_outflow_type(),
            Some(OutFlowType::QuantityEqualAmount)
        );
        assert_eq!(
            updated_double.singles[2].get_inferred_outflow_type(),
            Some(OutFlowType::QuantityEqualZero)
        );
        assert_eq!(updated_double.singles[3].get_inferred_outflow_type(), None);
        assert_eq!(
            updated_double.singles[4].get_inferred_outflow_type(),
            Some(OutFlowType::Manual)
        );
    }

    #[test]
    fn vertical_correct_to_remove_duplicate_account_removes_duplicates() {
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

        vertical_correct_by_remove_duplicate_account(&mut container);

        let updated_double = &container.doubles[0];
        let remaining = &updated_double.singles;

        assert_eq!(remaining.len(), 3);
        assert_eq!(remaining[0].user_input_account_id, "A");
        assert_eq!(remaining[0].user_input_is_debit, Some(true));

        assert_eq!(remaining[1].user_input_account_id, "B");
        assert_eq!(remaining[1].user_input_is_debit, Some(true));

        assert_eq!(remaining[2].user_input_account_id, "C");
        assert_eq!(remaining[2].user_input_is_debit, Some(true));
    }

    #[test]
    fn horizontal_correct_for_quantity_and_amount_inflow_manual() {
        let single = MockSingle {
            user_input_account_id: "1".to_string(),
            user_input_quantity: Some(5.0),
            user_input_amount: Some(10.0),
            inferred_is_inflow: Some(true),
            inferred_inflow_type: Some(InFlowType::Manual),
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
            inflow_type:  InFlowType::Manual,
            outflow_type: OutFlowType::Manual,
            inventory:    Vec::new(),
        });

        horizontal_infer_for_amount_from_quantity(100, &mut container, provider);

        let updated = &container.doubles[0].singles[0];
        assert_eq!(updated.get_from_user_input_quantity(), Some(5.0));
        assert_eq!(updated.get_from_user_input_amount(), Some(10.0));
    }

    #[test]
    fn horizontal_correct_for_quantity_and_amount_inflow_quantity_equal_amount() {
        let single = MockSingle {
            user_input_account_id: "1".to_string(),
            user_input_quantity: Some(5.0),
            user_input_amount: Some(10.0),
            user_input_is_inflow: Some(true),
            user_input_inflow_type: Some(InFlowType::QuantityEqualAmount),
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
            inflow_type:  InFlowType::Manual,
            outflow_type: OutFlowType::Manual,
            inventory:    Vec::new(),
        });

        reset_all_inferred_values(&mut container);
        horizontal_infer_for_amount_from_quantity(100, &mut container, provider);

        let updated = &container.doubles[0].singles[0];
        assert_eq!(updated.get_inferred_quantity(), Some(5.0));
        assert_eq!(updated.get_inferred_amount(), Some(5.0));
    }

    #[test]
    fn horizontal_correct_for_quantity_and_amount_inflow_quantity_equal_zero() {
        let single = MockSingle {
            user_input_account_id: "1".to_string(),
            user_input_quantity: Some(5.0),
            user_input_amount: Some(10.0),
            user_input_is_inflow: Some(true),
            user_input_inflow_type: Some(InFlowType::QuantityEqualZero),
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
            inflow_type:  InFlowType::Manual,
            outflow_type: OutFlowType::Manual,
            inventory:    Vec::new(),
        });

        reset_all_inferred_values(&mut container);
        horizontal_infer_for_amount_from_quantity(100, &mut container, provider);

        let updated = &container.doubles[0].singles[0];
        assert_eq!(updated.get_inferred_quantity(), Some(0.0));
        assert_eq!(updated.get_inferred_amount(), Some(10.0));
    }

    #[test]
    fn horizontal_correct_for_quantity_and_amount_outflow_manual() {
        let single = MockSingle {
            user_input_account_id: "1".to_string(),
            user_input_quantity: Some(10.0),
            user_input_amount: Some(100.0),
            user_input_is_inflow: Some(false),
            user_input_outflow_type: Some(OutFlowType::Manual),
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
            inflow_type:  InFlowType::Manual,
            outflow_type: OutFlowType::Manual,
            inventory:    vec![
                InventoryRecord {
                    time_unix: 1,
                    quantity:  2.0,
                    amount:    20.0,
                },
                InventoryRecord {
                    time_unix: 2,
                    quantity:  3.0,
                    amount:    30.0,
                },
            ],
        });

        reset_all_inferred_values(&mut container);
        horizontal_infer_for_amount_from_quantity(100, &mut container, provider);

        let updated = &container.doubles[0].singles[0];
        assert_eq!(updated.get_inferred_quantity(), Some(5.0));
        assert_eq!(updated.get_inferred_amount(), Some(50.0));
    }

    #[test]
    fn vertical_correct_to_common_subset_sum_splits_balanced_groups() {
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

        vertical_correct_by_common_subset_sum(&mut container);
        let updated_doubles = &container.doubles;
        assert_eq!(updated_doubles.len(), 2);
    }

    #[test]
    fn vertical_correct_to_common_subset_sum_splits_balanced_groups_with_uninferred_debit() {
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

        vertical_correct_by_common_subset_sum(&mut container);
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

        vertical_correct_to_remove_empty_double_entry(&mut container);
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

        vertical_infer_for_is_debit(&mut container);
        let updated_doubles = &container.doubles;
        assert_eq!(updated_doubles.len(), 1);
    }

    #[test]
    fn vertical_infer_for_is_debit_assigns_sides_to_balance_amounts() {
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
        vertical_infer_for_is_debit(&mut container);
        let updated_double = &container.doubles[0];
        for single in &updated_double.singles {
            assert!(
                single.inferred_is_debit.is_some(),
                "Entry {} has no inferred_is_debit",
                single.user_input_account_id
            );
        }
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

        vertical_infer_for_is_debit(&mut container);
        let updated_double = &container.doubles[0];
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
        vertical_infer_for_is_debit(&mut container);
        let updated_double = &container.doubles[0];
        for single in &updated_double.singles {
            assert!(
                single.inferred_is_debit.is_some(),
                "Entry {} has no inferred_is_debit",
                single.user_input_account_id
            );
        }
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
        let c2 = updated_double.singles.iter().find(|s| s.user_input_account_id == "C2").unwrap();
        assert_eq!(c2.inferred_amount, Some(25.0));
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

    #[test]
    fn test_quantity_from_amount_inflow_manual() {
        let mut single = MockSingle {
            user_input_account_id: "1".to_string(),
            inferred_amount: Some(10.0),
            inferred_quantity: Some(5.0),
            inferred_is_inflow: Some(true),
            inferred_inflow_type: Some(InFlowType::Manual),
            ..Default::default()
        };
        single.set_inferred_amount(Some(10.0));
        single.set_inferred_quantity(Some(5.0));
        single.set_inferred_is_inflow(Some(true));
        single.set_inferred_inflow_type(Some(InFlowType::Manual));

        let double = MockDouble {
            singles: vec![single],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };

        let mut provider = new_account_info_provider();
        provider.insert("1".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  InFlowType::Manual,
            outflow_type: OutFlowType::Manual,
            inventory:    vec![InventoryRecord {
                time_unix: 1,
                quantity:  10.0,
                amount:    20.0,
            }],
        });

        horizontal_infer_for_quantity_from_amount(100, &mut container, provider);

        let updated = &container.doubles[0].singles[0];
        assert_eq!(updated.get_inferred_quantity(), Some(5.0));
        assert_eq!(updated.get_inferred_amount(), Some(10.0));
    }

    #[test]
    fn test_quantity_from_amount_inflow_quantity_equal_amount() {
        let mut single = MockSingle {
            user_input_account_id: "1".to_string(),
            inferred_amount: Some(10.0),
            inferred_quantity: None,
            inferred_is_inflow: Some(true),
            inferred_inflow_type: Some(InFlowType::QuantityEqualAmount),
            ..Default::default()
        };
        single.set_inferred_amount(Some(10.0));
        single.set_inferred_is_inflow(Some(true));
        single.set_inferred_inflow_type(Some(InFlowType::QuantityEqualAmount));

        let double = MockDouble {
            singles: vec![single],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };

        let mut provider = new_account_info_provider();
        provider.insert("1".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  InFlowType::QuantityEqualAmount,
            outflow_type: OutFlowType::Manual,
            inventory:    vec![InventoryRecord {
                time_unix: 1,
                quantity:  10.0,
                amount:    20.0,
            }],
        });

        horizontal_infer_for_quantity_from_amount(100, &mut container, provider);

        let updated = &container.doubles[0].singles[0];
        assert_eq!(updated.get_inferred_quantity(), Some(10.0));
        assert_eq!(updated.get_inferred_amount(), Some(10.0));
    }

    #[test]
    fn test_quantity_from_amount_inflow_quantity_equal_zero() {
        let mut single = MockSingle {
            user_input_account_id: "1".to_string(),
            inferred_amount: Some(10.0),
            inferred_quantity: None,
            inferred_is_inflow: Some(true),
            inferred_inflow_type: Some(InFlowType::QuantityEqualZero),
            ..Default::default()
        };
        single.set_inferred_amount(Some(10.0));
        single.set_inferred_is_inflow(Some(true));
        single.set_inferred_inflow_type(Some(InFlowType::QuantityEqualZero));

        let double = MockDouble {
            singles: vec![single],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };

        let mut provider = new_account_info_provider();
        provider.insert("1".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  InFlowType::QuantityEqualZero,
            outflow_type: OutFlowType::Manual,
            inventory:    vec![InventoryRecord {
                time_unix: 1,
                quantity:  10.0,
                amount:    20.0,
            }],
        });

        horizontal_infer_for_quantity_from_amount(100, &mut container, provider);

        let updated = &container.doubles[0].singles[0];
        assert_eq!(updated.get_inferred_quantity(), Some(0.0));
        assert_eq!(updated.get_inferred_amount(), Some(10.0));
    }

    #[test]
    fn test_quantity_from_amount_outflow_manual() {
        let mut single = MockSingle {
            user_input_account_id: "1".to_string(),
            inferred_amount: Some(100.0),
            inferred_quantity: Some(5.0),
            inferred_is_inflow: Some(false),
            inferred_outflow_type: Some(OutFlowType::Manual),
            ..Default::default()
        };
        single.set_inferred_amount(Some(100.0));
        single.set_inferred_quantity(Some(5.0));
        single.set_inferred_is_inflow(Some(false));
        single.set_inferred_outflow_type(Some(OutFlowType::Manual));

        let double = MockDouble {
            singles: vec![single],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };

        let mut provider = new_account_info_provider();
        provider.insert("1".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  InFlowType::Manual,
            outflow_type: OutFlowType::Manual,
            inventory:    vec![InventoryRecord {
                time_unix: 1,
                quantity:  10.0,
                amount:    20.0,
            }],
        });

        horizontal_infer_for_quantity_from_amount(100, &mut container, provider);

        let updated = &container.doubles[0].singles[0];
        assert_eq!(updated.get_inferred_quantity(), Some(5.0));
        assert_eq!(updated.get_inferred_amount(), Some(20.0));
    }

    #[test]
    fn test_quantity_from_amount_outflow_quantity_equal_amount() {
        let mut single = MockSingle {
            user_input_account_id: "1".to_string(),
            inferred_amount: Some(100.0),
            inferred_quantity: None,
            inferred_is_inflow: Some(false),
            inferred_outflow_type: Some(OutFlowType::QuantityEqualAmount),
            ..Default::default()
        };
        single.set_inferred_amount(Some(100.0));
        single.set_inferred_is_inflow(Some(false));
        single.set_inferred_outflow_type(Some(OutFlowType::QuantityEqualAmount));

        let double = MockDouble {
            singles: vec![single],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };

        let mut provider = new_account_info_provider();
        provider.insert("1".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  InFlowType::Manual,
            outflow_type: OutFlowType::QuantityEqualAmount,
            inventory:    vec![InventoryRecord {
                time_unix: 1,
                quantity:  10.0,
                amount:    20.0,
            }],
        });

        horizontal_infer_for_quantity_from_amount(100, &mut container, provider);

        let updated = &container.doubles[0].singles[0];
        assert_eq!(updated.get_inferred_quantity(), Some(10.0));
        assert_eq!(updated.get_inferred_amount(), Some(10.0));
    }

    #[test]
    fn test_quantity_from_amount_outflow_quantity_equal_zero() {
        let mut single = MockSingle {
            user_input_account_id: "1".to_string(),
            inferred_amount: Some(100.0),
            inferred_quantity: None,
            inferred_is_inflow: Some(false),
            inferred_outflow_type: Some(OutFlowType::QuantityEqualZero),
            ..Default::default()
        };
        single.set_inferred_amount(Some(100.0));
        single.set_inferred_is_inflow(Some(false));
        single.set_inferred_outflow_type(Some(OutFlowType::QuantityEqualZero));

        let double = MockDouble {
            singles: vec![single],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };

        let mut provider = new_account_info_provider();
        provider.insert("1".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  InFlowType::Manual,
            outflow_type: OutFlowType::QuantityEqualZero,
            inventory:    vec![InventoryRecord {
                time_unix: 1,
                quantity:  10.0,
                amount:    20.0,
            }],
        });

        horizontal_infer_for_quantity_from_amount(100, &mut container, provider);

        let updated = &container.doubles[0].singles[0];
        assert_eq!(updated.get_inferred_quantity(), Some(0.0));
        assert_eq!(updated.get_inferred_amount(), Some(20.0));
    }

    #[test]
    fn test_quantity_from_amount_outflow_wac_fifo_lifo_hifo_lofo() {
        let flow_types = vec![
            OutFlowType::Wac,
            OutFlowType::Fifo,
            OutFlowType::Lifo,
            OutFlowType::Hifo,
            OutFlowType::Lofo,
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
            provider.insert("1".to_string(), MockAccountInfoProvider {
                is_debit:     true,
                inflow_type:  InFlowType::Manual,
                outflow_type: flow.clone(),
                inventory:    vec![
                    InventoryRecord {
                        time_unix: 1,
                        quantity:  6.0,
                        amount:    12.0,
                    },
                    InventoryRecord {
                        time_unix: 2,
                        quantity:  4.0,
                        amount:    8.0,
                    },
                ],
            });

            horizontal_infer_for_quantity_from_amount(100, &mut container, provider);

            let updated = &container.doubles[0].singles[0];
            assert_eq!(updated.get_inferred_amount(), Some(10.0));
        }
    }

    #[test]
    fn test_quantity_from_amount_missing_fields_skips() {
        let single = MockSingle {
            user_input_account_id: "1".to_string(),
            inferred_amount: None,
            inferred_is_inflow: Some(true),
            inferred_inflow_type: Some(InFlowType::Manual),
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
            inflow_type:  InFlowType::Manual,
            outflow_type: OutFlowType::Manual,
            inventory:    vec![],
        });
        horizontal_infer_for_quantity_from_amount(100, &mut container, provider);
        let updated = &container.doubles[0].singles[0];
        assert_eq!(updated.get_inferred_quantity(), None);
        assert_eq!(updated.get_inferred_amount(), None);

        let mut provider = new_account_info_provider();
        provider.insert("1".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  InFlowType::Manual,
            outflow_type: OutFlowType::Manual,
            inventory:    vec![],
        });
        let single2 = MockSingle {
            user_input_account_id: "1".to_string(),
            inferred_amount: Some(10.0),
            inferred_is_inflow: None,
            inferred_inflow_type: Some(InFlowType::Manual),
            ..Default::default()
        };
        let double2 = MockDouble {
            singles: vec![single2],
        };
        let mut container2 = MockEntryContainer {
            doubles: vec![double2],
        };
        horizontal_infer_for_quantity_from_amount(100, &mut container2, provider);
        let updated2 = &container2.doubles[0].singles[0];
        assert_eq!(updated2.get_inferred_quantity(), None);
    }

    #[test]
    fn test_quantity_from_amount() {
        let single = MockSingle {
            user_input_account_id: "1".to_string(),
            inferred_amount: Some(10.0),
            inferred_is_inflow: Some(true),
            inferred_inflow_type: Some(InFlowType::QuantityEqualAmount),
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
            inflow_type:  InFlowType::QuantityEqualAmount,
            outflow_type: OutFlowType::Manual,
            inventory:    vec![InventoryRecord {
                time_unix: 1,
                quantity:  10.0,
                amount:    20.0,
            }],
        });

        horizontal_infer_for_quantity_from_amount(100, &mut container, provider);
        let updated = &container.doubles[0].singles[0];
        assert_eq!(updated.get_inferred_quantity(), Some(10.0));
    }

    #[test]
    fn test_quantity_from_amount_with_empty_inventory() {
        let single = MockSingle {
            user_input_account_id: "1".to_string(),
            inferred_amount: Some(10.0),
            inferred_is_inflow: Some(false),
            inferred_outflow_type: Some(OutFlowType::Manual),
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
            inflow_type:  InFlowType::Manual,
            outflow_type: OutFlowType::Manual,
            inventory:    vec![],
        });

        horizontal_infer_for_quantity_from_amount(100, &mut container, provider);

        let updated = &container.doubles[0].singles[0];
        assert_eq!(updated.get_inferred_quantity(), None);
        assert_eq!(updated.get_inferred_amount(), Some(0.0));
    }

    #[test]
    fn test_quantity_from_amount_outflow_manual_amount_already_less_than_inventory() {
        let mut single = MockSingle {
            user_input_account_id: "1".to_string(),
            inferred_amount: Some(5.0),
            inferred_quantity: Some(2.0),
            inferred_is_inflow: Some(false),
            inferred_outflow_type: Some(OutFlowType::Manual),
            ..Default::default()
        };
        single.set_inferred_amount(Some(5.0));
        single.set_inferred_quantity(Some(2.0));
        single.set_inferred_is_inflow(Some(false));
        single.set_inferred_outflow_type(Some(OutFlowType::Manual));

        let double = MockDouble {
            singles: vec![single],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };

        let mut provider = new_account_info_provider();
        provider.insert("1".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  InFlowType::Manual,
            outflow_type: OutFlowType::Manual,
            inventory:    vec![InventoryRecord {
                time_unix: 1,
                quantity:  10.0,
                amount:    20.0,
            }],
        });

        horizontal_infer_for_quantity_from_amount(100, &mut container, provider);

        let updated = &container.doubles[0].singles[0];
        assert_eq!(updated.get_inferred_quantity(), Some(2.0));
        assert_eq!(updated.get_inferred_amount(), Some(5.0));
    }

    #[test]
    fn test_quantity_from_amount_outflow_cost_method_overwrites_quantity() {
        let mut single = MockSingle {
            user_input_account_id: "1".to_string(),
            inferred_amount: Some(10.0),
            inferred_quantity: Some(999.0),
            inferred_is_inflow: Some(false),
            inferred_outflow_type: Some(OutFlowType::Fifo),
            ..Default::default()
        };
        single.set_inferred_amount(Some(10.0));
        single.set_inferred_quantity(Some(999.0));
        single.set_inferred_is_inflow(Some(false));
        single.set_inferred_outflow_type(Some(OutFlowType::Fifo));

        let double = MockDouble {
            singles: vec![single],
        };
        let mut container = MockEntryContainer {
            doubles: vec![double],
        };

        let mut provider = new_account_info_provider();
        provider.insert("1".to_string(), MockAccountInfoProvider {
            is_debit:     true,
            inflow_type:  InFlowType::Manual,
            outflow_type: OutFlowType::Fifo,
            inventory:    vec![InventoryRecord {
                time_unix: 1,
                quantity:  5.0,
                amount:    10.0,
            }],
        });

        sort_inventory(
            OutFlowType::Fifo,
            &mut provider.get_mut(&"1".to_string()).unwrap().inventory,
        );

        horizontal_infer_for_quantity_from_amount(100, &mut container, provider);

        let updated = &container.doubles[0].singles[0];
        assert_eq!(updated.get_inferred_quantity(), Some(5.0));
        assert_eq!(updated.get_inferred_amount(), Some(10.0));
    }
}
