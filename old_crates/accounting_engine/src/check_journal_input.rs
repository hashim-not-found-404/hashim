use crate::accounting_stuff::DoubleEntry;
use crate::accounting_stuff::EntryContainer;
use crate::accounting_stuff::InFlowType;
use crate::accounting_stuff::Inventory;
use crate::accounting_stuff::OutFlowType;
use crate::accounting_stuff::apply_entry_on_inventory;
use crate::accounting_stuff::get_amount;
use crate::accounting_stuff::is_decrease_by_price;
use crate::accounting_stuff::is_inflow;
use crate::accounting_stuff::sort_inventory;
use crate::accounting_stuff::sum_inventory;
use crate::common_subset_sum::split_to_max;
use crate::number_type::Num;
use std::collections::HashSet;
use std::hash::Hash;

pub trait SingleEntryError {
    fn is_there_error(&self) -> bool;

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

pub trait DoubleEntryError {
    fn is_there_error(&self) -> bool;

    fn entry_is_empty(&mut self);
    fn you_need_to_split_the_entry(&mut self);
    fn debit_not_equal_credit(&mut self, total_debit: f64, total_credit: f64);
}

pub trait EntryContainerError {
    fn is_there_error(&self) -> bool;

    fn container_is_empty(&mut self);
}

pub trait SingleEntry {
    type AccountId: Eq + Hash;

    fn account_id(&self) -> Self::AccountId;
    fn is_debit(&self) -> bool;
    fn quantity(&self) -> f64;
    fn amount(&self) -> f64;
    fn inflow_type(&self) -> InFlowType;
    fn outflow_type(&self) -> OutFlowType;
}

pub trait AccountInfoProvider {
    type AccountId: Eq + Hash + Clone;
    type Inventory: Inventory;

    fn is_debit_nature(&self, id: &Self::AccountId) -> bool;
    fn get_or_create_inventory(&mut self, id: &Self::AccountId) -> &mut Self::Inventory;
}

pub fn state_less_check_for_entry<'a, C>(entry: &'a mut C)
where
    C: EntryContainer + EntryContainerError + 'a,
    C::Double<'a>: DoubleEntry + DoubleEntryError,
    <C::Double<'a> as DoubleEntry>::Single: SingleEntry + SingleEntryError + Clone,
{
    if entry.is_empty() {
        entry.container_is_empty();
        return;
    }

    for double in entry.iter_mut() {
        if double.is_empty() {
            double.entry_is_empty();
            continue;
        }

        let mut seen_accounts = HashSet::with_capacity(double.len());

        let mut total_debit = 0.0;
        let mut total_credit = 0.0;

        let mut debit_side = Vec::new();
        let mut credit_side = Vec::new();

        for single in double.iter_mut() {
            if !seen_accounts.insert(single.account_id()) {
                single.duplicate_account_in_entry();
            }

            if single.amount() == 0.0 && single.quantity() == 0.0 {
                single.quantity_and_amount_are_zero();
            }
            if single.amount() < 0.0 {
                single.the_amount_should_be_positive();
            }
            if single.quantity() < 0.0 {
                single.the_quantity_should_be_positive();
            }

            if single.is_debit() {
                total_debit += single.amount();
                debit_side.push(single.clone());
            } else {
                total_credit += single.amount();
                credit_side.push(single.clone());
            }
        }

        if split_to_max(&debit_side, &credit_side, &|a| Num(a.amount())).len() > 1 {
            double.you_need_to_split_the_entry();
        }

        if total_debit != total_credit {
            double.debit_not_equal_credit(total_debit, total_credit);
        }
    }
}

