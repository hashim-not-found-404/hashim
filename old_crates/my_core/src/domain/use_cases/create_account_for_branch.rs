use crate::domain::utility::new_types::AccountForBranchUuid;
use crate::domain::utility::new_types::AccountUuid;
use crate::domain::utility::new_types::BranchUuid;
use crate::domain::utility::new_types::UserUuid;
use crate::domain::utility::types;
use crate::domain::utility::types::MarkerMyErrorTrait;
use crate::domain::utility::types::Role;
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
    pub user_uuid:                UserUuid,
    pub new_uuid:                 AccountForBranchUuid,
    pub belong_to_account:        AccountUuid,
    pub belong_to_company_branch: BranchUuid,
    pub outflow_type:             OutFlowType,
    pub inflow_type:              InFlowType,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Ok {
    pub new_uuid:                 AccountForBranchUuid,
    pub belong_to_account:        AccountUuid,
    pub belong_to_company_branch: BranchUuid,
    pub outflow_type:             OutFlowType,
    pub inflow_type:              InFlowType,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct Error {
    pub(crate) user_uuid:                                Option<UserUuidError>,
    pub(crate) new_uuid:                                 Option<RowIdError>,
    pub(crate) belong_to_account:                        Option<RowIdError>,
    pub(crate) belong_to_company_branch:                 Option<RowIdError>,
    pub(crate) is_account_uuid_with_company_branch_used: bool,
}

impl MarkerMyErrorTrait for Error {}

pub struct ReadInput {
    pub user_uuid:                UserUuid,
    pub new_uuid:                 AccountForBranchUuid,
    pub belong_to_account:        AccountUuid,
    pub belong_to_company_branch: BranchUuid,
}

pub struct ReadOutput {
    pub user_roles:                               Vec<Role>,
    pub is_new_uuid_used:                         bool,
    pub is_account_uuid_exist:                    bool,
    pub is_company_branch_exist:                  bool,
    pub is_account_uuid_with_company_branch_used: bool,
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

        if !Id::validate(&self.belong_to_account) {
            errr.belong_to_account = Some(RowIdError::Invalid);
        }

        if !Id::validate(&self.belong_to_company_branch) {
            errr.belong_to_company_branch = Some(RowIdError::Invalid);
        }
        errr
    }

    pub(crate) async fn state_full_check<Db: DatabaseRead>(
        &self,
        db: &mut Db::Db<'_>,
    ) -> Result<Error, DynamicError> {
        let read_output = Db::read(db, &ReadInput {
            user_uuid:                self.user_uuid.clone(),
            new_uuid:                 self.new_uuid.clone(),
            belong_to_account:        self.belong_to_account.clone(),
            belong_to_company_branch: self.belong_to_company_branch.clone(),
        })
        .await?;

        let mut errr = Error::default();

        if read_output.is_new_uuid_used {
            errr.new_uuid = Some(RowIdError::Duplicated);
        }

        if !read_output.is_account_uuid_exist {
            errr.belong_to_account = Some(RowIdError::NotExist);
        }

        if !read_output.is_company_branch_exist {
            errr.belong_to_company_branch = Some(RowIdError::NotExist);
        }

        errr.is_account_uuid_with_company_branch_used =
            read_output.is_account_uuid_with_company_branch_used;

        if !Role::has_any(&read_output.user_roles, &[Role::Manager, Role::CoManager]) {
            errr.user_uuid = Some(UserUuidError::YouDontHavePermissionToDoThat);
        }

        Ok(errr)
    }

    pub(crate) fn state_less_operation(&self) -> Ok {
        Ok {
            new_uuid:                 self.new_uuid.clone(),
            belong_to_account:        self.belong_to_account.clone(),
            belong_to_company_branch: self.belong_to_company_branch.clone(),
            outflow_type:             self.outflow_type,
            inflow_type:              self.inflow_type,
        }
    }
}
