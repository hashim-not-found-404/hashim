use crate::accounting_domain::utility::types;
use crate::utility::traits;
use serde::Deserialize;
use serde::Serialize;

pub type MyResult = Result<Ok, Error>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub(crate) user_uuid:    types::UuidType,
    pub(crate) company_uuid: types::UuidType,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Ok {
    pub(crate) company_uuid: types::UuidType,
    pub(crate) data:         Vec<Data>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Data {
    pub row_uuid:                        types::UuidType,
    pub is_debit:                        bool,
    pub is_permanent_account:            bool,
    pub account_name:                    String,
    pub notes:                           Option<String>,
    pub unit_of_measurement_of_quantity: String,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct Error {
    pub(crate) user_uuid:    Option<types::UserUuidError>,
    pub(crate) company_uuid: Option<types::RowIdError>,
}

impl types::MarkerMyErrorTrait for Error {}

pub struct ReadInput {
    pub user_uuid:    types::UuidType,
    pub company_uuid: types::UuidType,
}

pub struct ReadOutput {
    pub data: Vec<Data>,
}

pub trait DatabaseRead {
    type Db<'a>;
    fn read(
        db: &mut Self::Db<'_>,
        read_input: &ReadInput,
    ) -> impl Future<Output = Result<ReadOutput, traits::DynamicError>>;
}

impl Input {
    pub(crate) fn state_less_check<Id: types::RowId>(&self) -> Error {
        let mut errr = Error::default();

        if !Id::validate(&self.user_uuid) {
            errr.user_uuid = Some(types::UserUuidError::Invalid);
        }

        if !Id::validate(&self.company_uuid) {
            errr.company_uuid = Some(types::RowIdError::Invalid);
        }

        errr
    }

    pub(crate) async fn state_full_operation<Db: DatabaseRead>(
        &self,
        db: &mut Db::Db<'_>,
    ) -> Result<Ok, traits::DynamicError> {
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
