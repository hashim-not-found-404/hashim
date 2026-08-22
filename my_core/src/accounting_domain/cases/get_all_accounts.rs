use crate::accounting_domain::utility::types;
use crate::accounting_domain::utility::types::MarkerMyErrorTrait;
use crate::accounting_domain::utility::types::RowId;
use crate::accounting_domain::utility::types::RowIdError;
use crate::accounting_domain::utility::types::UserUuidError;
use crate::accounting_domain::utility::types::UuidType;
use crate::utility::traits::DynamicError;
use serde::Deserialize;
use serde::Serialize;

pub type MyResult = Result<Ok, Error>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub(crate) user_uuid:    UuidType,
    pub(crate) company_uuid: UuidType,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Ok {
    pub(crate) company_uuid: UuidType,
    pub(crate) data:         Vec<Data>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Data {
    pub row_uuid:                        UuidType,
    pub is_debit:                        bool,
    pub is_permanent_account:            bool,
    pub account_name:                    String,
    pub notes:                           Option<String>,
    pub unit_of_measurement_of_quantity: String,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct Error {
    pub(crate) user_uuid:    Option<UserUuidError>,
    pub(crate) company_uuid: Option<RowIdError>,
}

impl MarkerMyErrorTrait for Error {}

pub struct ReadInput {
    pub user_uuid:    UuidType,
    pub company_uuid: UuidType,
}

pub struct ReadOutput {
    pub data: Vec<Data>,
}

pub trait DatabaseRead: types::DatabaseRead<Input = ReadInput, Output = ReadOutput> {}

impl Input {
    pub(crate) fn state_less_check<Id: RowId>(&self) -> Error {
        let mut errr = Error::default();

        if !Id::validate(&self.user_uuid) {
            errr.user_uuid = Some(UserUuidError::Invalid);
        }

        if !Id::validate(&self.company_uuid) {
            errr.company_uuid = Some(RowIdError::Invalid);
        }

        errr
    }

    pub(crate) async fn state_full_operation<Db: DatabaseRead>(
        &self,
        db: &mut Db::Db<'_>,
    ) -> Result<Ok, DynamicError> {
        let read_output = Db::read(db, &ReadInput {
            user_uuid:    self.user_uuid.clone(),
            company_uuid: self.company_uuid.clone(),
        })
        .await?;

        Ok(Ok {
            company_uuid: self.company_uuid.clone(),
            data:         read_output.data,
        })
    }
}
