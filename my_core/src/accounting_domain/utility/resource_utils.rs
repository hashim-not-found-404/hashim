use crate::accounting_domain::utility::accounting_stuff;
use crate::accounting_domain::utility::types;
use crate::utility::utils::MyUpSert;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct User {
    pub(crate) id:       String,
    pub(crate) name:     Option<String>,
    pub(crate) password: String,
}

#[derive(Default)]
pub(crate) struct Company {
    pub(crate) currency: types::Currency,
    pub(crate) name:     String,
}

#[derive(Default)]
pub(crate) struct AccessControlForCompany {
    pub(crate) data_group: types::UuidType,
    pub(crate) role:       types::Role,
    pub(crate) user_:      types::UuidType,
}

#[derive(Default)]
pub(crate) struct CompanyBranch {
    pub(crate) company_belong: types::UuidType,
    pub(crate) currency:       types::Currency,
    pub(crate) location:       types::Location,
    pub(crate) name:           String,
}

#[derive(Default)]
pub(crate) struct AccessControlForCompanyBranch {
    pub(crate) data_group: types::UuidType,
    pub(crate) role:       types::Role,
    pub(crate) user_:      types::UuidType,
}

#[derive(Default)]
pub(crate) struct Account {
    pub(crate) company_belong:                  types::UuidType,
    pub(crate) is_debit:                        bool,
    pub(crate) is_permanent_account:            bool,
    pub(crate) name:                            String,
    pub(crate) notes:                           String,
    pub(crate) unit_of_measurement_of_quantity: String,
    pub(crate) inventory:                       Vec<accounting_stuff::InventoryRecord>,
}

#[derive(Default)]
pub(crate) struct AccountFlowType {
    pub(crate) account:        types::UuidType,
    pub(crate) company_branch: types::UuidType,
    pub(crate) outflow_type:   accounting_stuff::OutFlowType,
    pub(crate) inflow_type:    accounting_stuff::InFlowType,
}

#[derive(Default)]
pub(crate) struct SharedEntry {
    pub(crate) writer: types::UuidType,
    pub(crate) notes:  Option<String>,
}

#[derive(Default)]
pub(crate) struct Entry {
    pub(crate) writer:          types::UuidType,
    pub(crate) time:            u64,
    pub(crate) shared_entry_id: types::UuidType,
}

#[derive(Default)]
pub(crate) struct SingleEntry {
    pub(crate) double_entry:       u32,
    pub(crate) entry:              types::UuidType,
    pub(crate) account:            types::UuidType,
    pub(crate) is_debit:           bool,
    pub(crate) cost_out_flow_type: accounting_stuff::OutFlowType,
    pub(crate) quantity:           f64,
    pub(crate) amount:             f64,
}

// -----------------------------------------------------------------------------

#[derive(Default)]
pub(crate) struct StateOfPendingTxn {
    pub(crate) access_control_for_company:        HashMap<types::UuidType, AccessControlForCompany>,
    pub(crate) access_control_for_company_branch:
        HashMap<types::UuidType, AccessControlForCompanyBranch>,
    pub(crate) account:                           HashMap<types::UuidType, Account>,
    pub(crate) account_flow_type:                 HashMap<types::UuidType, AccountFlowType>,
    pub(crate) company:                           HashMap<types::UuidType, Company>,
    pub(crate) company_branch:                    HashMap<types::UuidType, CompanyBranch>,
    pub(crate) user:                              HashMap<types::UuidType, User>,

    // ---- NEW: Pending transactions for journal entries ----
    pub(crate) shared_entry: HashMap<types::UuidType, SharedEntry>,
    pub(crate) entry:        HashMap<types::UuidType, Entry>,
    pub(crate) single_entry: HashMap<types::UuidType, SingleEntry>,
}

// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Subscribe {
    // Existing subscribes
    TableAccessControlForCompanyBranchFieldDataGroup,
    TableAccessControlForCompanyBranchFieldRole,
    TableAccessControlForCompanyBranchFieldUser,
    TableAccessControlForCompanyFieldDataGroup,
    TableAccessControlForCompanyFieldRole,
    TableAccessControlForCompanyFieldUser,
    TableAccountFieldCompanyBelong,
    TableAccountFieldIsDebit,
    TableAccountFieldIsPermanentAccount,
    TableAccountFieldName,
    TableAccountFieldNotes,
    TableAccountFieldUnitOfMeasurementOfQuantity,
    TableAccountFieldInventory,
    TableAccountFlowTypeFieldAccount,
    TableAccountFlowTypeFieldCompanyBranch,
    TableAccountFlowTypeFieldInflowType,
    TableAccountFlowTypeFieldOutflowType,
    TableCompanyBranchFieldCompanyBelong,
    TableCompanyBranchFieldCurrency,
    TableCompanyBranchFieldLocation,
    TableCompanyBranchFieldName,
    TableCompanyFieldCurrency,
    TableCompanyFieldName,
    TableUserFieldId,
    TableUserFieldName,
    TableSharedEntryFieldWriter,
    TableSharedEntryFieldNotes,
    TableEntryFieldWriter,
    TableEntryFieldTime,
    TableEntryFieldSharedEntryId,
    TableSingleEntryFieldDoubleEntry,
    TableSingleEntryFieldEntry,
    TableSingleEntryFieldAccount,
    TableSingleEntryFieldIsDebit,
    TableSingleEntryFieldCostOutFlowType,
    TableSingleEntryFieldQuantity,
    TableSingleEntryFieldAmount,
}

// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Resource {
    // Existing resources
    Jwt(types::JsonWebTokenType),

    TableAccessControlForCompanyBranchFieldDataGroup(types::UuidType),
    TableAccessControlForCompanyBranchFieldRole(types::Role),
    TableAccessControlForCompanyBranchFieldUser(types::UuidType),
    TableAccessControlForCompanyFieldDataGroup(types::UuidType),
    TableAccessControlForCompanyFieldRole(types::Role),
    TableAccessControlForCompanyFieldUser(types::UuidType),
    TableAccountFieldCompanyBelong(types::UuidType),
    TableAccountFieldIsDebit(bool),
    TableAccountFieldIsPermanentAccount(bool),
    TableAccountFieldName(String),
    TableAccountFieldNotes(String),
    TableAccountFieldUnitOfMeasurementOfQuantity(String),
    TableAccountFieldInventory(Vec<accounting_stuff::InventoryRecord>),
    TableAccountFlowTypeFieldAccount(types::UuidType),
    TableAccountFlowTypeFieldCompanyBranch(types::UuidType),
    TableAccountFlowTypeFieldInflowType(accounting_stuff::InFlowType),
    TableAccountFlowTypeFieldOutflowType(accounting_stuff::OutFlowType),
    TableCompanyBranchFieldCompanyBelong(types::UuidType),
    TableCompanyBranchFieldCurrency(types::Currency),
    TableCompanyBranchFieldLocation(types::Location),
    TableCompanyBranchFieldName(String),
    TableCompanyFieldCurrency(types::Currency),
    TableCompanyFieldName(String),
    TableUserFieldId(String),
    TableUserFieldName(String),
    TableSharedEntryFieldWriter(types::UuidType),
    TableSharedEntryFieldNotes(Option<String>),
    TableEntryFieldWriter(types::UuidType),
    TableEntryFieldTime(u64),
    TableEntryFieldSharedEntryId(types::UuidType),
    TableSingleEntryFieldDoubleEntry(u32),
    TableSingleEntryFieldEntry(types::UuidType),
    TableSingleEntryFieldAccount(types::UuidType),
    TableSingleEntryFieldIsDebit(bool),
    TableSingleEntryFieldCostOutFlowType(accounting_stuff::OutFlowType),
    TableSingleEntryFieldQuantity(f64),
    TableSingleEntryFieldAmount(f64),
}

