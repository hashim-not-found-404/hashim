use serde::Deserialize;
use serde::Serialize;
use std::ops::Deref;
use use_case_prelude_domain::*;

#[serde(name = "get_companies_and_branches")]
impl TraitOperationDTOInput for Input {}

#[serde(name = "get_companies_and_branches")]
impl TraitOperationDTOOk for Ok {}

#[serde(name = "get_companies_and_branches")]
impl TraitOperationDTOError for Error {}

const RESOURCES_NAME_TO_POKE: &[ResourceName] = &[
    new_resource_name("companies"),
    new_resource_name("branches"),
];

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
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Ok {
    pub user_uuid: UserUuid,
    pub companies: Vec<CompanyWithBranches>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CompanyWithBranches {
    pub uuid: CompanyUuid,
    pub name: String,
    pub currency: Currency,
    pub roles: Vec<Role>,
    pub branches: Vec<BranchInfo>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct BranchInfo {
    pub uuid: BranchUuid,
    pub name: String,
    pub roles: Vec<Role>,
}
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct Error {
    pub user_uuid: Option<UserUuidError>,
}

impl MarkerMyErrorTrait for Error {}

pub struct ReadInput {
    pub user_uuid: UserUuid,
}

pub struct ReadOutput {
    pub companies: Vec<CompanyWithBranches>,
}

impl Input {
    pub fn state_less_check(&self) -> Error {
        let mut errr = Error::default();

        if !Id::validate(&self.user_uuid) {
            errr.user_uuid = Some(UserUuidError::Invalid);
        }

        errr
    }

    pub async fn state_full_operation<Db: DatabaseRead<Input = ReadInput, Output = ReadOutput>>(
        &self,
        db: &mut Db::Db<'_>,
    ) -> Result<Ok> {
        let read_output = Db::read(
            db,
            &ReadInput {
                user_uuid: self.user_uuid.clone(),
            },
        )
        .await?;

        Ok(Ok {
            user_uuid: self.user_uuid.clone(),
            companies: read_output.companies,
        })
    }
}
