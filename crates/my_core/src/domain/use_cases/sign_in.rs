use crate::domain::utility::types;
use crate::domain::utility::types::HashedPassword;
use crate::domain::utility::types::JWT;
use crate::domain::utility::types::JsonWebTokenType;
use crate::domain::utility::types::MarkerMyErrorTrait;
use crate::domain::utility::uuid::User;
use crate::utility::traits::DynamicError;
use serde::Deserialize;
use serde::Serialize;

pub type MyResult = Result<Ok, Error>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub(crate) user_id:  String,
    pub(crate) password: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Ok {
    pub user_uuid:  User,
    pub user_id:    String,
    pub user_name:  Option<String>,
    pub(crate) jwt: JsonWebTokenType,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct Error {
    pub(crate) user_id:  Option<UserIdError>,
    pub(crate) password: Option<PasswordError>,
}

impl MarkerMyErrorTrait for Error {}

pub struct ReadInput {
    pub user_id: String,
}

pub struct ReadOutput {
    pub user_rowid_and_password_hash_and_name: Option<(User, String, Option<String>)>,
}

pub trait DatabaseRead: types::DatabaseRead<Input = ReadInput, Output = ReadOutput> {}

// utility types

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) enum UserIdError {
    NotExist,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) enum PasswordError {
    WrongPassword,
}

impl Input {
    pub(crate) async fn state_full_check<Auth: HashedPassword, Jwt: JWT, Db: DatabaseRead>(
        &self,
        jwt: &Jwt,
        db: &mut Db::Db<'_>,
    ) -> Result<MyResult, DynamicError> {
        let read_output = Db::read(db, &ReadInput {
            user_id: self.user_id.clone(),
        })
        .await?;

        let mut errr = Error::default();

        let Some((user_rowid, password_hash, user_name)) =
            &read_output.user_rowid_and_password_hash_and_name
        else {
            errr.user_id = Some(UserIdError::NotExist);
            return Ok(Err(errr));
        };

        if Auth::sign_in(&self.password, password_hash) {
            Ok(Ok(Ok {
                user_uuid: user_rowid.clone(),
                jwt:       jwt.sign(user_rowid),
                user_id:   self.user_id.clone(),
                user_name: user_name.clone(),
            }))
        } else {
            errr.password = Some(PasswordError::WrongPassword);
            Ok(Err(errr))
        }
    }

    pub(crate) fn state_full_operation(
        &self,
        jwt: &JsonWebTokenType,
        user_uuid: &User,
        user_name: Option<&String>,
    ) -> Ok {
        Ok {
            user_uuid: user_uuid.clone(),
            user_id:   self.user_id.clone(),
            user_name: user_name.cloned(),
            jwt:       jwt.clone(),
        }
    }
}