impl Resource {
    pub(crate) fn map_to_subs(&self) -> Option<Subscribe> {
        match self {
            Resource::Jwt(_) => None,
            Resource::TableUserFieldName(_) => Some(Subscribe::TableUserFieldName),
            Resource::TableUserFieldId(_) => Some(Subscribe::TableUserFieldId),
            Resource::TableCompanyFieldName(_) => Some(Subscribe::TableCompanyFieldName),
            Resource::TableCompanyFieldCurrency(_) => Some(Subscribe::TableCompanyFieldCurrency),
            Resource::TableCompanyBranchFieldName(_) => {
                Some(Subscribe::TableCompanyBranchFieldName)
            }
            Resource::TableCompanyBranchFieldCompanyBelong(_) => {
                Some(Subscribe::TableCompanyBranchFieldCompanyBelong)
            }
            Resource::TableCompanyBranchFieldLocation(_) => {
                Some(Subscribe::TableCompanyBranchFieldLocation)
            }
            Resource::TableCompanyBranchFieldCurrency(_) => {
                Some(Subscribe::TableCompanyBranchFieldCurrency)
            }
            Resource::TableAccessControlForCompanyFieldRole(_) => {
                Some(Subscribe::TableAccessControlForCompanyFieldRole)
            }
            Resource::TableAccessControlForCompanyFieldUser(_) => {
                Some(Subscribe::TableAccessControlForCompanyFieldUser)
            }
            Resource::TableAccessControlForCompanyFieldDataGroup(_) => {
                Some(Subscribe::TableAccessControlForCompanyFieldDataGroup)
            }
            Resource::TableAccessControlForCompanyBranchFieldRole(_) => {
                Some(Subscribe::TableAccessControlForCompanyBranchFieldRole)
            }
            Resource::TableAccessControlForCompanyBranchFieldUser(_) => {
                Some(Subscribe::TableAccessControlForCompanyBranchFieldUser)
            }
            Resource::TableAccessControlForCompanyBranchFieldDataGroup(_) => {
                Some(Subscribe::TableAccessControlForCompanyBranchFieldDataGroup)
            }
            Resource::TableAccountFieldCompanyBelong(_) => {
                Some(Subscribe::TableAccountFieldCompanyBelong)
            }
            Resource::TableAccountFieldIsDebit(_) => Some(Subscribe::TableAccountFieldIsDebit),
            Resource::TableAccountFieldIsPermanentAccount(_) => {
                Some(Subscribe::TableAccountFieldIsPermanentAccount)
            }
            Resource::TableAccountFieldName(_) => Some(Subscribe::TableAccountFieldName),
            Resource::TableAccountFieldNotes(_) => Some(Subscribe::TableAccountFieldNotes),
            Resource::TableAccountFieldUnitOfMeasurementOfQuantity(_) => {
                Some(Subscribe::TableAccountFieldUnitOfMeasurementOfQuantity)
            }
            Resource::TableAccountFlowTypeFieldAccount(_) => {
                Some(Subscribe::TableAccountFlowTypeFieldAccount)
            }
            Resource::TableAccountFlowTypeFieldCompanyBranch(_) => {
                Some(Subscribe::TableAccountFlowTypeFieldCompanyBranch)
            }
            Resource::TableAccountFlowTypeFieldInflowType(_) => {
                Some(Subscribe::TableAccountFlowTypeFieldInflowType)
            }
            Resource::TableAccountFlowTypeFieldOutflowType(_) => {
                Some(Subscribe::TableAccountFlowTypeFieldOutflowType)
            }

            // ---- NEW: Journal entry mappings ----
            Resource::TableSharedEntryFieldWriter(_) => {
                Some(Subscribe::TableSharedEntryFieldWriter)
            }
            Resource::TableSharedEntryFieldNotes(_) => Some(Subscribe::TableSharedEntryFieldNotes),
            Resource::TableEntryFieldWriter(_) => Some(Subscribe::TableEntryFieldWriter),
            Resource::TableEntryFieldTime(_) => Some(Subscribe::TableEntryFieldTime),
            Resource::TableEntryFieldSharedEntryId(_) => {
                Some(Subscribe::TableEntryFieldSharedEntryId)
            }
            Resource::TableSingleEntryFieldDoubleEntry(_) => {
                Some(Subscribe::TableSingleEntryFieldDoubleEntry)
            }
            Resource::TableSingleEntryFieldEntry(_) => Some(Subscribe::TableSingleEntryFieldEntry),
            Resource::TableSingleEntryFieldAccount(_) => {
                Some(Subscribe::TableSingleEntryFieldAccount)
            }
            Resource::TableSingleEntryFieldIsDebit(_) => {
                Some(Subscribe::TableSingleEntryFieldIsDebit)
            }
            Resource::TableSingleEntryFieldCostOutFlowType(_) => {
                Some(Subscribe::TableSingleEntryFieldCostOutFlowType)
            }
            Resource::TableSingleEntryFieldQuantity(_) => {
                Some(Subscribe::TableSingleEntryFieldQuantity)
            }
            Resource::TableSingleEntryFieldAmount(_) => {
                Some(Subscribe::TableSingleEntryFieldAmount)
            }
            Resource::TableAccountFieldInventory(_) => Some(Subscribe::TableAccountFieldInventory),
        }
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ResourceInfo {
    pub row_uuid: types::UuidType,
    pub resource: Resource,
}

pub(crate) fn apply_change(resources: Vec<ResourceInfo>, state: &mut StateOfPendingTxn) {
    for resource in resources {
        let row_uuid = resource.row_uuid;

        match resource.resource {
            Resource::Jwt(_) => {}
            Resource::TableUserFieldName(r) => {
                state.user.upsert(row_uuid, |table| table.name = Some(r))
            }
            Resource::TableUserFieldId(r) => state.user.upsert(row_uuid, |table| table.id = r),
            Resource::TableCompanyFieldName(r) => {
                state.company.upsert(row_uuid, |table| table.name = r)
            }
            Resource::TableCompanyBranchFieldName(r) => {
                state.company_branch.upsert(row_uuid, |table| table.name = r)
            }
            Resource::TableCompanyBranchFieldCompanyBelong(r) => {
                state.company_branch.upsert(row_uuid, |table| table.company_belong = r)
            }
            Resource::TableCompanyBranchFieldCurrency(r) => {
                state.company_branch.upsert(row_uuid, |table| table.currency = r)
            }
            Resource::TableCompanyBranchFieldLocation(r) => {
                state.company_branch.upsert(row_uuid, |table| table.location = r)
            }
            Resource::TableCompanyFieldCurrency(r) => {
                state.company.upsert(row_uuid, |table| table.currency = r)
            }
            Resource::TableAccessControlForCompanyFieldRole(r) => {
                state.access_control_for_company.upsert(row_uuid, |table| table.role = r)
            }
            Resource::TableAccessControlForCompanyFieldUser(r) => {
                state.access_control_for_company.upsert(row_uuid, |table| table.user_ = r)
            }
            Resource::TableAccessControlForCompanyFieldDataGroup(r) => {
                state.access_control_for_company.upsert(row_uuid, |table| table.data_group = r)
            }
            Resource::TableAccessControlForCompanyBranchFieldRole(r) => {
                state.access_control_for_company_branch.upsert(row_uuid, |table| table.role = r)
            }
            Resource::TableAccessControlForCompanyBranchFieldUser(r) => {
                state.access_control_for_company_branch.upsert(row_uuid, |table| table.user_ = r)
            }
            Resource::TableAccessControlForCompanyBranchFieldDataGroup(r) => {
                state
                    .access_control_for_company_branch
                    .upsert(row_uuid, |table| table.data_group = r)
            }
            Resource::TableAccountFieldCompanyBelong(r) => {
                state.account.upsert(row_uuid, |table| table.company_belong = r)
            }
            Resource::TableAccountFieldIsDebit(r) => {
                state.account.upsert(row_uuid, |table| table.is_debit = r)
            }
            Resource::TableAccountFieldIsPermanentAccount(r) => {
                state.account.upsert(row_uuid, |table| table.is_permanent_account = r)
            }
            Resource::TableAccountFieldName(r) => {
                state.account.upsert(row_uuid, |table| table.name = r)
            }
            Resource::TableAccountFieldNotes(r) => {
                state.account.upsert(row_uuid, |table| table.notes = r)
            }
            Resource::TableAccountFieldUnitOfMeasurementOfQuantity(r) => {
                state.account.upsert(row_uuid, |table| table.unit_of_measurement_of_quantity = r)
            }
            Resource::TableAccountFieldInventory(r) => {
                state.account.upsert(row_uuid, |table| table.inventory = r)
            }
            Resource::TableAccountFlowTypeFieldAccount(r) => {
                state.account_flow_type.upsert(row_uuid, |table| table.account = r)
            }
            Resource::TableAccountFlowTypeFieldCompanyBranch(r) => {
                state.account_flow_type.upsert(row_uuid, |table| table.company_branch = r)
            }
            Resource::TableAccountFlowTypeFieldInflowType(r) => {
                state.account_flow_type.upsert(row_uuid, |table| table.inflow_type = r)
            }
            Resource::TableAccountFlowTypeFieldOutflowType(r) => {
                state.account_flow_type.upsert(row_uuid, |table| table.outflow_type = r)
            }
            Resource::TableSharedEntryFieldWriter(r) => {
                state.shared_entry.upsert(row_uuid, |table| table.writer = r)
            }
            Resource::TableSharedEntryFieldNotes(r) => {
                state.shared_entry.upsert(row_uuid, |table| table.notes = r)
            }
            Resource::TableEntryFieldWriter(r) => {
                state.entry.upsert(row_uuid, |table| table.writer = r)
            }
            Resource::TableEntryFieldTime(r) => {
                state.entry.upsert(row_uuid, |table| table.time = r)
            }
            Resource::TableEntryFieldSharedEntryId(r) => {
                state.entry.upsert(row_uuid, |table| table.shared_entry_id = r)
            }
            Resource::TableSingleEntryFieldDoubleEntry(r) => {
                state.single_entry.upsert(row_uuid, |table| table.double_entry = r)
            }
            Resource::TableSingleEntryFieldEntry(r) => {
                state.single_entry.upsert(row_uuid, |table| table.entry = r)
            }
            Resource::TableSingleEntryFieldAccount(r) => {
                state.single_entry.upsert(row_uuid, |table| table.account = r)
            }
            Resource::TableSingleEntryFieldIsDebit(r) => {
                state.single_entry.upsert(row_uuid, |table| table.is_debit = r)
            }
            Resource::TableSingleEntryFieldCostOutFlowType(r) => {
                state.single_entry.upsert(row_uuid, |table| table.cost_out_flow_type = r)
            }
            Resource::TableSingleEntryFieldQuantity(r) => {
                state.single_entry.upsert(row_uuid, |table| table.quantity = r)
            }
            Resource::TableSingleEntryFieldAmount(r) => {
                state.single_entry.upsert(row_uuid, |table| table.amount = r)
            }
        }
    }
}
