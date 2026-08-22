use crate::domain::utility::types::Currency;
use crate::domain::utility::types::JsonWebTokenType;
use crate::domain::utility::types::Location;
use crate::domain::utility::types::Role;
use crate::domain::utility::uuid::Account;
use crate::domain::utility::uuid::AccountForBranch;
use crate::domain::utility::uuid::Branch;
use crate::domain::utility::uuid::Company;
use crate::domain::utility::uuid::SharedEntry;
use crate::domain::utility::uuid::User;
use crate::domain::utility::uuid::UuidType;
use accounting_engine::accounting_stuff::InFlowType;
use accounting_engine::accounting_stuff::InventoryRecord;
use accounting_engine::accounting_stuff::OutFlowType;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Resource {
    Jwt(JsonWebTokenType),

    TableAccessControlForCompanyBranchFieldDataGroup(Branch),
    TableAccessControlForCompanyBranchFieldRole(Role),
    TableAccessControlForCompanyBranchFieldUser(User),
    TableAccessControlForCompanyFieldDataGroup(Company),
    TableAccessControlForCompanyFieldRole(Role),
    TableAccessControlForCompanyFieldUser(User),
    TableAccountFieldCompanyBelong(Company),
    TableAccountFieldIsDebit(bool),
    TableAccountFieldIsPermanentAccount(bool),
    TableAccountFieldName(String),
    TableAccountFieldNotes(Option<String>),
    TableAccountFieldUnitOfMeasurementOfQuantity(String),
    TableAccountFieldInventory(Vec<InventoryRecord>),
    TableAccountFlowTypeFieldAccount(Account),
    TableAccountFlowTypeFieldCompanyBranch(Branch),
    TableAccountFlowTypeFieldInflowType(InFlowType),
    TableAccountFlowTypeFieldOutflowType(OutFlowType),
    TableCompanyBranchFieldCompanyBelong(Company),
    TableCompanyBranchFieldCurrency(Currency),
    TableCompanyBranchFieldLocation(Location),
    TableCompanyBranchFieldName(String),
    TableCompanyFieldCurrency(Currency),
    TableCompanyFieldName(String),
    TableUserFieldId(String),
    TableUserFieldName(String),
    TableSharedEntryFieldWriter(SharedEntry),
    TableSharedEntryFieldNotes(Option<String>),
    TableEntryFieldWriter(User),
    TableEntryFieldTime(u64),
    TableEntryFieldSharedEntryId(SharedEntry),
    TableSingleEntryFieldDoubleEntry(u32),
    TableSingleEntryFieldEntry(UuidType),
    TableSingleEntryFieldAccount(AccountForBranch),
    TableSingleEntryFieldIsDebit(bool),
    TableSingleEntryFieldCostOutFlowType(OutFlowType),
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResourceInfo {
    pub row_uuid: UuidType,
    pub resource: Resource,
}
