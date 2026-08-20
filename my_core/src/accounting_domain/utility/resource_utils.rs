use crate::accounting_domain::utility::types;
use accounting_engine::accounting_stuff;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Subscribe {
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Resource {
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
    TableAccountFieldNotes(Option<String>),
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ResourceInfo {
    pub row_uuid: types::UuidType,
    pub resource: Resource,
}
