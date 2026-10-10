use anyhow::Result;
use infrastructure::row_id::Id;
use infrastructure::row_id::RowId;
use kernel::new_types::BranchUuid;
use kernel::new_types::CompanyUuid;
use kernel::new_types::UserUuid;
use kernel::types::Currency;
use kernel::types::DatabaseRead;
use kernel::types::Location;
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

#[serde(name = "create_branch")]
impl TraitOperationDTOInput for Input {}

#[serde(name = "create_branch")]
impl TraitOperationDTOOk for Ok {}

#[serde(name = "create_branch")]
impl TraitOperationDTOError for Error {}

pub type MyResult = Result<Ok, Error>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub user_uuid: UserUuid,
    pub new_uuid: BranchUuid,
    pub company_belong: CompanyUuid,
    pub branch_name: String,
    pub currency: Currency,
    pub location: Location,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Ok {
    pub new_uuid: BranchUuid,
    pub company_belong: CompanyUuid,
    pub user_uuid: UserUuid,
    pub branch_name: String,
    pub currency: Currency,
    pub location: Location,
    pub role: Role,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct Error {
    pub(crate) user_uuid: Option<UserUuidError>,
    pub(crate) new_uuid: Option<RowIdError>,
    pub(crate) company_belong: Option<CompanyBelongError>,
    pub(crate) branch_name: Option<BranchNameError>,
    pub(crate) location: Option<LocationError>,
}

impl MarkerMyErrorTrait for Error {}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum CompanyBelongError {
    NotExist,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum BranchNameError {
    Empty,
    Duplicated,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum LocationError {
    Invalid,
}

pub struct ReadInput {
    pub user_uuid: UserUuid,
    pub new_uuid: BranchUuid,
    pub company_belong: CompanyUuid,
    pub branch_name: String,
}

pub struct ReadOutput {
    pub user_roles: Vec<Role>,
    pub is_new_uuid_used: bool,
    pub is_company_exist: bool,
    pub is_branch_name_used: bool,
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

        if !Id::validate(&self.company_belong) {
            errr.company_belong = Some(CompanyBelongError::NotExist);
        }

        if self.branch_name.trim().is_empty() {
            errr.branch_name = Some(BranchNameError::Empty);
        }

        if !self.location.is_valid() {
            errr.location = Some(LocationError::Invalid);
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
                user_uuid: self.user_uuid.clone(),
                new_uuid: self.new_uuid.clone(),
                company_belong: self.company_belong.clone(),
                branch_name: self.branch_name.clone(),
            },
        )
        .await?;

        let mut errr = Error::default();

        if read_output.is_new_uuid_used {
            errr.new_uuid = Some(RowIdError::Duplicated);
        }

        if !read_output.is_company_exist {
            errr.company_belong = Some(CompanyBelongError::NotExist);
        }

        if read_output.is_branch_name_used {
            errr.branch_name = Some(BranchNameError::Duplicated);
        }

        if !Role::has_any(&read_output.user_roles, &[Role::Manager, Role::CoManager]) {
            errr.user_uuid = Some(UserUuidError::YouDontHavePermissionToDoThat);
        }

        Ok(errr)
    }

    pub(crate) fn state_less_operation(&self) -> Ok {
        Ok {
            new_uuid: self.new_uuid.clone(),
            company_belong: self.company_belong.clone(),
            user_uuid: self.user_uuid.clone(),
            branch_name: self.branch_name.clone(),
            currency: self.currency.clone(),
            location: self.location.clone(),
            role: Role::CoManager,
        }
    }
}
