use anyhow::Result;
use infrastructure::row_id::Id;
use infrastructure::row_id::RowId;
use kernel::make_type;
use kernel::new_types::CompanyUuid;
use kernel::new_types::UserUuid;
use kernel::new_types::UuidType;
use kernel::types::DatabaseRead;
use kernel::types::MarkerMyErrorTrait;
use kernel::types::Role;
use kernel::types::RowIdError;
use kernel::types::UserUuidError;
use serde::Deserialize;
use serde::Serialize;
use typetag::serde;
use utility::dtos::TraitOperationDTOError;
use utility::dtos::TraitOperationDTOInput;
use utility::dtos::TraitOperationDTOOk;

#[serde(name = "get_all_journals")]
impl TraitOperationDTOInput for Input {}

#[serde(name = "get_all_journals")]
impl TraitOperationDTOOk for Ok {}

#[serde(name = "get_all_journals")]
impl TraitOperationDTOError for Error {}

pub type MyResult = Result<Ok, Error>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub user_uuid: UserUuid,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Ok {
    pub data: Vec<Data>,
}

make_type!(UuidType, JournalUuid);

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Data {
    pub row_uuid: JournalUuid,
    pub company_uuid: CompanyUuid,
    pub name: String,
    pub is_it_contain_quantity: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct Error {
    pub(crate) user_uuid: Option<UserUuidError>,
}

impl MarkerMyErrorTrait for Error {}

pub struct ReadInput {
    pub user_uuid: UserUuid,
}

pub struct ReadOutput {
    pub user_roles: Vec<(CompanyUuid, Role)>,
    pub data: Vec<Data>,
}

impl Input {
    pub(crate) fn state_less_check(&self) -> Error {
        let mut errr = Error::default();

        if !Id::validate(&self.user_uuid) {
            errr.user_uuid = Some(UserUuidError::Invalid);
        }

        if !Id::validate(&self.company_uuid) {
            errr.company_uuid = Some(RowIdError::Invalid);
        }

        errr
    }

    pub(crate) async fn state_full_operation<
        Db: DatabaseRead<Input = ReadInput, Output = ReadOutput>,
    >(
        &self,
        db: &mut Db::Db<'_>,
    ) -> Result<MyResult> {
        let read_output = Db::read(
            db,
            &ReadInput {
                user_uuid: self.user_uuid.clone(),
            },
        )
        .await?;

        if read_output.user_roles.is_empty() {
            return Ok(Err(Error {
                user_uuid: Some(UserUuidError::YouDontHavePermissionToDoThat),
            }));
        }

        Ok(Ok(Ok {
            data: read_output.data,
        }))
    }
}
