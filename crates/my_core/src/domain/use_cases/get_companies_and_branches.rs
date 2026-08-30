use crate::domain::utility::types;
use crate::domain::utility::types::Currency;
use crate::domain::utility::types::MarkerMyErrorTrait;
use crate::domain::utility::types::Role;
use crate::domain::utility::types::RowId;
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
    pub(crate) user_uuid: User,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Ok {
    pub(crate) user_uuid: User,
    pub(crate) data:      Vec<AllCompaniesThatUserInWithRoles>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AllCompaniesThatUserInWithRoles {
    pub company_uuid:     Company,
    pub company_name:     String,
    pub company_currancy: Currency,
    pub user_roles:       Vec<Role>,
    pub branches:         Vec<AllBranchesThatUserInWithRoles>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AllBranchesThatUserInWithRoles {
    pub branch_uuid:     Branch,
    pub branch_name:     String,
    pub branch_currancy: Currency,
    pub user_roles:      Vec<Role>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct Error {
    pub(crate) user_uuid: Option<UserUuidError>,
}

impl MarkerMyErrorTrait for Error {}

pub struct ReadInput {
    pub user_uuid: User,
}

pub struct ReadOutput {
    pub data: Vec<AllCompaniesThatUserInWithRoles>,
}

pub trait DatabaseRead: types::DatabaseRead<Input = ReadInput, Output = ReadOutput> {}

impl Input {
    pub(crate) fn state_less_check<Id: RowId>(&self) -> Error {
        let mut errr = Error::default();

        if !Id::validate(&self.user_uuid) {
            errr.user_uuid = Some(UserUuidError::Invalid);
        }

        errr
    }

    pub(crate) async fn state_full_operation<Db: DatabaseRead>(
        &self,
        db: &mut Db::Db<'_>,
    ) -> Result<Ok, DynamicError> {
        let read_output = Db::read(db, &ReadInput {
            user_uuid: self.user_uuid.clone(),
        })
        .await?;

        Ok(Ok {
            user_uuid: self.user_uuid.clone(),
            data:      read_output.data,
        })
    }
}
