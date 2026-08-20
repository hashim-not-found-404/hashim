use crate::accounting_domain::utility::types;
use crate::utility::traits;
use accounting_engine::accounting_stuff;
use serde::Deserialize;
use serde::Serialize;

pub type MyResult = Result<Ok, Error>;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Input {
    pub user_uuid:                types::UuidType,
    pub new_uuid:                 types::UuidType,
    pub belong_to_account:        types::UuidType,
    pub belong_to_company_branch: types::UuidType,
    pub outflow_type:             accounting_stuff::OutFlowType,
    pub inflow_type:              accounting_stuff::InFlowType,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Ok {
    pub new_uuid:                 types::UuidType,
    pub belong_to_account:        types::UuidType,
    pub belong_to_company_branch: types::UuidType,
    pub outflow_type:             accounting_stuff::OutFlowType,
    pub inflow_type:              accounting_stuff::InFlowType,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct Error {
    pub(crate) user_uuid:                                Option<types::UserUuidError>,
    pub(crate) new_uuid:                                 Option<types::RowIdError>,
    pub(crate) belong_to_account:                        Option<types::RowIdError>,
    pub(crate) belong_to_company_branch:                 Option<types::RowIdError>,
    pub(crate) is_account_uuid_with_company_branch_used: bool,
}

impl types::MarkerMyErrorTrait for Error {}

pub struct ReadInput {
    pub user_uuid:                types::UuidType,
    pub new_uuid:                 types::UuidType,
    pub belong_to_account:        types::UuidType,
    pub belong_to_company_branch: types::UuidType,
}

pub struct ReadOutput {
    pub user_roles:                               Vec<types::Role>,
    pub is_new_uuid_used:                         bool,
    pub is_account_uuid_exist:                    bool,
    pub is_company_branch_exist:                  bool,
    pub is_account_uuid_with_company_branch_used: bool,
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

        if !Id::validate(&self.new_uuid) {
            errr.new_uuid = Some(types::RowIdError::Invalid);
        }

        if !Id::validate(&self.user_uuid) {
            errr.user_uuid = Some(types::UserUuidError::Invalid);
        }

        if !Id::validate(&self.belong_to_account) {
            errr.belong_to_account = Some(types::RowIdError::Invalid);
        }

        if !Id::validate(&self.belong_to_company_branch) {
            errr.belong_to_company_branch = Some(types::RowIdError::Invalid);
        }
        errr
    }

    pub(crate) async fn state_full_check<Db: DatabaseRead>(
        &self,
        db: &mut Db::Db<'_>,
    ) -> Result<Error, traits::DynamicError> {
        let read_output = Db::read(db, &ReadInput {
            user_uuid:                self.user_uuid.clone(),
            new_uuid:                 self.new_uuid.clone(),
            belong_to_account:        self.belong_to_account.clone(),
            belong_to_company_branch: self.belong_to_company_branch.clone(),
        })
        .await?;

        let mut errr = Error::default();

        if read_output.is_new_uuid_used {
            errr.new_uuid = Some(types::RowIdError::Duplicated);
        }

        if !read_output.is_account_uuid_exist {
            errr.belong_to_account = Some(types::RowIdError::NotExist);
        }

        if !read_output.is_company_branch_exist {
            errr.belong_to_company_branch = Some(types::RowIdError::NotExist);
        }

        errr.is_account_uuid_with_company_branch_used =
            read_output.is_account_uuid_with_company_branch_used;

        if !types::Role::has_any(&read_output.user_roles, &[
            types::Role::Manager,
            types::Role::CoManager,
        ]) {
            errr.user_uuid = Some(types::UserUuidError::YouDontHavePermissionToDoThat);
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
