use anyhow::Result;
use infrastructure::row_id::Id;
use infrastructure::row_id::RowId;
use kernel::new_types::BranchUuid;
use kernel::new_types::CompanyUuid;
use kernel::new_types::UserUuid;
use kernel::types::Currency;
use kernel::types::DatabaseRead;
use kernel::types::MarkerMyErrorTrait;
use kernel::types::Role;
use kernel::types::UserUuidError;
use serde::Deserialize;
use serde::Serialize;
use typetag::serde;
use utility::dtos::TraitOperationDTOError;
use utility::dtos::TraitOperationDTOInput;
use utility::dtos::TraitOperationDTOOk;

#[serde(name = "get_companies_and_branches")]
impl TraitOperationDTOInput for Input {}

#[serde(name = "get_companies_and_branches")]
impl TraitOperationDTOOk for Ok {}

#[serde(name = "get_companies_and_branches")]
impl TraitOperationDTOError for Error {}

pub type MyResult = Result<Ok, Error>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub user_uuid: UserUuid,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Ok {
    pub user_uuid: UserUuid,
    pub companies: Vec<CompanyWithBranches>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CompanyWithBranches {
    pub uuid: CompanyUuid,
    pub name: String,
    pub currency: Currency,
    pub roles: Vec<Role>,
    pub branches: Vec<BranchInfo>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct BranchInfo {
    pub uuid: BranchUuid,
    pub name: String,
    pub roles: Vec<Role>,
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
    pub companies: Vec<CompanyWithBranches>,
}

impl Input {
    pub(crate) fn state_less_check(&self) -> Error {
        let mut errr = Error::default();

        if !Id::validate(&self.user_uuid) {
            errr.user_uuid = Some(UserUuidError::Invalid);
        }

        errr
    }

    pub(crate) async fn state_full_operation<
        Db: DatabaseRead<Input = ReadInput, Output = ReadOutput>,
    >(
        &self,
        db: &mut Db::Db<'_>,
    ) -> Result<Ok> {
        let read_output = Db::read(
            db,
            &ReadInput {
                user_uuid: self.user_uuid.clone(),
            },
        )
        .await?;

        Ok(Ok {
            user_uuid: self.user_uuid.clone(),
            companies: read_output.companies,
        })
    }
}
