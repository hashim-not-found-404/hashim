use crate::accounting_domain::utility::types;
use accounting_engine::accounting_stuff;
use serde::Deserialize;
use serde::Serialize;

pub type MyResult = Result<Ok, Error>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub(crate) user_uuid:           types::UuidType,
    pub(crate) company_branch_uuid: types::UuidType,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Ok {
    pub(crate) company_uuid:        types::UuidType,
    pub(crate) company_branch_uuid: types::UuidType,
    pub(crate) accounts:            Vec<Account>,
    pub(crate) accounts_for_branch: Vec<AccountForBranch>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Account {
    pub row_uuid:                        types::UuidType,
    pub is_debit:                        bool,
    pub is_permanent_account:            bool,
    pub account_name:                    String,
    pub notes:                           Option<String>,
    pub unit_of_measurement_of_quantity: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AccountForBranch {
    pub row_uuid:     types::UuidType,
    pub account_uuid: types::UuidType,
    pub outflow_type: accounting_stuff::OutFlowType,
    pub inflow_type:  accounting_stuff::InFlowType,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct Error {
    pub(crate) user_uuid:           Option<types::UserUuidError>,
    pub(crate) company_branch_uuid: Option<types::RowIdError>,
}

impl types::MarkerMyErrorTrait for Error {}

pub struct ReadInput {
    pub user_uuid:           types::UuidType,
    pub company_branch_uuid: types::UuidType,
}

#[derive(Debug, Default)]
pub struct ReadOutput {
    pub company_uuid:        types::UuidType,
    pub accounts:            Vec<Account>,
    pub accounts_for_branch: Vec<AccountForBranch>,
}

pub trait DatabaseRead:
    types::DatabaseRead<ReadInput = ReadInput, ReadOutput = ReadOutput>
{
}

impl Input {
    pub(crate) fn state_less_check<Id: types::RowId>(&self) -> Error {
        let mut errr = Error::default();

        if !Id::validate(&self.user_uuid) {
            errr.user_uuid = Some(types::UserUuidError::Invalid);
        }

        if !Id::validate(&self.company_branch_uuid) {
            errr.company_branch_uuid = Some(types::RowIdError::Invalid);
        }

        errr
    }

    pub(crate) async fn state_full_operation<Db: DatabaseRead>(
        &self,
        db: &mut Db::Db<'_>,
    ) -> Result<Ok, Db::Error> {
        let read_output = Db::read(db, &ReadInput {
            user_uuid:           self.user_uuid.clone(),
            company_branch_uuid: self.company_branch_uuid.clone(),
        })
        .await?;

        Ok(Ok {
            company_uuid:        read_output.company_uuid.clone(),
            company_branch_uuid: self.company_branch_uuid.clone(),
            accounts:            read_output.accounts.clone(),
            accounts_for_branch: read_output.accounts_for_branch.clone(),
        })
    }
}
