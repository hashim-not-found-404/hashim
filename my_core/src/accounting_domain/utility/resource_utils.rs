use crate::accounting_domain::utility::types;
use crate::utility::utils::MyUpSert;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct User {
    pub(crate) id: String,
    pub(crate) name: Option<String>,
    pub(crate) password: String,
}

#[derive(Default)]
pub(crate) struct Company {
    pub(crate) currency: types::Currency,
    pub(crate) name: String,
}

#[derive(Default)]
pub(crate) struct AccessControlForCompany {
    pub(crate) data_group: types::UuidType,
    pub(crate) role: types::Role,
    pub(crate) user_: types::UuidType,
}

#[derive(Default)]
pub(crate) struct CompanyBranch {
    pub(crate) company_belong: types::UuidType,
    pub(crate) currency: types::Currency,
    pub(crate) location: types::Location,
    pub(crate) name: String,
}

#[derive(Default)]
pub(crate) struct AccessControlForCompanyBranch {
    pub(crate) data_group: types::UuidType,
    pub(crate) role: types::Role,
    pub(crate) user_: types::UuidType,
}

#[derive(Default)]
pub(crate) struct Account {
    pub(crate) company_belong: types::UuidType,
    pub(crate) is_debit: bool,
    pub(crate) is_permanent_account: bool,
    pub(crate) name: String,
    pub(crate) notes: String,
    pub(crate) unit_of_measurement_of_quantity: String,
}

#[derive(Default)]
pub(crate) struct StateOfPendingTxn {
    pub(crate) access_control_for_company: HashMap<types::UuidType, AccessControlForCompany>,
    pub(crate) access_control_for_company_branch:
        HashMap<types::UuidType, AccessControlForCompanyBranch>,
    pub(crate) account: HashMap<types::UuidType, Account>,
    pub(crate) company: HashMap<types::UuidType, Company>,
    pub(crate) company_branch: HashMap<types::UuidType, CompanyBranch>,
    pub(crate) user: HashMap<types::UuidType, User>,
}

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
    TableCompanyBranchFieldCompanyBelong,
    TableCompanyBranchFieldCurrency,
    TableCompanyBranchFieldLocation,
    TableCompanyBranchFieldName,
    TableCompanyFieldCurrency,
    TableCompanyFieldName,
    TableUserFieldId,
    TableUserFieldName,
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
    TableAccountFieldNotes(String),
    TableAccountFieldUnitOfMeasurementOfQuantity(String),
    TableCompanyBranchFieldCompanyBelong(types::UuidType),
    TableCompanyBranchFieldCurrency(types::Currency),
    TableCompanyBranchFieldLocation(types::Location),
    TableCompanyBranchFieldName(String),
    TableCompanyFieldCurrency(types::Currency),
    TableCompanyFieldName(String),
    TableUserFieldId(String),
    TableUserFieldName(String),
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
        }
    }
}
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
            Resource::TableCompanyBranchFieldName(r) => state
                .company_branch
                .upsert(row_uuid, |table| table.name = r),
            Resource::TableCompanyBranchFieldCompanyBelong(r) => state
                .company_branch
                .upsert(row_uuid, |table| table.company_belong = r),
            Resource::TableCompanyBranchFieldCurrency(r) => state
                .company_branch
                .upsert(row_uuid, |table| table.currency = r),
            Resource::TableCompanyBranchFieldLocation(r) => state
                .company_branch
                .upsert(row_uuid, |table| table.location = r),
            Resource::TableCompanyFieldCurrency(r) => {
                state.company.upsert(row_uuid, |table| table.currency = r)
            }
            Resource::TableAccessControlForCompanyFieldRole(r) => state
                .access_control_for_company
                .upsert(row_uuid, |table| table.role = r),
            Resource::TableAccessControlForCompanyFieldUser(r) => state
                .access_control_for_company
                .upsert(row_uuid, |table| table.user_ = r),
            Resource::TableAccessControlForCompanyFieldDataGroup(r) => state
                .access_control_for_company
                .upsert(row_uuid, |table| table.data_group = r),
            Resource::TableAccessControlForCompanyBranchFieldRole(r) => state
                .access_control_for_company_branch
                .upsert(row_uuid, |table| table.role = r),
            Resource::TableAccessControlForCompanyBranchFieldUser(r) => state
                .access_control_for_company_branch
                .upsert(row_uuid, |table| table.user_ = r),
            Resource::TableAccessControlForCompanyBranchFieldDataGroup(r) => state
                .access_control_for_company_branch
                .upsert(row_uuid, |table| table.data_group = r),
            Resource::TableAccountFieldCompanyBelong(r) => state
                .account
                .upsert(row_uuid, |table| table.company_belong = r),
            Resource::TableAccountFieldIsDebit(r) => {
                state.account.upsert(row_uuid, |table| table.is_debit = r)
            }
            Resource::TableAccountFieldIsPermanentAccount(r) => state
                .account
                .upsert(row_uuid, |table| table.is_permanent_account = r),
            Resource::TableAccountFieldName(r) => {
                state.account.upsert(row_uuid, |table| table.name = r)
            }
            Resource::TableAccountFieldNotes(r) => {
                state.account.upsert(row_uuid, |table| table.notes = r)
            }
            Resource::TableAccountFieldUnitOfMeasurementOfQuantity(r) => state
                .account
                .upsert(row_uuid, |table| table.unit_of_measurement_of_quantity = r),
        }
    }
}
