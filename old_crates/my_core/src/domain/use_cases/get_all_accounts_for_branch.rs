use crate::domain::utility::new_types;
use crate::domain::utility::new_types::BranchUuid;
use crate::domain::utility::new_types::CompanyUuid;
use crate::domain::utility::new_types::UserUuid;
use crate::domain::utility::types;
use crate::domain::utility::types::MarkerMyErrorTrait;
use crate::domain::utility::types::RowId;
use crate::domain::utility::types::RowIdError;
use crate::domain::utility::types::UserUuidError;
use crate::utility::traits::DynamicError;
use accounting_engine::accounting_stuff::InFlowType;
use accounting_engine::accounting_stuff::OutFlowType;
use serde::Deserialize;
use serde::Serialize;

pub type MyResult = Result<Ok, Error>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub(crate) user_uuid:           UserUuid,
    pub(crate) company_branch_uuid: BranchUuid,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Ok {
    pub(crate) company_uuid:        CompanyUuid,
    pub(crate) company_branch_uuid: BranchUuid,
    pub(crate) accounts:            Vec<Account>,
    pub(crate) accounts_for_branch: Vec<AccountForBranch>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Account {
    pub row_uuid:                        new_types::AccountUuid,
    pub is_debit:                        bool,
    pub is_permanent_account:            bool,
    pub account_name:                    String,
    pub notes:                           Option<String>,
    pub unit_of_measurement_of_quantity: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AccountForBranch {
    pub row_uuid:     new_types::AccountForBranchUuid,
    pub account_uuid: new_types::AccountUuid,
    pub outflow_type: OutFlowType,
    pub inflow_type:  InFlowType,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct Error {
    pub(crate) user_uuid:           Option<UserUuidError>,
    pub(crate) company_branch_uuid: Option<RowIdError>,
}

impl MarkerMyErrorTrait for Error {}

pub struct ReadInput {
    pub user_uuid:           UserUuid,
    pub company_branch_uuid: BranchUuid,
}

#[derive(Debug, Default)]
pub struct ReadOutput {
    pub company_uuid:        CompanyUuid,
    pub accounts:            Vec<Account>,
    pub accounts_for_branch: Vec<AccountForBranch>,
}

pub trait DatabaseRead: types::DatabaseRead<Input = ReadInput, Output = ReadOutput> {}

impl Input {
    pub(crate) fn state_less_check<Id: RowId>(&self) -> Error {
        let mut errr = Error::default();

        if !Id::validate(&self.user_uuid) {
            errr.user_uuid = Some(UserUuidError::Invalid);
        }

        if !Id::validate(&self.company_branch_uuid) {
            errr.company_branch_uuid = Some(RowIdError::Invalid);
        }

        errr
    }

    pub(crate) async fn state_full_operation<Db: DatabaseRead>(
        &self,
        db: &mut Db::Db<'_>,
    ) -> Result<Ok, DynamicError> {
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
