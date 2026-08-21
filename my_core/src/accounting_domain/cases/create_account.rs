use crate::accounting_domain::utility::types;
use serde::Deserialize;
use serde::Serialize;

pub type MyResult = Result<Ok, Error>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub user_uuid:                       types::UuidType,
    pub new_uuid:                        types::UuidType,
    pub is_debit:                        bool,
    pub is_permanent_account:            bool,
    pub account_name:                    String,
    pub notes:                           Option<String>,
    pub unit_of_measurement_of_quantity: String,
    pub belong_to_company:               types::UuidType,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Ok {
    pub new_uuid:                        types::UuidType,
    pub is_debit:                        bool,
    pub is_permanent_account:            bool,
    pub account_name:                    String,
    pub notes:                           Option<String>,
    pub unit_of_measurement_of_quantity: String,
    pub belong_to_company:               types::UuidType,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct Error {
    pub(crate) user_uuid:         Option<types::UserUuidError>,
    pub(crate) new_uuid:          Option<types::RowIdError>,
    pub(crate) belong_to_company: Option<types::RowIdError>,
    pub(crate) account_name:      Option<AccountNameError>,
}

impl types::MarkerMyErrorTrait for Error {}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) enum AccountNameError {
    Duplicated,
}

pub struct ReadInput {
    pub user_uuid:         types::UuidType,
    pub new_uuid:          types::UuidType,
    pub belong_to_company: types::UuidType,
    pub account_name:      String,
}

pub struct ReadOutput {
    pub is_company_uuid_exist: bool,
    pub is_new_uuid_used:      bool,
    pub user_roles:            Vec<types::Role>,
    pub is_account_name_used:  bool,
}

pub trait DatabaseRead: types::DatabaseRead<Input = ReadInput, Output = ReadOutput> {}

impl Input {
    pub(crate) fn state_less_check<Id: types::RowId>(&self) -> Error {
        let mut errr = Error::default();

        if !Id::validate(&self.new_uuid) {
            errr.new_uuid = Some(types::RowIdError::Invalid);
        }

        if !Id::validate(&self.user_uuid) {
            errr.user_uuid = Some(types::UserUuidError::Invalid);
        }

        if !Id::validate(&self.belong_to_company) {
            errr.belong_to_company = Some(types::RowIdError::Invalid);
        }
        errr
    }

    pub(crate) async fn state_full_check<Db: DatabaseRead>(
        &self,
        db: &mut Db::Db<'_>,
    ) -> Result<Error, Db::Error> {
        let read_output = Db::read(db, &ReadInput {
            user_uuid:         self.user_uuid.clone(),
            new_uuid:          self.new_uuid.clone(),
            belong_to_company: self.belong_to_company.clone(),
            account_name:      self.account_name.clone(),
        })
        .await?;

        let mut errr = Error::default();

        if read_output.is_new_uuid_used {
            errr.new_uuid = Some(types::RowIdError::Duplicated);
        }

        if !read_output.is_company_uuid_exist {
            errr.belong_to_company = Some(types::RowIdError::NotExist);
        }

        if read_output.is_account_name_used {
            errr.account_name = Some(AccountNameError::Duplicated);
        }

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
            new_uuid:                        self.new_uuid.clone(),
            is_debit:                        self.is_debit,
            is_permanent_account:            self.is_permanent_account,
            account_name:                    self.account_name.clone(),
            notes:                           self.notes.clone(),
            unit_of_measurement_of_quantity: self.unit_of_measurement_of_quantity.clone(),
            belong_to_company:               self.belong_to_company.clone(),
        }
    }
}
