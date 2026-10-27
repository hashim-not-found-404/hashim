use anyhow::Result;
use infrastructure::row_id::Id;
use infrastructure::row_id::RowId;
use kernel::make_type;
use kernel::new_types::CompanyUuid;
use kernel::new_types::UserUuid;
use kernel::new_types::UuidType;
use kernel::types::DatabaseRead;
use kernel::types::MarkerMyErrorTrait;
use kernel::types::Role;
use kernel::types::RowIdError;
use kernel::types::UserUuidError;
use serde::Deserialize;
use serde::Serialize;
use typetag::serde;
use utility::dtos::TraitOperationDTOError;
use utility::dtos::TraitOperationDTOInput;
use utility::dtos::TraitOperationDTOOk;

#[serde(name = "create_journal")]
impl TraitOperationDTOInput for Input {}

#[serde(name = "create_journal")]
impl TraitOperationDTOOk for Ok {}

#[serde(name = "create_journal")]
impl TraitOperationDTOError for Error {}

make_type!(UuidType, JournalUuid);

pub type MyResult = Result<Ok, Error>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub user_uuid: UserUuid,
    pub new_uuid: JournalUuid,
    pub company_belong: CompanyUuid,
    pub name: String,
    pub is_it_contain_quantity: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Ok {
    pub new_uuid: JournalUuid,
    pub company_belong: CompanyUuid,
    pub name: String,
    pub is_it_contain_quantity: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct Error {
    pub(crate) user_uuid: Option<UserUuidError>,
    pub(crate) new_uuid: Option<RowIdError>,
    pub(crate) company_belong: Option<RowIdError>,
    pub(crate) name: Option<JournalNameError>,
}

impl MarkerMyErrorTrait for Error {}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum JournalNameError {
    Empty,
    Duplicated,
}

pub struct ReadInput {
    pub user_uuid: UserUuid,
    pub new_uuid: JournalUuid,
    pub company_belong: CompanyUuid,
    pub name: String,
}

pub struct ReadOutput {
    pub user_roles: Vec<Role>,
    pub is_new_uuid_used: bool,
    pub is_company_exist: bool,
    pub is_name_used: bool,
}

impl Input {
    pub(crate) fn state_less_check(&self) -> Error {
        let mut errr = Error::default();

        if !Id::validate(&self.new_uuid.0) {
            errr.new_uuid = Some(RowIdError::Invalid);
        }
        if !Id::validate(&self.user_uuid) {
            errr.user_uuid = Some(UserUuidError::Invalid);
        }
        if !Id::validate(&self.company_belong) {
            errr.company_belong = Some(RowIdError::Invalid);
        }
        if self.name.trim().is_empty() {
            errr.name = Some(JournalNameError::Empty);
        }
        errr
    }

    pub(crate) async fn state_full_check<
        Db: DatabaseRead<Input = ReadInput, Output = ReadOutput>,
    >(
        &self,
        db: &mut Db::Db<'_>,
    ) -> Result<Error> {
        let read_output = Db::read(
            db,
            &ReadInput {
                user_uuid: self.user_uuid.clone(),
                new_uuid: self.new_uuid.clone(),
                company_belong: self.company_belong.clone(),
                name: self.name.clone(),
            },
        )
        .await?;

        let mut errr = Error::default();

        if read_output.is_new_uuid_used {
            errr.new_uuid = Some(RowIdError::Duplicated);
        }
        if !read_output.is_company_exist {
            errr.company_belong = Some(RowIdError::NotExist);
        }
        if read_output.is_name_used {
            errr.name = Some(JournalNameError::Duplicated);
        }
        if !Role::has_any(&read_output.user_roles, &[Role::Manager, Role::CoManager]) {
            errr.user_uuid = Some(UserUuidError::YouDontHavePermissionToDoThat);
        }
        Ok(errr)
    }

    pub(crate) fn state_less_operation(&self) -> Ok {
        Ok {
            new_uuid: self.new_uuid.clone(),
            company_belong: self.company_belong.clone(),
            name: self.name.clone(),
            is_it_contain_quantity: self.is_it_contain_quantity,
        }
    }
}