pub fn state_full_check_for_entry<'a, C, A>(time_unix: u64, entry: &'a mut C, account_info: &mut A)
where
    C: EntryContainer + EntryContainerError + 'a,
    C::Double<'a>: DoubleEntry + DoubleEntryError,
    <C::Double<'a> as DoubleEntry>::Single: SingleEntry + SingleEntryError,
    A: AccountInfoProvider<
        AccountId = <<C::Double<'a> as DoubleEntry>::Single as SingleEntry>::AccountId,
    >,
    A::Inventory: Inventory,
{
    for double in entry.iter_mut() {
        for single in double.iter_mut() {
            let account_id = single.account_id();
            let nature = account_info.is_debit_nature(&account_id);
            let in_flow_type = single.inflow_type();
            let out_flow_type = single.outflow_type();
            let inventory = account_info.get_or_create_inventory(&account_id);

            if inventory.is_empty() {
                single.inventory_is_empty();
            }

            let is_inflow = is_inflow(nature, single.is_debit());

            let (amt, qty) = if is_inflow {
                (single.amount(), single.quantity())
            } else {
                (-single.amount(), -single.quantity())
            };

            if is_inflow {
                match in_flow_type {
                    InFlowType::Manual => {}
                    InFlowType::QuantityEqualAmount => {
                        if qty != amt {
                            single.quantity_not_equal_amount();
                        }
                    }
                    InFlowType::QuantityEqualZero => {
                        if qty != 0.0 {
                            single.quantity_not_equal_zero();
                        }
                    }
                }
            } else {
                let (total_qty, total_amt) = sum_inventory(inventory);
                if total_amt + amt < 0.0 {
                    single.insufficient_amount_in_inventory(total_amt);
                }
                if total_qty + qty < 0.0 {
                    single.insufficient_quantity_in_inventory(total_qty);
                }

                sort_inventory(out_flow_type, inventory);

                match out_flow_type {
                    OutFlowType::Manual => {}
                    OutFlowType::Wac
                    | OutFlowType::Fifo
                    | OutFlowType::Lifo
                    | OutFlowType::Hifo
                    | OutFlowType::Lofo => {
                        let expected_amount = get_amount(single.quantity(), inventory);
                        if expected_amount != single.amount() {
                            single.amount_mismatch(expected_amount);
                        }
                    }
                    OutFlowType::QuantityEqualAmount => {
                        if qty != amt {
                            single.quantity_not_equal_amount();
                        }
                    }
                    OutFlowType::QuantityEqualZero => {
                        if qty != 0.0 {
                            single.quantity_not_equal_zero();
                        }
                    }
                }
            }

            if !single.is_there_error() {
                apply_entry_on_inventory::<A::Inventory>(
                    time_unix,
                    single.amount(),
                    single.quantity(),
                    is_inflow,
                    is_decrease_by_price(out_flow_type),
                    inventory,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounting_stuff::InventoryRecord;
    use std::collections::HashMap;

    #[derive(Debug, Clone, Default, PartialEq)]
    pub(crate) struct DebitNotEqualCreditError {
        total_debit:  f64,
        total_credit: f64,
    }

    impl SingleEntryError for TestSingleEntry {
        fn is_there_error(&self) -> bool {
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

    impl DoubleEntryError for TestDoubleEntry {
        fn is_there_error(&self) -> bool {
            if self.entry_is_empty
                || self.you_need_to_split_the_entry
                || self.debit_not_equal_credit.is_some()
            {
                return true;
            }

            for i in &self.lines {
                if i.is_there_error() {
                    return true;
                }
            }

            false
        }

        fn entry_is_empty(&mut self) {
            self.entry_is_empty = true;
        }

        fn you_need_to_split_the_entry(&mut self) {
            self.you_need_to_split_the_entry = true;
        }

        fn debit_not_equal_credit(&mut self, total_debit: f64, total_credit: f64) {
            self.debit_not_equal_credit = Some(DebitNotEqualCreditError {
                total_debit,
                total_credit,
            });
        }
    }

    impl EntryContainerError for TestEntryContainer {
        fn is_there_error(&self) -> bool {
            if self.container_is_empty {
                return true;
            }

            for i in self.groups.clone() {
                if i.is_there_error() {
                    return true;
                }
            }

            false
        }

        fn container_is_empty(&mut self) {
            self.container_is_empty = true;
        }
    }

    #[derive(Debug, Clone, Default, Eq, Hash, PartialEq)]
    struct AccountId(String);

    #[derive(Debug, Clone, Default, PartialEq)]
    struct TestSingleEntry {
        account:  AccountId,
        debit:    bool,
        qty:      f64,
        amt:      f64,
        in_flow:  InFlowType,
        out_flow: OutFlowType,

        quantity_and_amount_are_zero:       bool,
        duplicate_account_in_entry:         bool,
        inventory_is_empty:                 bool,
        the_amount_should_be_positive:      bool,
        the_quantity_should_be_positive:    bool,
        quantity_not_equal_amount:          bool,
        quantity_not_equal_zero:            bool,
        insufficient_quantity_in_inventory: Option<f64>,
        amount_mismatch:                    Option<f64>,
        insufficient_amount_in_inventory:   Option<f64>,
    }

    impl SingleEntry for TestSingleEntry {
        type AccountId = AccountId;

        fn account_id(&self) -> Self::AccountId {
            self.account.clone()
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

        fn inflow_type(&self) -> InFlowType {
            self.in_flow.clone()
        }

        fn outflow_type(&self) -> OutFlowType {
            self.out_flow.clone()
        }
    }

    #[derive(Debug, Clone, Default, PartialEq)]
    struct TestDoubleEntry {
        lines: Vec<TestSingleEntry>,

        entry_is_empty:              bool,
        you_need_to_split_the_entry: bool,
        debit_not_equal_credit:      Option<DebitNotEqualCreditError>,
    }

    impl DoubleEntry for TestDoubleEntry {
        type Iter<'a> = std::vec::IntoIter<TestSingleEntry>;
        type IterMut<'a> = std::slice::IterMut<'a, TestSingleEntry>;
        type IterRef<'a> = std::slice::Iter<'a, TestSingleEntry>;
        type Single = TestSingleEntry;

        fn into_iter<'a>(self) -> Self::Iter<'a> {
            self.lines.into_iter()
        }

        fn iter_ref(&self) -> Self::IterRef<'_> {
            self.lines.iter()
        }

        fn iter_mut(&mut self) -> Self::IterMut<'_> {
            self.lines.iter_mut()
        }

        fn set_singles(&mut self, singles: Vec<Self::Single>) {
            self.lines = singles;
        }

        fn is_empty(&self) -> bool {
            self.lines.is_empty()
        }

        fn len(&self) -> usize {
            self.lines.len()
        }

        fn retain<F>(&mut self, f: F)
        where
            F: FnMut(&Self::Single) -> bool,
        {
            self.lines.retain(f);
        }
    }

    #[derive(Debug, Clone, Default, PartialEq)]
    struct TestEntryContainer {
        groups: Vec<TestDoubleEntry>,

        container_is_empty: bool,
    }

    impl EntryContainer for TestEntryContainer {
        type Double<'a> = TestDoubleEntry;
        type Iter<'a> = std::vec::IntoIter<TestDoubleEntry>;
        type IterMut<'a> = std::slice::IterMut<'a, TestDoubleEntry>;
        type IterRef<'a> = std::slice::Iter<'a, TestDoubleEntry>;

        fn iter<'a>(self) -> Self::Iter<'a> {
            self.groups.into_iter()
        }

        fn iter_ref(&self) -> Self::IterRef<'_> {
            self.groups.iter()
        }

        fn iter_mut(&mut self) -> Self::IterMut<'_> {
            self.groups.iter_mut()
        }

        fn set_doubles(&mut self, doubles: Vec<Self::Double<'_>>) {
            self.groups = doubles;
        }

        fn is_empty(&self) -> bool {
            self.groups.is_empty()
        }

        fn len(&self) -> usize {
            self.groups.len()
        }

        fn retain<F>(&mut self, f: F)
        where
            F: FnMut(&Self::Double<'_>) -> bool,
        {
            self.groups.retain(f);
        }
    }

    type TestInventory = Vec<InventoryRecord>;

    struct TestAccountInfoProvider {
        natures:     HashMap<AccountId, bool>,
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
            *self.natures.get(id).unwrap()
        }

        fn get_or_create_inventory(&mut self, id: &Self::AccountId) -> &mut Self::Inventory {
            self.inventories.entry(id.clone()).or_insert_with(TestInventory::default)
        }
    }

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
            ..Default::default()
        }
    }

    fn simple_entry(account: &str, debit: bool, qty: f64, amt: f64) -> TestSingleEntry {
        entry(account, debit, qty, amt, InFlowType::Manual, OutFlowType::Manual)
    }

    #[test]
    fn test_state_less_empty_entry() {
        let mut entry = TestEntryContainer {
            groups: vec![],
            ..Default::default()
        };

        state_less_check_for_entry(&mut entry);
        assert!(entry.container_is_empty);
        assert!(entry.is_there_error());
    }

    #[test]
    fn test_state_less_duplicate_account() {
        let single1 = simple_entry("A", true, 1.0, 10.0);
        let single2 = simple_entry("A", false, 1.0, 10.0);
        let double = TestDoubleEntry {
            lines: vec![single1, single2],
            ..Default::default()
        };
        let mut entry = TestEntryContainer {
            groups: vec![double],
            ..Default::default()
        };
        state_less_check_for_entry(&mut entry);
        assert!(!entry.container_is_empty);
        assert!(entry.is_there_error());
        let de = &entry.groups[0];
        assert!(!de.entry_is_empty);
        assert_eq!(de.lines.len(), 2);
        assert!(de.lines[1].duplicate_account_in_entry);
    }

    #[test]
    fn test_state_less_zero_qty_and_amount() {
        let single = entry("A", true, 0.0, 0.0, InFlowType::Manual, OutFlowType::Manual);
        let double = TestDoubleEntry {
            lines: vec![single],
            ..Default::default()
        };
        let mut entry = TestEntryContainer {
            groups: vec![double],
            ..Default::default()
        };
        state_less_check_for_entry(&mut entry);
        let se = &entry.groups[0].lines[0];
        assert!(se.quantity_and_amount_are_zero);
    }

    #[test]
    fn test_state_less_negative_values() {
        let single = simple_entry("A", true, -1.0, -5.0);
        let double = TestDoubleEntry {
            lines: vec![single],
            ..Default::default()
        };
        let mut entry = TestEntryContainer {
            groups: vec![double],
            ..Default::default()
        };
        state_less_check_for_entry(&mut entry);
        let se = &entry.groups[0].lines[0];
        assert!(se.the_amount_should_be_positive);
        assert!(se.the_quantity_should_be_positive);
    }

    #[test]
    fn test_state_less_debit_credit_mismatch() {
        let d1 = simple_entry("A", true, 1.0, 10.0);
        let c1 = simple_entry("B", false, 1.0, 8.0);
        let double = TestDoubleEntry {
            lines: vec![d1, c1],
            ..Default::default()
        };
        let mut entry = TestEntryContainer {
            groups: vec![double],
            ..Default::default()
        };
        state_less_check_for_entry(&mut entry);
        let de = &entry.groups[0];
        assert!(de.debit_not_equal_credit.is_some());
        let dnc = de.debit_not_equal_credit.as_ref().unwrap();
        assert_eq!(dnc.total_debit, 10.0);
        assert_eq!(dnc.total_credit, 8.0);
    }

    #[test]
    fn test_state_less_splittable_entry() {
        let d1 = entry("A", true, 1.0, 1.0, InFlowType::Manual, OutFlowType::Manual);
        let d2 = entry("D", true, 1.0, 4.0, InFlowType::Manual, OutFlowType::Manual);
        let d3 = entry("E", true, 1.0, 5.0, InFlowType::Manual, OutFlowType::Manual);
        let c1 = entry("B", false, 1.0, 2.0, InFlowType::Manual, OutFlowType::Manual);
        let c2 = entry("C", false, 1.0, 3.0, InFlowType::Manual, OutFlowType::Manual);
        let c3 = entry("F", false, 1.0, 5.0, InFlowType::Manual, OutFlowType::Manual);
        let double = TestDoubleEntry {
            lines: vec![d1, d2, d3, c1, c2, c3],
            ..Default::default()
        };
        let mut entry = TestEntryContainer {
            groups: vec![double],
            ..Default::default()
        };
        state_less_check_for_entry(&mut entry);
        let de = &entry.groups[0];
        assert!(de.you_need_to_split_the_entry);
    }

    #[test]
    fn test_state_less_non_splittable_entry() {
        let d1 = simple_entry("A", true, 1.0, 1.0);
        let d2 = simple_entry("B", true, 1.0, 2.0);
        let d3 = simple_entry("F", true, 1.0, 6.0);
        let c1 = simple_entry("D", false, 1.0, 4.0);
        let c2 = simple_entry("E", false, 1.0, 5.0);
        let double = TestDoubleEntry {
            lines: vec![d1, d2, d3, c1, c2],
            ..Default::default()
        };
        let mut entry = TestEntryContainer {
            groups: vec![double],
            ..Default::default()
        };
        state_less_check_for_entry(&mut entry);
        let de = &entry.groups[0];
        assert!(!de.you_need_to_split_the_entry);
    }

    fn setup_provider() -> TestAccountInfoProvider {
        let mut provider = TestAccountInfoProvider::new();
        provider.add_account(AccountId("A".to_string()), true);
        provider.add_account(AccountId("B".to_string()), true);
        provider.add_account(AccountId("C".to_string()), true);
        provider.add_account(AccountId("D".to_string()), true);
        provider.add_account(AccountId("E".to_string()), true);
        provider.add_account(AccountId("F".to_string()), true);
        provider.add_account(AccountId("G".to_string()), false);
        provider
    }

    #[test]
    fn test_state_full_inventory_empty_error() {
        let mut provider = setup_provider();
        let single = simple_entry("A", true, 1.0, 10.0);
        let double = TestDoubleEntry {
            lines: vec![single],
            ..Default::default()
        };
        let mut entry = TestEntryContainer {
            groups: vec![double],
            ..Default::default()
        };
        state_full_check_for_entry(100000000000, &mut entry, &mut provider);
        let se = &entry.groups[0].lines[0];
        assert!(se.inventory_is_empty);
    }

    #[test]
    fn test_state_full_quantity_equal_amount_inflow() {
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  10.0,
            amount:    100.0,
        });

        let single =
            entry("A", true, 5.0, 5.0, InFlowType::QuantityEqualAmount, OutFlowType::Manual);
        let double = TestDoubleEntry {
            lines: vec![single],
            ..Default::default()
        };
        let mut entry = TestEntryContainer {
            groups: vec![double],
            ..Default::default()
        };
        state_full_check_for_entry(100000000000, &mut entry, &mut provider);
        let se = &entry.groups[0].lines[0];
        assert!(!se.quantity_not_equal_amount);
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

        let single =
            entry("A", true, 5.0, 4.0, InFlowType::QuantityEqualAmount, OutFlowType::Manual);
        let double = TestDoubleEntry {
            lines: vec![single],
            ..Default::default()
        };
        let mut entry = TestEntryContainer {
            groups: vec![double],
            ..Default::default()
        };
        state_full_check_for_entry(100000000000, &mut entry, &mut provider);
        let se = &entry.groups[0].lines[0];
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
            ..Default::default()
        };
        let mut entry = TestEntryContainer {
            groups: vec![double],
            ..Default::default()
        };
        state_full_check_for_entry(100000000000, &mut entry, &mut provider);
        let se = &entry.groups[0].lines[0];
        assert!(!se.quantity_not_equal_zero);
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

        let single =
            entry("A", true, 1.0, 10.0, InFlowType::QuantityEqualZero, OutFlowType::Manual);
        let double = TestDoubleEntry {
            lines: vec![single],
            ..Default::default()
        };
        let mut entry = TestEntryContainer {
            groups: vec![double],
            ..Default::default()
        };
        state_full_check_for_entry(100000000000, &mut entry, &mut provider);
        let se = &entry.groups[0].lines[0];
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
        });

        let single = entry("A", false, 5.0, 20.0, InFlowType::Manual, OutFlowType::Manual);
        let double = TestDoubleEntry {
            lines: vec![single],
            ..Default::default()
        };
        let mut entry = TestEntryContainer {
            groups: vec![double],
            ..Default::default()
        };
        state_full_check_for_entry(100000000000, &mut entry, &mut provider);
        let se = &entry.groups[0].lines[0];
        assert!(se.insufficient_amount_in_inventory.is_some());
        let ia = se.insufficient_amount_in_inventory.as_ref().unwrap();
        assert_eq!(*ia, 10.0);
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

        let single = entry("A", false, 5.0, 20.0, InFlowType::Manual, OutFlowType::Manual);
        let double = TestDoubleEntry {
            lines: vec![single],
            ..Default::default()
        };
        let mut entry = TestEntryContainer {
            groups: vec![double],
            ..Default::default()
        };
        state_full_check_for_entry(100000000000, &mut entry, &mut provider);
        let se = &entry.groups[0].lines[0];
        assert!(se.insufficient_quantity_in_inventory.is_some());
        let iq = se.insufficient_quantity_in_inventory.as_ref().unwrap();
        assert_eq!(*iq, 2.0);
    }

    #[test]
    fn test_state_full_outflow_amount_mismatch_wac() {
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  10.0,
            amount:    100.0,
        });

        let single = entry("A", false, 2.0, 25.0, InFlowType::Manual, OutFlowType::Wac);
        let double = TestDoubleEntry {
            lines: vec![single],
            ..Default::default()
        };
        let mut entry = TestEntryContainer {
            groups: vec![double],
            ..Default::default()
        };
        state_full_check_for_entry(100000000000, &mut entry, &mut provider);
        let se = &entry.groups[0].lines[0];
        assert!(se.amount_mismatch.is_some());
        let am = se.amount_mismatch.as_ref().unwrap();
        assert_eq!(*am, 20.0);
    }

    #[test]
    fn test_state_full_outflow_amount_mismatch_fifo() {
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
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

        let single = entry("A", false, 4.0, 20.0, InFlowType::Manual, OutFlowType::Fifo);
        let double = TestDoubleEntry {
            lines: vec![single],
            ..Default::default()
        };
        let mut entry = TestEntryContainer {
            groups: vec![double],
            ..Default::default()
        };
        state_full_check_for_entry(100000000000, &mut entry, &mut provider);
        let se = &entry.groups[0].lines[0];
        assert!(se.amount_mismatch.is_some());
        let am = se.amount_mismatch.as_ref().unwrap();
        assert_eq!(*am, 16.0);
    }

    #[test]
    fn test_state_full_outflow_quantity_equal_amount() {
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  10.0,
            amount:    20.0,
        });

        let single =
            entry("A", false, 2.0, 2.0, InFlowType::Manual, OutFlowType::QuantityEqualAmount);
        let double = TestDoubleEntry {
            lines: vec![single],
            ..Default::default()
        };
        let mut entry = TestEntryContainer {
            groups: vec![double],
            ..Default::default()
        };
        state_full_check_for_entry(100000000000, &mut entry, &mut provider);
        let se = &entry.groups[0].lines[0];
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
            ..Default::default()
        };
        let mut entry = TestEntryContainer {
            groups: vec![double],
            ..Default::default()
        };
        state_full_check_for_entry(100000000000, &mut entry, &mut provider);
        let se = &entry.groups[0].lines[0];
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
            ..Default::default()
        };
        let mut entry = TestEntryContainer {
            groups: vec![double],
            ..Default::default()
        };
        state_full_check_for_entry(100000000000, &mut entry, &mut provider);
        let se = &entry.groups[0].lines[0];
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
            ..Default::default()
        };
        let mut entry = TestEntryContainer {
            groups: vec![double],
            ..Default::default()
        };
        state_full_check_for_entry(100000000000, &mut entry, &mut provider);
        let se = &entry.groups[0].lines[0];
        assert!(se.quantity_not_equal_zero);
    }

    #[test]
    fn test_state_full_outflow_empty_inventory() {
        let mut provider = setup_provider();
        let single = simple_entry("A", false, 1.0, 10.0);
        let double = TestDoubleEntry {
            lines: vec![single],
            ..Default::default()
        };
        let mut entry = TestEntryContainer {
            groups: vec![double],
            ..Default::default()
        };
        state_full_check_for_entry(100000000000, &mut entry, &mut provider);
        let se = &entry.groups[0].lines[0];
        assert!(se.inventory_is_empty);
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
        let mut entry = TestEntryContainer {
            groups: vec![TestDoubleEntry {
                lines: vec![single],
                ..Default::default()
            }],
            ..Default::default()
        };

        state_full_check_for_entry(100, &mut entry, &mut provider);

        let (qty, amt) = sum_inventory(&provider.inventories[&AccountId("A".to_string())]);
        assert_eq!(qty, 5.0);
        assert_eq!(amt, 10.0);
    }

    #[test]
    fn test_state_less_duplicate_account_with_three_entries() {
        let e1 = simple_entry("A", true, 1.0, 10.0);
        let e2 = simple_entry("B", false, 1.0, 5.0);
        let e3 = simple_entry("A", true, 1.0, 5.0);
        let double = TestDoubleEntry {
            lines: vec![e1, e2, e3],
            ..Default::default()
        };
        let mut entry = TestEntryContainer {
            groups: vec![double],
            ..Default::default()
        };
        state_less_check_for_entry(&mut entry);
        let se = &entry.groups[0].lines;
        assert!(!se[0].duplicate_account_in_entry);
        assert!(!se[1].duplicate_account_in_entry);
        assert!(se[2].duplicate_account_in_entry);
    }

    #[test]
    fn test_state_less_empty_double_entry() {
        let double = TestDoubleEntry {
            lines: vec![],
            ..Default::default()
        };
        let mut entry = TestEntryContainer {
            groups: vec![double],
            ..Default::default()
        };
        state_less_check_for_entry(&mut entry);
        assert!(!entry.container_is_empty);
        assert!(entry.is_there_error());
        let de = &entry.groups[0];
        assert!(de.entry_is_empty);
        assert_eq!(de.lines.len(), 0);
    }

    #[test]
    fn test_state_less_multiple_double_entries() {
        let d1 = simple_entry("A", true, 1.0, 10.0);
        let c1 = simple_entry("B", false, 1.0, 10.0);
        let double1 = TestDoubleEntry {
            lines: vec![d1, c1],
            ..Default::default()
        };

        let d2 = simple_entry("C", true, 1.0, 5.0);
        let c2 = simple_entry("D", false, 1.0, 3.0);
        let double2 = TestDoubleEntry {
            lines: vec![d2, c2],
            ..Default::default()
        };

        let mut entry = TestEntryContainer {
            groups: vec![double1, double2],
            ..Default::default()
        };
        state_less_check_for_entry(&mut entry);
        assert!(entry.is_there_error());
        let de1 = &entry.groups[0];
        assert!(de1.debit_not_equal_credit.is_none());
        let de2 = &entry.groups[1];
        assert!(de2.debit_not_equal_credit.is_some());
        let dnc = de2.debit_not_equal_credit.as_ref().unwrap();
        assert_eq!(dnc.total_debit, 5.0);
        assert_eq!(dnc.total_credit, 3.0);
    }

    #[test]
    fn test_state_full_wac_combines_even_on_error() {
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
        let single = entry("A", false, 10.0, 20.0, InFlowType::Manual, OutFlowType::Wac);
        let double = TestDoubleEntry {
            lines: vec![single],
            ..Default::default()
        };
        let mut entry = TestEntryContainer {
            groups: vec![double],
            ..Default::default()
        };
        state_full_check_for_entry(100, &mut entry, &mut provider);

        let se = &entry.groups[0].lines[0];
        assert!(se.insufficient_quantity_in_inventory.is_some());

        let inv_after = provider.get_or_create_inventory(&AccountId("A".to_string()));
        assert_eq!(inv_after.iter1().count(), 1);
        let rec = inv_after.iter1().next().unwrap();
        assert_eq!(rec.quantity, 8.0);
        assert_eq!(rec.amount, 16.0);
    }

    #[test]
    fn test_state_full_manual_outflow_does_combine() {
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
            ..Default::default()
        };
        let mut entry = TestEntryContainer {
            groups: vec![double],
            ..Default::default()
        };

        state_full_check_for_entry(100, &mut entry, &mut provider);

        let inv_after = provider.get_or_create_inventory(&AccountId("A".to_string()));
        assert_eq!(inv_after.iter1().count(), 1);
        let records: Vec<_> = inv_after.iter1().collect();
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

        sort_inventory(OutFlowType::Fifo, inv);
        let amt = get_amount(4.0, inv);
        assert_eq!(amt, 8.0);
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

        sort_inventory(OutFlowType::Lifo, inv);
        let amt = get_amount(4.0, inv);
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
        });
        inv.push(InventoryRecord {
            time_unix: 2,
            quantity:  3.0,
            amount:    9.0,
        });
        inv.push(InventoryRecord {
            time_unix: 3,
            quantity:  2.0,
            amount:    8.0,
        });

        sort_inventory(OutFlowType::Hifo, inv);
        let amt = get_amount(3.0, inv);
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
        });
        inv.push(InventoryRecord {
            time_unix: 2,
            quantity:  3.0,
            amount:    9.0,
        });
        inv.push(InventoryRecord {
            time_unix: 3,
            quantity:  2.0,
            amount:    8.0,
        });

        sort_inventory(OutFlowType::Lofo, inv);
        let amt = get_amount(6.0, inv);
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

        sort_inventory(OutFlowType::Wac, inv);
        let amt = get_amount(4.0, inv);
        let expected = 4.0 * (19.0 / 8.0);
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

    #[test]
    fn test_apply_entry_normal_inflow() {
        let mut inv = TestInventory::default();
        apply_entry_on_inventory(100, 20.0, 5.0, true, true, &mut inv);
        assert_eq!(inv.iter1().count(), 1);
        let rec = inv.iter1().next().unwrap();
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
        apply_entry_on_inventory(200, 30.0, 10.0, false, true, &mut inv);
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
        apply_entry_on_inventory(300, 3.0, 0.0, true, true, &mut inv);
        let (qty, amt) = sum_inventory(&inv);
        assert_eq!(qty, 5.0);
        assert_eq!(amt, 13.0);
        assert_eq!(inv.iter1().count(), 1);
        let rec = inv.iter1().next().unwrap();
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
        apply_entry_on_inventory(400, 0.0, 2.0, true, true, &mut inv);
        let (qty, amt) = sum_inventory(&inv);
        assert_eq!(qty, 7.0);
        assert_eq!(amt, 10.0);
        let rec = inv.iter1().next().unwrap();
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
        apply_entry_on_inventory(500, 0.0, 2.0, false, true, &mut inv);
        let (qty, amt) = sum_inventory(&inv);
        assert_eq!(qty, 3.0);
        assert_eq!(amt, 10.0);
        let rec = inv.iter1().next().unwrap();
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
        apply_entry_on_inventory(600, 5.0, 0.0, false, true, &mut inv);
        let (qty, amt) = sum_inventory(&inv);
        assert_eq!(qty, 5.0);
        assert_eq!(amt, 5.0);
        let rec = inv.iter1().next().unwrap();
        assert_eq!(rec.time_unix, 600);
        assert_eq!(rec.quantity, 5.0);
        assert_eq!(rec.amount, 5.0);
    }

    #[test]
    fn test_apply_entry_no_panics() {
        let mut inv = TestInventory::default();
        apply_entry_on_inventory(700, 10.0, -5.0, true, true, &mut inv);
    }

    #[test]
    fn test_single_entry_error_is_there_error() {
        let err = TestSingleEntry::default();
        assert!(!err.is_there_error());

        let mut err2 = TestSingleEntry::default();
        err2.quantity_and_amount_are_zero = true;
        assert!(err2.is_there_error());

        let mut err3 = TestSingleEntry::default();
        err3.insufficient_quantity_in_inventory = Some(5.0);
        assert!(err3.is_there_error());
    }

    #[test]
    fn test_state_full_outflow_amount_mismatch_lifo() {
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

        let single = entry("A", false, 4.0, 15.0, InFlowType::Manual, OutFlowType::Lifo);
        let double = TestDoubleEntry {
            lines: vec![single],
            ..Default::default()
        };
        let mut entry = TestEntryContainer {
            groups: vec![double],
            ..Default::default()
        };
        state_full_check_for_entry(100, &mut entry, &mut provider);
        let se = &entry.groups[0].lines[0];
        assert!(se.amount_mismatch.is_some());
        let am = se.amount_mismatch.as_ref().unwrap();
        assert_eq!(*am, 11.0);
    }

    #[test]
    fn test_state_full_outflow_amount_mismatch_hifo() {
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
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

        let single = entry("A", false, 3.0, 12.0, InFlowType::Manual, OutFlowType::Hifo);
        let double = TestDoubleEntry {
            lines: vec![single],
            ..Default::default()
        };
        let mut entry = TestEntryContainer {
            groups: vec![double],
            ..Default::default()
        };
        state_full_check_for_entry(100, &mut entry, &mut provider);
        let se = &entry.groups[0].lines[0];
        assert!(se.amount_mismatch.is_some());
        let am = se.amount_mismatch.as_ref().unwrap();
        assert_eq!(*am, 11.0);
    }

    #[test]
    fn test_state_full_outflow_amount_mismatch_lofo() {
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
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

        let single = entry("A", false, 6.0, 14.0, InFlowType::Manual, OutFlowType::Lofo);
        let double = TestDoubleEntry {
            lines: vec![single],
            ..Default::default()
        };
        let mut entry = TestEntryContainer {
            groups: vec![double],
            ..Default::default()
        };
        state_full_check_for_entry(100, &mut entry, &mut provider);
        let se = &entry.groups[0].lines[0];
        assert!(se.amount_mismatch.is_some());
        let am = se.amount_mismatch.as_ref().unwrap();
        assert_eq!(*am, 13.0);
    }

    #[test]
    fn test_state_full_credit_nature_account_inflow() {
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
            ..Default::default()
        };
        let mut entry = TestEntryContainer {
            groups: vec![double],
            ..Default::default()
        };
        state_full_check_for_entry(100, &mut entry, &mut provider);
        assert!(entry.groups[0].lines[0].is_there_error() == false);
        let (qty, amt) = sum_inventory(&provider.inventories[&AccountId("G".to_string())]);
        assert_eq!(qty, 7.0);
        assert_eq!(amt, 20.0);
    }

    #[test]
    fn test_state_full_credit_nature_account_outflow() {
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
            ..Default::default()
        };
        let mut entry = TestEntryContainer {
            groups: vec![double],
            ..Default::default()
        };
        state_full_check_for_entry(100, &mut entry, &mut provider);
        assert!(entry.groups[0].lines[0].is_there_error() == false);
        let (qty, amt) = sum_inventory(&provider.inventories[&AccountId("G".to_string())]);
        assert_eq!(qty, 3.0);
        assert_eq!(amt, 6.0);
    }

    #[test]
    fn test_state_full_partial_application_when_one_line_errors() {
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        });

        let e1 = entry("A", false, 5.0, 20.0, InFlowType::Manual, OutFlowType::Manual);
        let e2 = entry("A", true, 3.0, 15.0, InFlowType::Manual, OutFlowType::Manual);
        let double = TestDoubleEntry {
            lines: vec![e1, e2],
            ..Default::default()
        };
        let mut entry = TestEntryContainer {
            groups: vec![double],
            ..Default::default()
        };
        state_full_check_for_entry(100, &mut entry, &mut provider);

        assert!(&entry.groups[0].lines[0].insufficient_amount_in_inventory.is_some());
        assert!(!&entry.groups[0].lines[1].is_there_error());

        let (qty, amt) = sum_inventory(&provider.inventories[&AccountId("A".to_string())]);
        assert_eq!(qty, 8.0);
        assert_eq!(amt, 25.0);
    }

    #[test]
    fn test_double_entry_error_is_there_error() {
        let mut de = TestDoubleEntry::default();
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
        let se = TestSingleEntry {
            quantity_and_amount_are_zero: true,
            ..Default::default()
        };
        de.lines.push(se);
        assert!(de.is_there_error());
    }

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
    fn test_apply_entry_rare_clear_inventory() {
        let mut inv = TestInventory::default();
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        });
        apply_entry_on_inventory(200, 0.0, 5.0, false, true, &mut inv);
        assert_eq!(inv[0].quantity, 0.0);
        assert_eq!(inv[0].amount, 10.0);
    }

    #[test]
    fn test_apply_entry_rare_adjustment_where_total_becomes_zero() {
        let mut inv = TestInventory::default();
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        });
        apply_entry_on_inventory(300, 10.0, 0.0, false, true, &mut inv);
        assert_eq!(inv[0].quantity, 5.0);
        assert_eq!(inv[0].amount, 0.0);
    }

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
            ..Default::default()
        };
        let mut entry = TestEntryContainer {
            groups: vec![double],
            ..Default::default()
        };
        state_full_check_for_entry(100, &mut entry, &mut provider);
        assert!(!&entry.groups[0].lines[0].is_there_error());
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
            ..Default::default()
        };
        let mut entry = TestEntryContainer {
            groups: vec![double],
            ..Default::default()
        };
        state_full_check_for_entry(100, &mut entry, &mut provider);
        assert!(!&entry.groups[0].lines[0].is_there_error());
        let (qty, amt) = sum_inventory(&provider.inventories[&AccountId("A".to_string())]);
        assert_eq!(qty, 7.0);
        assert_eq!(amt, 10.0);
    }

    #[test]
    fn test_state_full_rare_outflow_amount_negative_qty_zero() {
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        });

        let single = entry("A", false, 0.0, 5.0, InFlowType::Manual, OutFlowType::Manual);
        let double = TestDoubleEntry {
            lines: vec![single],
            ..Default::default()
        };
        let mut entry = TestEntryContainer {
            groups: vec![double],
            ..Default::default()
        };
        state_full_check_for_entry(100, &mut entry, &mut provider);
        assert!(!&entry.groups[0].lines[0].is_there_error());
        let (qty, amt) = sum_inventory(&provider.inventories[&AccountId("A".to_string())]);
        assert_eq!(qty, 5.0);
        assert_eq!(amt, 5.0);
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
            ..Default::default()
        };
        let mut entry = TestEntryContainer {
            groups: vec![double],
            ..Default::default()
        };
        state_full_check_for_entry(100, &mut entry, &mut provider);
        assert!(!&entry.groups[0].lines[0].is_there_error());
        let (qty, amt) = sum_inventory(&provider.inventories[&AccountId("A".to_string())]);
        assert_eq!(qty, 3.0);
        assert_eq!(amt, 10.0);
    }

    #[test]
    fn test_sort_inventory_hifo_direct() {
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
        inv.push(InventoryRecord {
            time_unix: 3,
            quantity:  2.0,
            amount:    8.0,
        });
        sort_inventory(OutFlowType::Hifo, &mut inv);
        let prices: Vec<f64> = inv.iter1().map(|r| r.amount / r.quantity).collect();
        assert_eq!(prices, vec![4.0, 3.0, 2.0]);
    }

    #[test]
    fn test_sort_inventory_lofo_direct() {
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
        inv.push(InventoryRecord {
            time_unix: 3,
            quantity:  2.0,
            amount:    8.0,
        });
        sort_inventory(OutFlowType::Lofo, &mut inv);
        let prices: Vec<f64> = inv.iter1().map(|r| r.amount / r.quantity).collect();
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
            ..Default::default()
        };
        let mut entry = TestEntryContainer {
            groups: vec![double],
            ..Default::default()
        };
        state_full_check_for_entry(100, &mut entry, &mut provider);

        let se = &entry.groups[0].lines[0];
        assert!(se.amount_mismatch.is_none());
        let (qty, amt) = sum_inventory(&provider.inventories[&AccountId("A".to_string())]);
        assert_eq!(qty, 3.0);
        assert_eq!(amt, 0.0);
    }

    #[test]
    fn test_state_full_manual_outflow_amount_more_than_inv() {
        let mut provider = setup_provider();
        let inv = provider.get_or_create_inventory(&AccountId("A".to_string()));
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        });

        let single = entry("A", false, 2.0, 11.0, InFlowType::Manual, OutFlowType::Manual);
        let double = TestDoubleEntry {
            lines: vec![single],
            ..Default::default()
        };
        let mut entry = TestEntryContainer {
            groups: vec![double],
            ..Default::default()
        };
        state_full_check_for_entry(100, &mut entry, &mut provider);

        let se = &entry.groups[0].lines[0];
        assert!(se.amount_mismatch.is_none());
        let (qty, amt) = sum_inventory(&provider.inventories[&AccountId("A".to_string())]);
        assert_eq!(qty, 5.0);
        assert_eq!(amt, 10.0);
    }

    #[test]
    fn test_apply_entry_manual_outflow_uses_given_amount() {
        let mut inv = TestInventory::default();
        inv.push(InventoryRecord {
            time_unix: 1,
            quantity:  5.0,
            amount:    10.0,
        });
        apply_entry_on_inventory(100, 3.0, 2.0, false, false, &mut inv);
        let (qty, amt) = sum_inventory(&inv);
        assert_eq!(qty, 3.0);
        assert_eq!(amt, 7.0);
    }
}
