use crate::accounting_domain::utility::types;
use crate::accounting_domain::utility::types::Currency;
use crate::accounting_domain::utility::types::MarkerMyErrorTrait;
use crate::accounting_domain::utility::types::Role;
use crate::accounting_domain::utility::types::RowId;
use crate::accounting_domain::utility::types::RowIdError;
use crate::accounting_domain::utility::types::UserUuidError;
use crate::accounting_domain::utility::uuid::Company;
use crate::accounting_domain::utility::uuid::User;
use crate::utility::traits::DynamicError;
use serde::Deserialize;
use serde::Serialize;

pub type MyResult = Result<Ok, Error>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub(crate) user_uuid:    User,
    pub(crate) new_uuid:     Company,
    pub(crate) company_name: String,
    pub(crate) currency:     Currency,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Ok {
    pub new_uuid:     Company,
    pub company_name: String,
    pub currency:     Currency,
    pub user_uuid:    User,
    pub role:         Role,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct Error {
    pub(crate) user_uuid: Option<UserUuidError>,
    pub(crate) new_uuid:  Option<RowIdError>,
}

impl MarkerMyErrorTrait for Error {}

pub struct ReadInput {
    pub new_uuid: Company,
}

pub struct ReadOutput {
    pub is_new_uuid_used: bool,
}

pub trait DatabaseRead: types::DatabaseRead<Input = ReadInput, Output = ReadOutput> {}

impl Input {
    pub(crate) fn state_less_check<Id: RowId>(&self) -> Error {
        let mut errr = Error::default();

        if !Id::validate(&self.new_uuid) {
            errr.new_uuid = Some(RowIdError::Invalid);
        }

        if !Id::validate(&self.user_uuid) {
            errr.user_uuid = Some(UserUuidError::Invalid);
        }
        errr
    }

    pub(crate) async fn state_full_check<Db: DatabaseRead>(
        &self,
        db: &mut Db::Db<'_>,
    ) -> Result<Error, DynamicError> {
        let read_output = Db::read(db, &ReadInput {
            new_uuid: self.new_uuid.clone(),
        })
        .await?;

        let mut errr = Error::default();
        if read_output.is_new_uuid_used {
            errr.new_uuid = Some(RowIdError::Duplicated);
        }

        Ok(errr)
    }

    pub(crate) fn state_less_operation(&self) -> Ok {
        const ROLE: Role = Role::Manager;

        Ok {
            new_uuid:     self.new_uuid.clone(),
            company_name: self.company_name.clone(),
            currency:     self.currency.clone(),
            user_uuid:    self.user_uuid.clone(),
            role:         ROLE,
        }
    }
}
