use crate::accounting_domain::utility::accounting_stuff;
use crate::accounting_domain::utility::types;
use crate::utility::traits;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::collections::HashSet;

pub type MyResult = Result<Ok, Error>;

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
    pub account_info:             HashMap<types::UuidType /* account uuid */, AccountInfo>,
    pub inventory:
        HashMap<types::UuidType /* account uuid */, Vec<accounting_stuff::InventoryRecord>>,
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

impl accounting_stuff::DoubleEntryUtile for Vec<SingleEntry> {
    type SingleEntry = SingleEntry;

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn len(&self) -> usize {
        self.len()
    }
}

impl accounting_stuff::SingleEntryUtile for SingleEntry {
    type AccountId = types::UuidType;

    fn get_account(&self) -> Self::AccountId {
        self.account.clone()
    }

    fn get_quantity(&self) -> f64 {
        self.quantity
    }

    fn get_amount(&self) -> f64 {
        self.amount
    }
}

impl Input {
    pub(crate) fn state_less_check<Id: types::RowId>(&self) -> Error {
        let mut errr = Error::default();

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

        for (i, double_entry) in self.double_entries.iter().enumerate() {
            errr.double_entries[i].accounting_error =
                accounting_stuff::state_less_check_for_entry(double_entry.0.clone());
        }
        errr
    }

    pub(crate) async fn state_full_check<Db: DatabaseRead>(
        &self,
        db: &mut Db::Db<'_>,
    ) -> Result<Error, traits::DynamicError> {
        let mut accounts_uuid = HashSet::new();

        for double_entry in &self.double_entries {
            for single_entry in &double_entry.0 {
                accounts_uuid.insert(single_entry.account.clone());
            }
        }

        let mut new_entries_uuid = HashSet::new();

        for double_entry in &self.double_entries {
            for single_entry in &double_entry.0 {
                new_entries_uuid.insert(single_entry.new_uuid.clone());
            }
        }

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

        if read_output.is_new_uuid_used {
            errr.new_uuid = Some(types::RowIdError::Duplicated);
        }

        if !types::Role::has_any(&read_output.user_roles, &[
            types::Role::Manager,
            types::Role::CoManager,
        ]) {
            errr.user_uuid = Some(types::UserUuidError::YouDontHavePermissionToDoThat);
        }

        Ok(errr)
    }

    pub(crate) fn state_less_operation(&self) -> Ok {
        Ok {
            new_uuid:        todo!(),
            user_uuid:       todo!(),
            time:            todo!(),
            notes:           todo!(),
            shared_entry_id: todo!(),
            double_entry:    todo!(),
            inventory:       todo!(),
        }
    }
}
