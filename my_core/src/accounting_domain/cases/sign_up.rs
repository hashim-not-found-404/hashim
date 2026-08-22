use crate::accounting_domain::utility::types;
use crate::accounting_domain::utility::types::HashedPassword;
use crate::accounting_domain::utility::types::JWT;
use crate::accounting_domain::utility::types::JsonWebTokenType;
use crate::accounting_domain::utility::types::MarkerMyErrorTrait;
use crate::accounting_domain::utility::types::RowId;
use crate::accounting_domain::utility::types::RowIdError;
use crate::accounting_domain::utility::types::UuidType;
use crate::utility::traits::DynamicError;
use serde::Deserialize;
use serde::Serialize;

pub type MyResult = Result<Ok, Error>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub(crate) new_uuid: UuidType,
    pub(crate) name:     Option<String>,
    pub(crate) user_id:  String,
    pub(crate) password: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Ok {
    pub new_uuid:        UuidType,
    pub user_id:         String,
    pub user_name:       Option<String>,
    pub hashed_password: String,
    pub(crate) jwt:      JsonWebTokenType,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct Error {
    pub(crate) new_uuid: Option<RowIdError>,
    pub(crate) user_id:  Option<UserIdError>,
    pub(crate) name:     Option<String>,
}

impl MarkerMyErrorTrait for Error {}

pub struct ReadInput {
    pub new_uuid: UuidType,
    pub user_id:  String,
}

pub struct ReadOutput {
    pub is_new_uuid_exist: bool,
    pub is_user_id_exist:  bool,
}

pub trait DatabaseRead: types::DatabaseRead<Input = ReadInput, Output = ReadOutput> {}

// utility types

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) enum UserIdError {
    Duplicated,
}

impl Input {
    pub(crate) fn state_less_check<Id: RowId>(&self) -> Error {
        let mut errr = Error::default();

        if !Id::validate(&self.new_uuid) {
            errr.new_uuid = Some(RowIdError::Invalid);
        }

        errr
    }

    pub(crate) async fn state_full_check<Db: DatabaseRead>(
        &self,
        db: &mut Db::Db<'_>,
    ) -> Result<Error, DynamicError> {
        let read_output = Db::read(db, &ReadInput {
            new_uuid: self.new_uuid.clone(),
            user_id:  self.user_id.clone(),
        })
        .await?;

        let mut errr = Error::default();

        if read_output.is_new_uuid_exist {
            errr.new_uuid = Some(RowIdError::Duplicated);
        }

        if read_output.is_user_id_exist {
            errr.user_id = Some(UserIdError::Duplicated);
        }

        Ok(errr)
    }

    pub(crate) fn state_full_operation<Auth: HashedPassword, Jwt: JWT>(&self, jwt: &Jwt) -> Ok {
        let hashed_password = Auth::sign_up(&self.password);
        let jwt = jwt.sign(&self.new_uuid);

        Ok {
            new_uuid: self.new_uuid.clone(),
            user_id: self.user_id.clone(),
            user_name: self.name.clone(),
            hashed_password,
            jwt,
        }
    }
}
