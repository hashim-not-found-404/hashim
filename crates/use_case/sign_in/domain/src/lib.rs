use serde::Deserialize;
use serde::Serialize;
use use_case_prelude_domain::*;

#[serde(name = "sign_in")]
impl TraitOperationDTOInput for Input {}

#[serde(name = "sign_in")]
impl TraitOperationDTOOk for Ok {}

#[serde(name = "sign_in")]
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
        None
    }
}

pub type MyResult = Result<Ok, Error>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub user_id: String,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Ok {
    pub user_uuid: UserUuid,
    pub user_id: String,
    pub user_name: Option<String>,
    pub jwt: JsonWebTokenType,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct Error {
    pub user_id: Option<UserIdError>,
    pub password: Option<PasswordError>,
}

impl MarkerMyErrorTrait for Error {}

pub struct ReadInput {
    pub user_id: String,
}

/// On the **server**, the `String` inside the tuple is the argon2 password hash.
/// On the **client cache**, the `String` inside the tuple is the stored JWT.
/// Both sides use the same wire-shape so that `DatabaseRead` is one trait.
pub struct ReadOutput {
    pub user_rowid_and_password_or_jwt_and_name: Option<(UserUuid, String, Option<String>)>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum UserIdError {
    NotExist,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum PasswordError {
    WrongPassword,
}

impl Input {
    /// Server only. Reads user by `user_id`, verifies password with argon2,
    /// returns `Ok` with a freshly signed JWT.
    pub async fn state_full_operation<
        Db: DatabaseRead<Input = ReadInput, Output = ReadOutput>,
        Jwt: JWT<UserUuid>,
        Auth: HashedPassword,
    >(
        &self,
        db: &mut Db::Db<'_>,
        jwt_signer: &Jwt,
    ) -> Result<MyResult> {
        let read_output = Db::read(
            db,
            &ReadInput {
                user_id: self.user_id.clone(),
            },
        )
        .await?;

        let Some((user_uuid, password_hash, user_name)) =
            read_output.user_rowid_and_password_or_jwt_and_name
        else {
            return Ok(Err(Error {
                user_id: Some(UserIdError::NotExist),
                password: None,
            }));
        };

        if !Auth::sign_in(&self.password, &password_hash) {
            return Ok(Err(Error {
                user_id: None,
                password: Some(PasswordError::WrongPassword),
            }));
        }

        let jwt = jwt_signer.sign(&user_uuid);

        Ok(Ok(Ok {
            user_uuid,
            user_id: self.user_id.clone(),
            user_name,
            jwt,
        }))
    }
}
