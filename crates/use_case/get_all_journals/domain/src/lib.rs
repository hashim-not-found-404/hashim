use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Deref;
use use_case_prelude_domain::*;

#[serde(name = "get_all_journals")]
impl TraitOperationDTOInput for Input {}

#[serde(name = "get_all_journals")]
impl TraitOperationDTOOk for Ok {}

#[serde(name = "get_all_journals")]
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

pub type MyResult = Result<Ok, Error>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub user_uuid: UserUuid,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Ok {
    pub data: Vec<JournalInfo>,
}

make_type!(UuidType, JournalUuid);

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct JournalInfo {
    pub row_uuid: JournalUuid,
    pub company_uuid: CompanyUuid,
    pub name: String,
    pub is_it_contain_quantity: bool,
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
    pub user_roles: HashMap<CompanyUuid, Vec<Role>>,
    pub data: Vec<JournalInfo>,
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
    ) -> Result<MyResult> {
        let read_output = Db::read(
            db,
            &ReadInput {
                user_uuid: self.user_uuid.clone(),
            },
        )
        .await?;

        let mut accepted_uuid = HashSet::new();

        for (company_uuid, role) in read_output.user_roles {
            if Role::has_any(&[Role::Manager], role.as_slice()) {
                accepted_uuid.insert(company_uuid);
            }
        }

        let mut journals = Vec::new();

        for i in read_output.data {
            if accepted_uuid.contains(&i.company_uuid) {
                journals.push(i);
            }
        }

        Ok(Ok(Ok { data: journals }))
    }
}
