use anyhow::Result;
use infrastructure::authentication::Auth;
use infrastructure::authentication::HashedPassword;
use infrastructure::jwt::JWT;
use infrastructure::jwt::JsonWebTokenType;
use infrastructure::row_id::Id;
use infrastructure::row_id::RowId;
use kernel::new_types::UserUuid;
use kernel::types::DatabaseRead;
use kernel::types::MarkerMyErrorTrait;
use kernel::types::RowIdError;
use serde::Deserialize;
use serde::Serialize;
use typetag::serde;
use utility::dtos::TraitOperationDTOError;
use utility::dtos::TraitOperationDTOInput;
use utility::dtos::TraitOperationDTOOk;

#[serde]
impl TraitOperationDTOInput for Input {}

#[serde]
impl TraitOperationDTOOk for Ok {}

#[serde]
impl TraitOperationDTOError for Error {}

pub type MyResult = Result<Ok, Error>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub(crate) user_uuid: UserUuid,
    pub(crate) name: Option<String>,
    pub(crate) user_id: String,
    pub(crate) password: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Ok {
    pub new_uuid: UserUuid,
    pub user_id: String,
    pub user_name: Option<String>,
    pub hashed_password: String,
    pub jwt: JsonWebTokenType,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct Error {
    pub(crate) new_uuid: Option<RowIdError>,
    pub(crate) user_id: Option<UserIdError>,
    pub(crate) name: Option<String>,
}

impl MarkerMyErrorTrait for Error {}

pub struct ReadInput {
    pub new_uuid: UserUuid,
    pub user_id: String,
}

pub struct ReadOutput {
    pub is_new_uuid_exist: bool,
    pub is_user_id_exist: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) enum UserIdError {
    Duplicated,
}

impl Input {
    pub(crate) fn state_less_check(&self) -> Error {
        let mut errr = Error::default();

        if !Id::validate(&self.user_uuid) {
            errr.new_uuid = Some(RowIdError::Invalid);
        }

        errr
    }

    pub(crate) async fn state_full_check<
        Db: DatabaseRead<Input = ReadInput, Output = ReadOutput>,
    >(
        &self,
        db: &mut Db::Db<'_>,
    ) -> Result<Error> {
        let read_output = Db::read(
            db,
            &ReadInput {
                new_uuid: self.user_uuid.clone(),
                user_id: self.user_id.clone(),
            },
        )
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

    pub(crate) fn state_full_operation<Jwt: JWT>(&self, jwt: &Jwt) -> Ok {
        let hashed_password = Auth::sign_up(&self.password);
        let jwt = jwt.sign(&self.user_uuid);

        Ok {
            new_uuid: self.user_uuid.clone(),
            user_id: self.user_id.clone(),
            user_name: self.name.clone(),
            hashed_password,
            jwt,
        }
    }
}
