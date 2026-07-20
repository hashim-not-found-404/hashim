use crate::accounting_domain::cases::utility::types;
use serde::{Deserialize, Serialize};

pub type MyResult = Result<Ok, Error>;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Input {
    pub(crate) user_uuid: types::UuidType,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Ok {
    pub(crate) user_uuid: types::UuidType,
    pub(crate) data: Vec<AllCompaniesThatUserInWithRoles>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AllCompaniesThatUserInWithRoles {
    pub company_uuid: types::UuidType,
    pub company_name: String,
    pub company_currancy: types::Currency,
    pub user_roles: Vec<types::Role>,
    pub branches: Vec<AllBranchesThatUserInWithRoles>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AllBranchesThatUserInWithRoles {
    pub branch_uuid: types::UuidType,
    pub branch_name: String,
    pub branch_currancy: types::Currency,
    pub user_roles: Vec<types::Role>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct Error {
    pub(crate) user_uuid: Option<types::UserUuidError>,
}

impl types::MyErrorTrait for Error {}

pub struct ReadInput {
    pub user_uuid: types::UuidType,
}

pub struct ReadOutput {
    pub data: Vec<AllCompaniesThatUserInWithRoles>,
}

impl Input {
    pub(crate) fn state_less_check<Id: types::RowId>(&self) -> Error {
        let mut errr = Error::default();

        if !Id::validate(&self.user_uuid) {
            errr.user_uuid = Some(types::UserUuidError::Invalid);
        }

        errr
    }
}
