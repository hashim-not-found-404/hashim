use serde::Deserialize;
use serde::Serialize;
use std::ops::Deref;
use use_case_prelude_domain::*;

#[serde(name = "create_journal")]
impl TraitOperationDTOInput for Input {}

#[serde(name = "create_journal")]
impl TraitOperationDTOOk for Ok {}

#[serde(name = "create_journal")]
impl TraitOperationDTOError for Error {}

const RESOURCES_NAME_TO_POKE: &[ResourceName] = &[new_resource_name("journals")];

impl TraitOperationClientOk for Ok {
    fn subs_to_poke(&self) -> &'static [ResourceName] {
        RESOURCES_NAME_TO_POKE
    }
}

impl TraitOperationClientError for Error {
    fn subs_to_poke(&self) -> &'static [ResourceName] {
        RESOURCES_NAME_TO_POKE
    }
}

impl TraitOperationClientInput for Input {
    fn user_uuid(&self) -> Option<[u8; 16]> {
        Some(*self.user_uuid.deref().deref())
    }

    fn state_less_check(&self) -> Option<TypeOperationClientError> {
        let errr = self.state_less_check();
        if errr.is_there_error() {
            Some(Box::new(errr))
        } else {
            None
        }
    }
}

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
    pub user_uuid: Option<UserUuidError>,
    pub new_uuid: Option<RowIdError>,
    pub company_belong: Option<RowIdError>,
    pub name: Option<JournalNameError>,
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
    pub fn state_less_check(&self) -> Error {
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

    pub async fn state_full_check<Db: DatabaseRead<Input = ReadInput, Output = ReadOutput>>(
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

    pub fn state_less_operation(&self) -> Ok {
        Ok {
            new_uuid: self.new_uuid.clone(),
            company_belong: self.company_belong.clone(),
            name: self.name.clone(),
            is_it_contain_quantity: self.is_it_contain_quantity,
        }
    }
}
