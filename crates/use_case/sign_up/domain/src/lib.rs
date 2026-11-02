use serde::Deserialize;
use serde::Serialize;
use use_case_prelude_domain::*;

#[serde(name = "sign_up")]
impl TraitOperationDTOInput for Input {}

#[serde(name = "sign_up")]
impl TraitOperationDTOOk for Ok {}

#[serde(name = "sign_up")]
impl TraitOperationDTOError for Error {}

const RESOURCES_NAME_TO_POKE: &[ResourceName] = &[new_resource_name("users")];

impl TraitOperationClientOk for Ok {
    fn subs_to_poke(&self) -> &'static [ResourceName] {
        RESOURCES_NAME_TO_POKE
    }
}

impl TraitOperationClientError for Error {
    fn subs_to_poke(&self) -> &'static [ResourceName] {
        RESOURCES_NAME_TO_POKE
    }
}

impl TraitOperationClientInput for Input {
    fn user_uuid(&self) -> Option<[u8; 16]> {
        None
    }

    fn state_less_check(&self) -> Option<TypeOperationClientError> {
        let errr = self.state_less_check();
        if errr.is_there_error() {
            Some(Box::new(errr))
        } else {
            None
        }
    }
}

pub type MyResult = Result<Ok, Error>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub user_uuid: UserUuid,
    pub name: Option<String>,
    pub user_id: String,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Ok {
    pub new_uuid: UserUuid,
    pub user_id: String,
    pub user_name: Option<String>,
    pub hashed_password: String,
    pub jwt: Option<JsonWebTokenType>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct Error {
    pub new_uuid: Option<RowIdError>,
    pub user_id: Option<UserIdError>,
    pub name: Option<UserNameError>,
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
pub enum UserIdError {
    Duplicated,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum UserNameError {
    Empty,
}

impl Input {
    pub fn state_less_check(&self) -> Error {
        let mut errr = Error::default();

        if !Id::validate(&self.user_uuid) {
            errr.new_uuid = Some(RowIdError::Invalid);
        }

        errr
    }

    pub async fn state_full_check<Db: DatabaseRead<Input = ReadInput, Output = ReadOutput>>(
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

    pub fn state_full_operation<Jwt: JWT<UserUuid>, Auth: HashedPassword>(&self, jwt: &Jwt) -> Ok {
        let hashed_password = Auth::sign_up(&self.password);
        let jwt = jwt.sign(&self.user_uuid);

        Ok {
            new_uuid: self.user_uuid.clone(),
            user_id: self.user_id.clone(),
            user_name: self.name.clone(),
            hashed_password,
            jwt: Some(jwt),
        }
    }
}
