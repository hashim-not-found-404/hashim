use anyhow::Result;
use infrastructure::row_id::Id;
use infrastructure::row_id::RowId;
use kernel::new_types::CompanyUuid;
use kernel::new_types::UserUuid;
use kernel::types::Currency;
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

#[serde(name = "create_company")]
impl TraitOperationDTOInput for Input {}

#[serde(name = "create_company")]
impl TraitOperationDTOOk for Ok {}

#[serde(name = "create_company")]
impl TraitOperationDTOError for Error {}

pub type MyResult = Result<Ok, Error>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub user_uuid: UserUuid,
    pub new_uuid: CompanyUuid,
    pub company_name: String,
    pub currency: Currency,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Ok {
    pub new_uuid: CompanyUuid,
    pub company_name: String,
    pub currency: Currency,
    pub user_uuid: UserUuid,
    pub role: Role,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct Error {
    pub(crate) user_uuid: Option<UserUuidError>,
    pub(crate) new_uuid: Option<RowIdError>,
    pub(crate) company_name: Option<CompanyNameError>,
}

impl MarkerMyErrorTrait for Error {}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum CompanyNameError {
    Empty,
}

pub struct ReadInput {
    pub new_uuid: CompanyUuid,
}

pub struct ReadOutput {
    pub is_new_uuid_used: bool,
}

impl Input {
    pub(crate) fn state_less_check(&self) -> Error {
        let mut errr = Error::default();

        if !Id::validate(&self.new_uuid) {
            errr.new_uuid = Some(RowIdError::Invalid);
        }

        if !Id::validate(&self.user_uuid) {
            errr.user_uuid = Some(UserUuidError::Invalid);
        }

        if self.company_name.trim().is_empty() {
            errr.company_name = Some(CompanyNameError::Empty);
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
                new_uuid: self.new_uuid.clone(),
            },
        )
        .await?;

        let mut errr = Error::default();

        if read_output.is_new_uuid_used {
            errr.new_uuid = Some(RowIdError::Duplicated);
        }

        Ok(errr)
    }

    pub(crate) fn state_less_operation(&self) -> Ok {
        Ok {
            new_uuid: self.new_uuid.clone(),
            company_name: self.company_name.clone(),
            currency: self.currency.clone(),
            user_uuid: self.user_uuid.clone(),
            role: Role::Manager,
        }
    }
}
