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

    fn is_there_error_single_entry(&self) -> bool;

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

    fn get_info<'a>(&'a self, id: &Self::AccountId) -> Option<AccountInfo<'a, Self::Inventory>>;
}

pub struct AccountInfo<'a, I> {
    is_debit:     bool,
    inflow_type:  accounting_stuff::InFlowType,
    outflow_type: accounting_stuff::OutFlowType,
    inventory:    &'a mut I,
}

fn horizontal_correct_for_flow<C, A>(entry: &mut C, account_info: &A)
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

fn horizontal_infer_for_inflow_type<C, A>(entry: &mut C, account_info: &A)
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

fn horizontal_infer_for_outflow_type<C, A>(entry: &mut C, account_info: &A)
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

// fn horizontal_correct_for_quantity<C, A>(entry: &mut C, account_info: &mut A)
// where
//     C: EntryContainer,
//     C::Double: DoubleEntry,
//     <C::Double as DoubleEntry>::Single: SingleEntry,
//     A: AccountInfoProvider<
//         AccountId = <<C::Double as DoubleEntry>::Single as SingleEntry>::AccountId,
//     >,
//     A::Inventory: Inventory,
// {
//     for double in entry.iter_mut() {
//         for single in double.iter_mut() {
//             if let Some(quantity_from_user) = single.get_from_user_input_quantity() {
//                 if let Some(is_inflow) = single.get_inferred_is_inflow() {
//                     if is_inflow {
//                         if let Some(inferred_inflow_type) = single.get_inferred_inflow_type() {
//                             match inferred_inflow_type {
//                                 accounting_stuff::InFlowType::Manual => {}
//                                 accounting_stuff::InFlowType::QuantityEqualAmount => {}
//                                 accounting_stuff::InFlowType::QuantityEqualZero => {
//                                     single.set_user_input_quantity(Some(0.0))
//                                 }
//                             }
//                         }
//                     }
//                 } else {
//                     if let Some(inferred_outflow_type) = single.get_inferred_outflow_type() {
//                         let account_id = single.get_from_user_input_account_id();

//                         if let Some(info) = account_info.get_info(&account_id) {
//                             match inferred_outflow_type {
//                                 accounting_stuff::OutFlowType::Manual => {}
//                                 accounting_stuff::OutFlowType::QuantityEqualAmount => {}
//                                 accounting_stuff::OutFlowType::QuantityEqualZero => {
//                                     single.set_user_input_quantity(Some(0.0))
//                                 }
//                                 accounting_stuff::OutFlowType::Wac => todo!(),
//                                 accounting_stuff::OutFlowType::Fifo => todo!(),
//                                 accounting_stuff::OutFlowType::Lifo => todo!(),
//                                 accounting_stuff::OutFlowType::Hifo => todo!(),
//                                 accounting_stuff::OutFlowType::Lofo => todo!(),
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }

