use crate::accounting_domain::utility::accounting_stuff::DoubleEntry;
use crate::accounting_domain::utility::accounting_stuff::EntryContainer;
use crate::accounting_domain::utility::accounting_stuff::Inventory;
use crate::accounting_domain::utility::accounting_stuff::{self};
use std::collections::HashSet;
use std::hash::Hash;

pub trait SingleEntry {
    type AccountId: Eq + Hash + Clone;

    fn get_from_user_input_account_id(&self) -> Self::AccountId;

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

    // Errors

    fn is_there_error_in_single_entry(&self) -> bool;

    fn quantity_and_amount_are_zero(&mut self);
    fn duplicate_account_in_entry(&mut self);
    fn inventory_is_empty(&mut self);
    fn the_amount_should_be_positive(&mut self);
    fn the_quantity_should_be_positive(&mut self);
    fn quantity_not_equal_amount(&mut self);
    fn quantity_not_equal_zero(&mut self);
    fn insufficient_quantity_in_inventory(&mut self, total_quantity: f64);
    fn amount_mismatch(&mut self, expected_amount: f64);
    fn insufficient_amount_in_inventory(&mut self, total_amount: f64);
}

pub trait AccountInfoProvider {
    type AccountId: Eq + Hash + Clone;
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
            single.set_inferred_is_debit(None);
            single.set_inferred_is_inflow(None);
            single.set_inferred_quantity(None);
            single.set_inferred_amount(None);
            single.set_inferred_inflow_type(None);
            single.set_inferred_outflow_type(None);
        }
    }
}

