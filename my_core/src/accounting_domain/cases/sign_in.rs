use crate::accounting_domain::cases::utility::types;
use serde::{Deserialize, Serialize};

pub type MyResult = Result<Ok, Error>;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Input {
    pub(crate) user_id: String,
    pub(crate) password: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Ok {
    pub user_uuid: types::UuidType,
    pub user_id: String,
    pub user_name: Option<String>,
    pub(crate) jwt: types::JsonWebTokenType,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct Error {
    pub(crate) user_id: Option<UserIdError>,
    pub(crate) password: Option<PasswordError>,
}

impl types::MyErrorTrait for Error {}

// utility types

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub(crate) enum UserIdError {
    NotExist,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub(crate) enum PasswordError {
    WrongPassword,
}

impl Input {
    pub(crate) fn state_full_check<Auth: types::HashedPassword, Jwt: types::JWT>(
        &self,
        jwt: &Jwt,
        user_rowid_and_password_hash_and_name: &Option<(types::UuidType, String, Option<String>)>,
    ) -> MyResult {
        let mut errr = Error::default();

        let (user_rowid, password_hash, user_name) = match user_rowid_and_password_hash_and_name {
            Some((user_rowid, password_hash, user_name)) => (user_rowid, password_hash, user_name),
            None => {
                errr.user_id = Some(UserIdError::NotExist);
                return Err(errr);
            }
        };

        match Auth::sign_in(&self.password, password_hash) {
            true => {
                return Ok(Ok {
                    user_uuid: user_rowid.clone(),
                    jwt: jwt.sign(&user_rowid),
                    user_id: self.user_id.clone(),
                    user_name: user_name.clone(),
                });
            }
            false => {
                errr.password = Some(PasswordError::WrongPassword);
                return Err(errr);
            }
        };
    }

    pub(crate) fn state_full_operation(
        &self,
        jwt: &types::JsonWebTokenType,
        user_uuid: &types::UuidType,
        user_name: &Option<String>,
    ) -> Ok {
        return Ok {
            user_uuid: user_uuid.clone(),
            user_id: self.user_id.clone(),
            user_name: user_name.clone(),
            jwt: jwt.clone(),
        };
    }
}
