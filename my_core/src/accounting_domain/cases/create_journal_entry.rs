use crate::accounting_domain::utility::accounting_stuff;
use crate::accounting_domain::utility::types;
use crate::utility::traits;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::collections::HashSet;

pub type MyResult = Result<Ok, Error>;

// -----------------------------------------------------------------------------
// Input DTOs
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Input {
    pub new_uuid:                 types::UuidType,
    pub belong_to_company_branch: types::UuidType,
    pub user_uuid:                types::UuidType,
    pub notes:                    Option<String>,
    pub shared_entry_id:          Option<types::UuidType>,
    pub double_entries:           Vec<DoubleEntry>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DoubleEntry(Vec<SingleEntry>);

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SingleEntry {
    pub new_uuid: types::UuidType,
    pub account:  types::UuidType,
    pub status:   Status,
    pub quantity: f64,
    pub amount:   f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Status {
    M1 {
        is_debit: bool,
    },
    M2 {
        is_inflow: bool,
    },
    M3 {
        flow: accounting_stuff::CostFlowType,
    },
    M4 {
        is_debit: bool,
        in_flow:  accounting_stuff::InFlowType,
        out_flow: accounting_stuff::OutFlowType,
    },
}

// -----------------------------------------------------------------------------
// Output DTOs
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Ok {
    pub new_uuid:        types::UuidType,
    pub user_uuid:       types::UuidType,
    pub time:            u64,
    pub notes:           Option<String>,
    pub shared_entry_id: Option<types::UuidType>,
    pub double_entry:    Vec<SingleEntryOk>,
    pub inventory:       Vec<Inventory>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Inventory {
    pub row_uuid:  types::UuidType,
    pub account:   types::UuidType,
    pub time_unix: u64,
    pub quantity:  f64,
    pub amount:    f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SingleEntryOk {
    pub new_uuid:            types::UuidType,
    pub double_entry_number: u32,
    pub account:             types::UuidType,
    pub is_debit:            bool,
    pub out_flow_type:       accounting_stuff::OutFlowType,
    pub quantity:            f64,
    pub amount:              f64,
}

// -----------------------------------------------------------------------------
// Error types
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct Error {
    pub(crate) user_uuid:                Option<types::UserUuidError>,
    pub(crate) new_uuid:                 Option<types::RowIdError>,
    pub(crate) belong_to_company_branch: Option<types::RowIdError>,
    pub(crate) shared_entry_id:          Option<types::RowIdError>,
    pub(crate) double_entries:           Vec<DoubleEntryError>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct DoubleEntryError {
    pub(crate) accounting_error:    accounting_stuff::Error,
    pub(crate) single_entry_errors: Vec<SingleEntryError>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct SingleEntryError {
    pub(crate) new_uuid: Option<types::RowIdError>,
    pub(crate) account:  Option<types::RowIdError>,
}

impl types::MyErrorTrait for Error {}

// -----------------------------------------------------------------------------
// Database read types
// -----------------------------------------------------------------------------

pub struct ReadInput {
    pub new_uuid:                 types::UuidType,
    pub belong_to_company_branch: types::UuidType,
    pub user_uuid:                types::UuidType,
    pub shared_entry_id:          Option<types::UuidType>,
    pub accounts_uuid:            HashSet<types::UuidType>,
    pub new_entries_uuid:         HashSet<types::UuidType>,
}

pub struct ReadOutput {
    pub is_new_uuid_used:         bool,
    pub user_roles:               Vec<types::Role>,
    pub is_shared_entry_exist:    bool,
    pub is_new_entries_uuid_used: HashMap<types::UuidType, bool>,
    pub account_info:             HashMap<types::UuidType, AccountInfo>,
    pub inventory:                HashMap<types::UuidType, Vec<accounting_stuff::InventoryRecord>>,
}

pub struct AccountInfo {
    pub is_debit:      bool,
    pub in_flow_type:  accounting_stuff::InFlowType,
    pub out_flow_type: accounting_stuff::OutFlowType,
}

pub trait DatabaseRead {
    type Db<'a>;
    fn read(
        db: &mut Self::Db<'_>,
        read_input: &ReadInput,
    ) -> impl Future<Output = Result<ReadOutput, traits::DynamicError>>;
}

// -----------------------------------------------------------------------------
// Accounting trait implementations
// -----------------------------------------------------------------------------

// 1. SingleEntry – map our SingleEntry to the accounting trait
impl accounting_stuff::SingleEntry for SingleEntry {
    type AccountId = types::UuidType;

    fn account_id(&self) -> &Self::AccountId {
        &self.account
    }

    fn quantity(&self) -> f64 {
        self.quantity
    }

    fn amount(&self) -> f64 {
        self.amount
    }

    fn resolve_flow_type<A: accounting_stuff::AccountInfoProvider<AccountId = types::UuidType>>(
        &self,
        provider: &A,
    ) -> accounting_stuff::CostFlowType {
        match &self.status {
            Status::M1 {
                is_debit,
            } => {
                let (default_in, default_out) =
                    provider.get_default_flow_types(&self.account).unwrap_or((
                        accounting_stuff::InFlowType::None,
                        accounting_stuff::OutFlowType::None,
                    ));
                if *is_debit {
                    accounting_stuff::CostFlowType::InFlow(default_in)
                } else {
                    accounting_stuff::CostFlowType::OutFlow(default_out)
                }
            }
            Status::M2 {
                is_inflow,
            } => {
                let (default_in, default_out) =
                    provider.get_default_flow_types(&self.account).unwrap_or((
                        accounting_stuff::InFlowType::None,
                        accounting_stuff::OutFlowType::None,
                    ));
                if *is_inflow {
                    accounting_stuff::CostFlowType::InFlow(default_in)
                } else {
                    accounting_stuff::CostFlowType::OutFlow(default_out)
                }
            }
            Status::M3 {
                flow,
            } => flow.clone(),
            Status::M4 {
                is_debit,
                in_flow,
                out_flow,
            } => {
                if *is_debit {
                    accounting_stuff::CostFlowType::InFlow(in_flow.clone())
                } else {
                    accounting_stuff::CostFlowType::OutFlow(out_flow.clone())
                }
            }
        }
    }
}

// 2. EntryContainer – implement for Vec<SingleEntry>
impl accounting_stuff::EntryContainer for Vec<SingleEntry> {
    type Iter<'a> = std::slice::Iter<'a, SingleEntry>;
    type Single = SingleEntry;

    fn iter(&self) -> Self::Iter<'_> {
        self.as_slice().iter()
    }

    fn is_empty(&self) -> bool {
        Vec::is_empty(self)
    }

    fn len(&self) -> usize {
        Vec::len(self)
    }
}

// 3. Inventory – implement for Vec<accounting_stuff::InventoryRecord>
impl accounting_stuff::Inventory for Vec<accounting_stuff::InventoryRecord> {
    fn push(&mut self, record: accounting_stuff::InventoryRecord) {
        Vec::push(self, record);
    }

    fn clear(&mut self) {
        Vec::clear(self);
    }

    fn is_empty(&self) -> bool {
        Vec::is_empty(self)
    }

    fn iter(&self) -> impl Iterator<Item = &accounting_stuff::InventoryRecord> {
        self.as_slice().iter()
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut accounting_stuff::InventoryRecord> {
        self.as_mut_slice().iter_mut()
    }

    fn sort_by<F>(&mut self, compare: F)
    where
        F: FnMut(
            &accounting_stuff::InventoryRecord,
            &accounting_stuff::InventoryRecord,
        ) -> std::cmp::Ordering,
    {
        self.as_mut_slice().sort_by(compare);
    }

    fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&accounting_stuff::InventoryRecord) -> bool,
    {
        <Vec<accounting_stuff::InventoryRecord>>::retain(self, f);
    }

    fn pop(&mut self) -> Option<accounting_stuff::InventoryRecord> {
        Vec::pop(self)
    }
}

// 4. AccountInfoProvider – a local struct to hold both account info and inventory
pub struct AccountingState {
    pub account_infos: HashMap<types::UuidType, AccountInfo>,
    pub inventories:   HashMap<types::UuidType, Vec<accounting_stuff::InventoryRecord>>,
}

impl accounting_stuff::AccountInfoProvider for AccountingState {
    type AccountId = types::UuidType;
    type Inventory = Vec<accounting_stuff::InventoryRecord>;

    fn get_nature(&self, id: &Self::AccountId) -> Option<accounting_stuff::Nature> {
        self.account_infos.get(id).map(|info| {
            if info.is_debit {
                accounting_stuff::Nature::Debit
            } else {
                accounting_stuff::Nature::Credit
            }
        })
    }

    fn get_inventory_mut(&mut self, id: &Self::AccountId) -> Option<&mut Self::Inventory> {
        self.inventories.get_mut(id)
    }

    fn get_or_create_inventory(&mut self, id: &Self::AccountId) -> &mut Self::Inventory {
        self.inventories.entry(id.clone()).or_default()
    }

    fn get_default_flow_types(
        &self,
        id: &Self::AccountId,
    ) -> Option<(accounting_stuff::InFlowType, accounting_stuff::OutFlowType)> {
        self.account_infos
            .get(id)
            .map(|info| (info.in_flow_type.clone(), info.out_flow_type.clone()))
    }
}

// -----------------------------------------------------------------------------
// Input methods (business logic)
// -----------------------------------------------------------------------------

impl Input {
    pub(crate) fn state_less_check<Id: types::RowId>(&self) -> Error {
        let mut errr = Error::default();

        // Validate top-level UUIDs
        if !Id::validate(&self.new_uuid) {
            errr.new_uuid = Some(types::RowIdError::Invalid);
        }
        if !Id::validate(&self.belong_to_company_branch) {
            errr.belong_to_company_branch = Some(types::RowIdError::Invalid);
        }
        if !Id::validate(&self.user_uuid) {
            errr.user_uuid = Some(types::UserUuidError::Invalid);
        }
        if let Some(uuid) = &self.shared_entry_id {
            if !Id::validate(uuid) {
                errr.shared_entry_id = Some(types::RowIdError::Invalid);
            }
        }

        let mut seen_new_uuids = HashSet::new();

        // Validate each double entry
        for (i, double_entry) in self.double_entries.iter().enumerate() {
            let accounting_err = accounting_stuff::state_less_check_for_entry(&double_entry.0);
            let mut single_entry_errors = Vec::with_capacity(double_entry.0.len());

            for single in &double_entry.0 {
                let mut single_err = SingleEntryError::default();

                if !Id::validate(&single.new_uuid) {
                    single_err.new_uuid = Some(types::RowIdError::Invalid);
                }
                if !Id::validate(&single.account) {
                    single_err.account = Some(types::RowIdError::Invalid);
                }

                // Check duplicate new_uuid across all single entries
                if !seen_new_uuids.insert(single.new_uuid.clone()) {
                    single_err.new_uuid = Some(types::RowIdError::Duplicated);
                }

                single_entry_errors[i] = single_err;
            }

            errr.double_entries.push(DoubleEntryError {
                accounting_error: accounting_err,
                single_entry_errors,
            });
        }

        errr
    }

    pub(crate) async fn state_full_check<Db: DatabaseRead>(
        &self,
        db: &mut Db::Db<'_>,
    ) -> Result<Error, traits::DynamicError> {
        // Collect all account UUIDs and new entry UUIDs from all single entries
        let mut accounts_uuid = HashSet::new();
        let mut new_entries_uuid = HashSet::new();

        for double_entry in &self.double_entries {
            for single_entry in &double_entry.0 {
                accounts_uuid.insert(single_entry.account.clone());
                new_entries_uuid.insert(single_entry.new_uuid.clone());
            }
        }

        // Read from database
        let read_output = Db::read(db, &ReadInput {
            new_uuid: self.new_uuid.clone(),
            belong_to_company_branch: self.belong_to_company_branch.clone(),
            user_uuid: self.user_uuid.clone(),
            shared_entry_id: self.shared_entry_id.clone(),
            accounts_uuid,
            new_entries_uuid,
        })
        .await?;

        let mut errr = Error::default();

        // Check main entry UUID uniqueness
        if read_output.is_new_uuid_used {
            errr.new_uuid = Some(types::RowIdError::Duplicated);
        }

        // Check user permissions
        if !types::Role::has_any(&read_output.user_roles, &[
            types::Role::Manager,
            types::Role::CoManager,
        ]) {
            errr.user_uuid = Some(types::UserUuidError::YouDontHavePermissionToDoThat);
        }

        // Check shared entry existence if provided
        if self.shared_entry_id.is_some() {
            if !read_output.is_shared_entry_exist {
                errr.shared_entry_id = Some(types::RowIdError::NotExist);
            }
        }

        // Build accounting state from read output
        let mut accounting_state = AccountingState {
            account_infos: read_output.account_info,
            inventories:   read_output.inventory,
        };

        // Validate each double entry
        for double_entry in &self.double_entries {
            // Run full accounting check
            let accounting_err = accounting_stuff::state_full_check_for_entry(
                &double_entry.0,
                &mut accounting_state,
            );

            // Collect UUID errors for each single entry (duplicate check)
            let mut single_entry_errors = Vec::with_capacity(double_entry.0.len());
            for single in &double_entry.0 {
                let mut single_err = SingleEntryError::default();

                // Check if this new_uuid is already used
                if let Some(&used) = read_output.is_new_entries_uuid_used.get(&single.new_uuid) {
                    if used {
                        single_err.new_uuid = Some(types::RowIdError::Duplicated);
                    }
                }
                // Account existence is already validated in accounting_err (account_info_not_found)
                // So we don't need to duplicate it here.

                single_entry_errors.push(single_err);
            }

            errr.double_entries.push(DoubleEntryError {
                accounting_error: accounting_err,
                single_entry_errors,
            });
        }

        Ok(errr)
    }

    pub(crate) fn state_less_operation(&self) -> Ok {
        // TODO: implement actual transformation (requires inventory application)
        // For now, placeholder
        Ok {
            new_uuid:        self.new_uuid.clone(),
            user_uuid:       self.user_uuid.clone(),
            time:            0, // placeholder
            notes:           self.notes.clone(),
            shared_entry_id: self.shared_entry_id.clone(),
            double_entry:    Vec::new(), // placeholder
            inventory:       Vec::new(), // placeholder
        }
    }
}
