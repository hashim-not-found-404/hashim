use serde::Deserialize;
use serde::Serialize;
use std::ops::Deref;
use use_case_prelude_domain::*;

#[serde(name = "create_company")]
impl TraitOperationDTOInput for Input {}

#[serde(name = "create_company")]
impl TraitOperationDTOOk for Ok {}

#[serde(name = "create_company")]
impl TraitOperationDTOError for Error {}

const RESOURCES_NAME_TO_POKE: &[ResourceName] = &[new_resource_name("companies")];

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

pub type MyResult = Result<Ok, Error>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub user_uuid: UserUuid,
    pub new_uuid: CompanyUuid,
    pub company_name: String,
    pub currency: Currency,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Ok {
    pub new_uuid: CompanyUuid,
    pub company_name: String,
    pub currency: Currency,
    pub user_uuid: UserUuid,
    pub role: Role,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct Error {
    pub user_uuid: Option<UserUuidError>,
    pub new_uuid: Option<RowIdError>,
    pub company_name: Option<CompanyNameError>,
}

impl MarkerMyErrorTrait for Error {}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum CompanyNameError {
    Empty,
}

pub struct ReadInput {
    pub new_uuid: CompanyUuid,
}

pub struct ReadOutput {
    pub is_new_uuid_used: bool,
}

impl Input {
    pub fn state_less_check(&self) -> Error {
        let mut errr = Error::default();

        if !Id::validate(&self.new_uuid) {
            errr.new_uuid = Some(RowIdError::Invalid);
        }

        if !Id::validate(&self.user_uuid) {
            errr.user_uuid = Some(UserUuidError::Invalid);
        }

        if self.company_name.trim().is_empty() {
            errr.company_name = Some(CompanyNameError::Empty);
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
                new_uuid: self.new_uuid.clone(),
            },
        )
        .await?;

        let mut errr = Error::default();

        if read_output.is_new_uuid_used {
            errr.new_uuid = Some(RowIdError::Duplicated);
        }

        Ok(errr)
    }

    pub fn state_less_operation(&self) -> Ok {
        Ok {
            new_uuid: self.new_uuid.clone(),
            company_name: self.company_name.clone(),
            currency: self.currency.clone(),
            user_uuid: self.user_uuid.clone(),
            role: Role::Manager,
        }
    }
}