fn correct_the_input<C, A>(entry: &C, mut account_info: A)
where
    C: EntryContainer,
    C::Double: DoubleEntry,
    <C::Double as DoubleEntry>::Single: SingleEntry,
    A: AccountInfoProvider<
        AccountId = <<C::Double as DoubleEntry>::Single as SingleEntry>::AccountId,
    >,
    A::Inventory: Inventory,
{
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounting_domain::utility::accounting_stuff;
    use std::collections::HashMap;

    // ---------- Dummy inventory ----------
    #[derive(Default)]
    struct DummyInventory;

    impl accounting_stuff::Inventory for DummyInventory {
        fn push(&mut self, _record: accounting_stuff::InventoryRecord) {}

        fn clear(&mut self) {}

        fn is_empty(&self) -> bool {
            true
        }

        fn iter(&self) -> impl Iterator<Item = &accounting_stuff::InventoryRecord> {
            std::iter::empty()
        }

        fn iter_mut(&mut self) -> impl Iterator<Item = &mut accounting_stuff::InventoryRecord> {
            std::iter::empty()
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
        fn is_there_error_single_entry(&self) -> bool {
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
        pub infos:
            HashMap<String, (bool, accounting_stuff::InFlowType, accounting_stuff::OutFlowType)>,
        // Raw pointer to a leaked DummyInventory. We never mutate it.
        inventory: *mut DummyInventory,
    }

    // Ensure the provider is Send/Sync (not needed but safe)
    unsafe impl Send for MockAccountInfoProvider {}

    impl MockAccountInfoProvider {
        pub fn new(
            infos: HashMap<
                String,
                (bool, accounting_stuff::InFlowType, accounting_stuff::OutFlowType),
            >,
        ) -> Self {
            let inventory = Box::leak(Box::new(DummyInventory));
            Self {
                infos,
                inventory: inventory as *mut DummyInventory,
            }
        }
    }

    impl AccountInfoProvider for MockAccountInfoProvider {
        type AccountId = String;
        type Inventory = DummyInventory;

        fn get_info<'a>(
            &'a self,
            id: &Self::AccountId,
        ) -> Option<AccountInfo<'a, Self::Inventory>> {
            self.infos.get(id).map(|(is_debit, inflow_type, outflow_type)| {
                // Safety: The raw pointer is valid for the entire lifetime of the provider,
                // and we never mutate the inventory (only read is_debit from infos).
                let inventory_ref = unsafe { &mut *self.inventory };
                AccountInfo {
                    is_debit:     *is_debit,
                    inflow_type:  inflow_type.clone(),
                    outflow_type: outflow_type.clone(),
                    inventory:    inventory_ref,
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
        let mut infos = HashMap::new();
        infos.insert(
            "1".to_string(),
            (true, accounting_stuff::InFlowType::Manual, accounting_stuff::OutFlowType::Manual),
        );
        infos.insert(
            "2".to_string(),
            (false, accounting_stuff::InFlowType::Manual, accounting_stuff::OutFlowType::Manual),
        );
        let provider = MockAccountInfoProvider::new(infos);

        // Call the function
        horizontal_correct_for_flow(&mut container, &provider);

        // Verify
        let updated_double = &container.doubles[0];

        assert_eq!(updated_double.singles[0].get_from_user_input_is_inflow(), None);
        assert_eq!(updated_double.singles[1].get_from_user_input_is_inflow(), Some(true));
        assert_eq!(updated_double.singles[2].get_from_user_input_is_inflow(), Some(true));
        assert_eq!(updated_double.singles[3].get_from_user_input_is_inflow(), Some(false));
        assert_eq!(updated_double.singles[4].get_from_user_input_is_inflow(), Some(false));
        assert_eq!(updated_double.singles[5].get_from_user_input_is_inflow(), Some(true));
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
        let mut infos = HashMap::new();
        infos.insert(
            "1".to_string(),
            (
                true,
                accounting_stuff::InFlowType::QuantityEqualZero,
                accounting_stuff::OutFlowType::Manual,
            ),
        );
        infos.insert(
            "2".to_string(),
            (
                true,
                accounting_stuff::InFlowType::QuantityEqualAmount,
                accounting_stuff::OutFlowType::Manual,
            ),
        );
        infos.insert(
            "3".to_string(),
            (
                true,
                accounting_stuff::InFlowType::QuantityEqualZero,
                accounting_stuff::OutFlowType::Manual,
            ),
        );

        let provider = MockAccountInfoProvider::new(infos);

        // Call the function
        horizontal_infer_for_inflow_type(&mut container, &provider);

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
        let mut infos = HashMap::new();
        infos.insert(
            "1".to_string(),
            (
                true,
                accounting_stuff::InFlowType::QuantityEqualZero,
                accounting_stuff::OutFlowType::Manual,
            ),
        );
        infos.insert(
            "2".to_string(),
            (
                true,
                accounting_stuff::InFlowType::QuantityEqualAmount,
                accounting_stuff::OutFlowType::Manual,
            ),
        );
        infos.insert(
            "3".to_string(),
            (
                true,
                accounting_stuff::InFlowType::QuantityEqualZero,
                accounting_stuff::OutFlowType::QuantityEqualZero,
            ),
        );

        let provider = MockAccountInfoProvider::new(infos);

        // Call the function
        horizontal_infer_for_outflow_type(&mut container, &provider);

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
}
