use crate::domain::utility::types;
use crate::domain::utility::types::Currency;
use crate::domain::utility::types::Location;
use crate::domain::utility::types::MarkerMyErrorTrait;
use crate::domain::utility::types::Role;
use crate::domain::utility::types::RowId;
use crate::domain::utility::types::RowIdError;
use crate::domain::utility::types::UserUuidError;
use crate::domain::utility::uuid::Branch;
use crate::domain::utility::uuid::Company;
use crate::domain::utility::uuid::User;
use crate::utility::traits::DynamicError;
use serde::Deserialize;
use serde::Serialize;

pub type MyResult = Result<Ok, Error>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub(crate) user_uuid:      User,
    pub(crate) new_uuid:       Branch,
    pub(crate) company_belong: Company,
    pub(crate) branch_name:    String,
    pub(crate) location:       Location,
    pub(crate) currency:       Currency,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Ok {
    pub new_uuid:       Branch,
    pub branch_name:    String,
    pub company_belong: Company,
    pub user_uuid:      User,
    pub currency:       Currency,
    pub location:       Location,
    pub role:           Role,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct Error {
    pub(crate) user_uuid:      Option<UserUuidError>,
    pub(crate) new_uuid:       Option<RowIdError>,
    pub(crate) company_belong: Option<CompanyBelongError>,
    pub(crate) branch_name:    Option<BranchNameError>,
    pub(crate) location:       Option<LocationError>,
}

impl MarkerMyErrorTrait for Error {}

pub struct ReadInput {
    pub user_uuid:      User,
    pub new_uuid:       Branch,
    pub company_belong: Company,
    pub branch_name:    String,
}

pub struct ReadOutput {
    pub user_roles:          Vec<Role>,
    pub is_new_uuid_used:    bool,
    pub is_company_exist:    bool,
    pub is_branch_name_used: bool,
}

pub trait DatabaseRead: types::DatabaseRead<Input = ReadInput, Output = ReadOutput> {}

// utility types

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) enum CompanyBelongError {
    IdInWrongFormat,
    NotExist,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) enum BranchNameError {
    Duplicated,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) enum LocationError {
    Invalid,
}

impl Input {
    pub(crate) fn state_less_check<Id: RowId>(&self) -> Error {
        let mut errr = Error::default();

        if !Id::validate(&self.new_uuid) {
            errr.new_uuid = Some(RowIdError::Invalid);
        }

        if !Id::validate(&self.user_uuid) {
            errr.user_uuid = Some(UserUuidError::Invalid);
        }

        if !Id::validate(&self.company_belong) {
            errr.company_belong = Some(CompanyBelongError::IdInWrongFormat);
        }

        errr
    }

    pub(crate) async fn state_full_check<Db: DatabaseRead>(
        &self,
        db: &mut Db::Db<'_>,
    ) -> Result<Error, DynamicError> {
        let read_output = Db::read(db, &ReadInput {
            user_uuid:      self.user_uuid.clone(),
            new_uuid:       self.new_uuid.clone(),
            company_belong: self.company_belong.clone(),
            branch_name:    self.branch_name.clone(),
        })
        .await?;

        let mut errr = Error::default();

        if !Role::has_any(&read_output.user_roles, &[Role::Manager, Role::CoManager]) {
            errr.user_uuid = Some(UserUuidError::YouDontHavePermissionToDoThat);
        }

        if read_output.is_new_uuid_used {
            errr.new_uuid = Some(RowIdError::Duplicated);
        }

        if !read_output.is_company_exist {
            errr.company_belong = Some(CompanyBelongError::NotExist);
        }

        if read_output.is_branch_name_used {
            errr.branch_name = Some(BranchNameError::Duplicated);
        }

        if !self.location.is_valid() {
            errr.location = Some(LocationError::Invalid);
        }

        Ok(errr)
    }

    pub(crate) fn state_less_operation(&self) -> Ok {
        const ROLE: Role = Role::CoManager;

        Ok {
            new_uuid:       self.new_uuid.clone(),
            branch_name:    self.branch_name.clone(),
            company_belong: self.company_belong.clone(),
            user_uuid:      self.user_uuid.clone(),
            currency:       self.currency.clone(),
            location:       self.location.clone(),
            role:           ROLE,
        }
    }
}
