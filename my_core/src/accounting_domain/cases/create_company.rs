use crate::accounting_domain::cases::utility::types;
use serde::{Deserialize, Serialize};

pub type MyResult = Result<Ok, Error>;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Input {
    pub(crate) user_uuid: types::UuidType,
    pub(crate) new_uuid: types::UuidType,
    pub(crate) company_name: String,
    pub(crate) currency: types::Currency,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Ok {
    pub new_uuid: types::UuidType,
    pub company_name: String,
    pub currency: types::Currency,
    pub user_uuid: types::UuidType,
    pub role: types::Role,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct Error {
    pub(crate) user_uuid: Option<types::UserUuidError>,
    pub(crate) new_uuid: Option<types::RowIdError>,
}

impl types::MyErrorTrait for Error {}

impl Input {
    pub(crate) fn state_less_check<Id: types::RowId>(&self) -> Error {
        let mut errr = Error::default();

        if !Id::validate(&self.new_uuid) {
            errr.new_uuid = Some(types::RowIdError::Invalid);
        }

        if !Id::validate(&self.user_uuid) {
            errr.user_uuid = Some(types::UserUuidError::Invalid);
        }
        errr
    }

    pub(crate) fn state_full_check<Id: types::RowId>(&self, is_new_uuid_used: bool) -> Error {
        let mut errr = Error::default();
        if is_new_uuid_used {
            errr.new_uuid = Some(types::RowIdError::Duplicated);
        }
        errr
    }

    pub(crate) fn state_less_operation(&self) -> Ok {
        const ROLE: types::Role = types::Role::Manager;

        Ok {
            new_uuid: self.new_uuid.clone(),
            company_name: self.company_name.clone(),
            currency: self.currency.clone(),
            user_uuid: self.user_uuid.clone(),
            role: ROLE,
        }
    }
}