fn horizontal_correct_for_is_inflow<C, A>(entry: &mut C, account_info: &mut A)
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
            if let Some(_) = single.get_from_user_input_is_inflow() {
                let account_id = single.get_from_user_input_account_id();

                if let Some(info) = account_info.get_info(&account_id) {
                    if let Some(is_debit) = single.get_from_user_input_is_debit() {
                        single.set_user_input_is_inflow(Some(accounting_stuff::is_inflow(
                            info.is_debit,
                            is_debit,
                        )));
                    }
                }
            }
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
            if let Some(is_debit) = single.get_from_user_input_is_debit() {
                single.set_inferred_is_debit(Some(is_debit));
            } else {
                if let Some(is_inflow) = single.get_from_user_input_is_inflow() {
                    let account_id = single.get_from_user_input_account_id();
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
                let account_id = single.get_from_user_input_account_id();
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
            match single.get_from_user_input_inflow_type() {
                Some(inflow_type_from_user) => {
                    single.set_inferred_inflow_type(Some(inflow_type_from_user));
                }
                None => {
                    let account_id = single.get_from_user_input_account_id();

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
            match single.get_from_user_input_outflow_type() {
                Some(outflow_type_from_user) => {
                    single.set_inferred_outflow_type(Some(outflow_type_from_user));
                }
                None => {
                    let account_id = single.get_from_user_input_account_id();

                    if let Some(info) = account_info.get_info(&account_id) {
                        single.set_inferred_outflow_type(Some(info.outflow_type));
                    }
                }
            }
        }
    }
}

fn vertical_correct_to_remove_duplicate_account<C>(entry: &mut C)
where
    C: EntryContainer,
    C::Double: DoubleEntry,
    <C::Double as DoubleEntry>::Single: SingleEntry,
{
    for double in entry.iter_mut() {
        let mut seen_accounts = HashSet::new();
        double.retain(|single| seen_accounts.insert(single.get_from_user_input_account_id()));
    }
}

fn horizontal_correct_for_quantity_and_amount<C, A>(
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
            let mut quantity_from_user = match single.get_from_user_input_quantity() {
                Some(a) => a,
                None => continue,
            };

            let is_inflow = match single.get_inferred_is_inflow() {
                Some(a) => a,
                None => continue,
            };

            let account_id = single.get_from_user_input_account_id();

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
                        if let Some(_) = single.get_from_user_input_amount() {
                            single.set_user_input_amount(Some(quantity_from_user));
                        }
                    }
                    accounting_stuff::InFlowType::QuantityEqualZero => {
                        single.set_user_input_quantity(Some(0.0))
                    }
                }
            } else {
                let inferred_outflow_type = match single.get_inferred_outflow_type() {
                    Some(a) => a,
                    None => continue,
                };

                let total_quantity_in_inventory =
                    info.inventory.iter1().fold(0.0, |total, record| total + record.quantity);

                if quantity_from_user > total_quantity_in_inventory {
                    quantity_from_user = total_quantity_in_inventory;
                }

                accounting_stuff::sort_inventory(&inferred_outflow_type, info.inventory);

                match inferred_outflow_type {
                    accounting_stuff::OutFlowType::Manual => {
                        single.set_user_input_quantity(Some(quantity_from_user));

                        if let Some(mut amount_from_user) = single.get_from_user_input_amount() {
                            let total_amount_in_inventory = info
                                .inventory
                                .iter1()
                                .fold(0.0, |total, record| total + record.amount);

                            if amount_from_user > total_amount_in_inventory {
                                amount_from_user = total_amount_in_inventory;
                            }

                            single.set_user_input_amount(Some(amount_from_user));
                        };
                    }
                    accounting_stuff::OutFlowType::QuantityEqualAmount => {
                        single.set_user_input_quantity(Some(quantity_from_user));

                        if let Some(_) = single.get_from_user_input_amount() {
                            let total_amount_in_inventory = info
                                .inventory
                                .iter1()
                                .fold(0.0, |total, record| total + record.amount);

                            let mut amount_from_user = quantity_from_user;

                            if amount_from_user > total_amount_in_inventory {
                                amount_from_user = total_amount_in_inventory;

                                single.set_user_input_outflow_type(Some(
                                    accounting_stuff::OutFlowType::Manual,
                                ));

                                single.set_inferred_outflow_type(Some(
                                    accounting_stuff::OutFlowType::Manual,
                                ));
                            }

                            single.set_user_input_amount(Some(amount_from_user));
                        };
                    }
                    accounting_stuff::OutFlowType::QuantityEqualZero => {
                        single.set_user_input_quantity(Some(0.0));

                        if let Some(mut amount_from_user) = single.get_from_user_input_amount() {
                            let total_amount_in_inventory = info
                                .inventory
                                .iter1()
                                .fold(0.0, |total, record| total + record.amount);

                            if amount_from_user > total_amount_in_inventory {
                                amount_from_user = total_amount_in_inventory;
                            }

                            single.set_user_input_amount(Some(amount_from_user));
                        };
                    }
                    accounting_stuff::OutFlowType::Wac
                    | accounting_stuff::OutFlowType::Fifo
                    | accounting_stuff::OutFlowType::Lifo
                    | accounting_stuff::OutFlowType::Hifo
                    | accounting_stuff::OutFlowType::Lofo => {
                        let expected_amount =
                            accounting_stuff::get_amount(quantity_from_user, info.inventory);

                        single.set_user_input_amount(Some(expected_amount));
                    }
                };
            }

            if !single.is_there_error_in_single_entry()
                && let Some(amount) = single.get_inferred_amount()
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
    C::Double: DoubleEntry,
    <C::Double as DoubleEntry>::Single: SingleEntry,
    A: AccountInfoProvider<
        AccountId = <<C::Double as DoubleEntry>::Single as SingleEntry>::AccountId,
    >,
    A::Inventory: Inventory,
{
    reset_all_inferred_values(entry);
    vertical_correct_to_remove_duplicate_account(entry);
    horizontal_infer_for_is_debit(entry, &mut account_info);
    horizontal_infer_for_is_inflow(entry, &mut account_info);
    horizontal_infer_for_inflow_type(entry, &mut account_info);
    horizontal_infer_for_outflow_type(entry, &mut account_info);
    horizontal_correct_for_is_inflow(entry, &mut account_info);
    horizontal_correct_for_quantity_and_amount(time_unix, entry, &mut account_info);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounting_domain::utility::accounting_stuff::InventoryRecord;
    use crate::accounting_domain::utility::accounting_stuff::{self};
    use std::collections::HashMap;

    // ---------- Dummy inventory ----------

    impl accounting_stuff::Inventory for Vec<InventoryRecord> {
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

        fn sort_by<F>(&mut self, _compare: F)
        where
            F: FnMut(
                &accounting_stuff::InventoryRecord,
                &accounting_stuff::InventoryRecord,
            ) -> std::cmp::Ordering,
        {
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
        fn get_from_user_input_account_id(&self) -> Self::AccountId {
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

        // ---------- Error methods ----------
        fn is_there_error_in_single_entry(&self) -> bool {
            self.quantity_and_amount_are_zero
                || self.duplicate_account_in_entry
                || self.inventory_is_empty
                || self.the_amount_should_be_positive
                || self.the_quantity_should_be_positive
                || self.quantity_not_equal_amount
                || self.quantity_not_equal_zero
                || self.insufficient_quantity_in_inventory.is_some()
                || self.amount_mismatch.is_some()
                || self.insufficient_amount_in_inventory.is_some()
        }

        fn quantity_and_amount_are_zero(&mut self) {
            self.quantity_and_amount_are_zero = true;
        }

        fn duplicate_account_in_entry(&mut self) {
            self.duplicate_account_in_entry = true;
        }

        fn inventory_is_empty(&mut self) {
            self.inventory_is_empty = true;
        }

        fn the_amount_should_be_positive(&mut self) {
            self.the_amount_should_be_positive = true;
        }

        fn the_quantity_should_be_positive(&mut self) {
            self.the_quantity_should_be_positive = true;
        }

        fn quantity_not_equal_amount(&mut self) {
            self.quantity_not_equal_amount = true;
        }

        fn quantity_not_equal_zero(&mut self) {
            self.quantity_not_equal_zero = true;
        }

        fn insufficient_quantity_in_inventory(&mut self, total_quantity: f64) {
            self.insufficient_quantity_in_inventory = Some(total_quantity);
        }

        fn amount_mismatch(&mut self, expected_amount: f64) {
            self.amount_mismatch = Some(expected_amount);
        }

        fn insufficient_amount_in_inventory(&mut self, total_amount: f64) {
            self.insufficient_amount_in_inventory = Some(total_amount);
        }
    }

    // ---------- Mock DoubleEntry ----------
    #[derive(Debug)]
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
    fn horizontal_correct_for_flow_sets_inflow_when_info_exists() {
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
        horizontal_correct_for_is_inflow(&mut container, &mut provider);

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
        vertical_correct_to_remove_duplicate_account(&mut container);

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

        horizontal_correct_for_quantity_and_amount(100, &mut container, &mut provider);

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
            inflow_type:  accounting_stuff::InFlowType::Manual,
            outflow_type: accounting_stuff::OutFlowType::Manual,
            inventory:    Vec::new(),
        });

        horizontal_correct_for_quantity_and_amount(100, &mut container, &mut provider);

        let updated = &container.doubles[0].singles[0];
        assert_eq!(updated.get_from_user_input_quantity(), Some(5.0));
        assert_eq!(updated.get_from_user_input_amount(), Some(5.0)); // amount becomes quantity
    }

    #[test]
    fn horizontal_correct_for_quantity_and_amount_inflow_quantity_equal_zero() {
        // Inflow with QuantityEqualZero: quantity should be set to 0.
        let single = MockSingle {
            user_input_account_id: "1".to_string(),
            user_input_quantity: Some(5.0),
            user_input_amount: Some(10.0),
            inferred_is_inflow: Some(true),
            inferred_inflow_type: Some(accounting_stuff::InFlowType::QuantityEqualZero),
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

        horizontal_correct_for_quantity_and_amount(100, &mut container, &mut provider);

        let updated = &container.doubles[0].singles[0];
        assert_eq!(updated.get_from_user_input_quantity(), Some(0.0));
        assert_eq!(updated.get_from_user_input_amount(), Some(10.0)); // amount unchanged
    }

    #[test]
    fn horizontal_correct_for_quantity_and_amount_outflow_manual() {
        // Outflow with Manual: adjust quantity to inventory quantity, amount to inventory amount if exceeds.
        let single = MockSingle {
            user_input_account_id: "1".to_string(),
            user_input_quantity: Some(10.0),
            user_input_amount: Some(100.0),
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

        horizontal_correct_for_quantity_and_amount(100, &mut container, &mut provider);

        let updated = &container.doubles[0].singles[0];
        // Quantity should be capped at 5.0 (total inventory quantity)
        assert_eq!(updated.get_from_user_input_quantity(), Some(5.0));
        // Amount should be capped at 50.0 (total inventory amount)
        assert_eq!(updated.get_from_user_input_amount(), Some(50.0));
        // No error flags set
        assert!(!updated.is_there_error_in_single_entry());
    }
}
