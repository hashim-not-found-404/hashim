use crate::accounting_domain::utility::accounting_stuff;
use crate::accounting_domain::utility::types;
use crate::utility::traits;
use serde::Deserialize;
use serde::Serialize;

pub type MyResult = Result<Ok, Error>;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Input {
    pub(crate) user_uuid:           types::UuidType,
    pub(crate) company_branch_uuid: types::UuidType,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Ok {
    pub(crate) company_uuid:        types::UuidType,
    pub(crate) company_branch_uuid: types::UuidType,
    pub(crate) accounts:            Vec<Account>,
    pub(crate) accounts_for_branch: Vec<AccountForBranch>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Account {
    pub row_uuid:                        types::UuidType,
    pub is_debit:                        bool,
    pub is_permanent_account:            bool,
    pub account_name:                    String,
    pub notes:                           String,
    pub unit_of_measurement_of_quantity: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AccountForBranch {
    pub row_uuid:     types::UuidType,
    pub account_uuid: types::UuidType,
    pub outflow_type: accounting_stuff::OutFlowType,
    pub inflow_type:  accounting_stuff::InFlowType,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct Error {
    pub(crate) user_uuid:           Option<types::UserUuidError>,
    pub(crate) company_branch_uuid: Option<types::RowIdError>,
}

impl types::MyErrorTrait for Error {
    fn is_there_error(&self) -> bool {
        *self != Self::default()
    }
}

pub struct ReadInput {
    pub user_uuid:           types::UuidType,
    pub company_branch_uuid: types::UuidType,
}

#[derive(Default)]
pub struct ReadOutput {
    pub company_uuid:        types::UuidType,
    pub accounts:            Vec<Account>,
    pub accounts_for_branch: Vec<AccountForBranch>,
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

        if !Id::validate(&self.company_branch_uuid) {
            errr.company_branch_uuid = Some(types::RowIdError::Invalid);
        }

        errr
    }

    pub(crate) async fn state_full_operation<Db: DatabaseRead>(
        &self,
        db: &mut Db::Db<'_>,
    ) -> Result<Ok, traits::DynamicError> {
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
