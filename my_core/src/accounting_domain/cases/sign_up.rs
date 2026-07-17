use crate::accounting_domain::cases::utility::types;
use serde::{Deserialize, Serialize};

pub type MyResult = Result<Ok, Error>;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Input {
    pub(crate) new_uuid: types::UuidType,
    pub(crate) name: Option<String>,
    pub(crate) user_id: String,
    pub(crate) password: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Ok {
    pub new_uuid: types::UuidType,
    pub user_id: String,
    pub user_name: Option<String>,
    pub hashed_password: String,
    pub(crate) jwt: types::JsonWebTokenType,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct Error {
    pub(crate) new_uuid: Option<types::RowIdError>,
    pub(crate) user_id: Option<UserIdError>,
    pub(crate) name: Option<String>,
}

impl types::MyErrorTrait for Error {}

// utility types

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub(crate) enum UserIdError {
    Duplicated,
}

impl Input {
    pub(crate) fn state_less_check<Id: types::RowId>(&self) -> Error {
        let mut errr = Error::default();

        if !Id::validate(&self.new_uuid) {
            errr.new_uuid = Some(types::RowIdError::Invalid);
        }

        errr
    }

    pub(crate) fn state_full_check<Id: types::RowId>(
        &self,
        is_new_uuid_exist: bool,
        is_user_id_exist: bool,
    ) -> Error {
        let mut errr = Error::default();

        if is_new_uuid_exist {
            errr.new_uuid = Some(types::RowIdError::Duplicated);
        }

        if is_user_id_exist {
            errr.user_id = Some(UserIdError::Duplicated);
        }

        errr
    }

    pub(crate) fn state_full_operation<Auth: types::HashedPassword, Jwt: types::JWT>(
        &self,
        jwt: &Jwt,
    ) -> Ok {
        let hashed_password = Auth::sign_up(&self.password);
        let jwt = jwt.sign(&self.new_uuid);

        return Ok {
            new_uuid: self.new_uuid.clone(),
            user_id: self.user_id.clone(),
            user_name: self.name.clone(),
            hashed_password,
            jwt,
        };
    }
}
