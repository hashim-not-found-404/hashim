use serde::Deserialize;
use serde::Serialize;
use std::ops::Deref;
use use_case_prelude_domain::*;

make_type!(UuidType, AccountUuid);

#[serde(name = "get_all_accounts")]
impl TraitOperationDTOInput for Input {}

#[serde(name = "get_all_accounts")]
impl TraitOperationDTOOk for Ok {}

#[serde(name = "get_all_accounts")]
impl TraitOperationDTOError for Error {}

const RESOURCES_NAME_TO_POKE: &[ResourceName] = &[new_resource_name("accounts")];

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
    pub company_uuid: CompanyUuid,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Ok {
    pub company_uuid: CompanyUuid,
    pub data: Vec<Data>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Data {
    pub row_uuid: AccountUuid,
    pub is_debit: bool,
    pub is_permanent_account: bool,
    pub account_name: String,
    pub unit_of_measurement_of_quantity: String,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct Error {
    pub user_uuid: Option<UserUuidError>,
    pub(crate) company_uuid: Option<RowIdError>,
}

impl MarkerMyErrorTrait for Error {}

pub struct ReadInput {
    pub user_uuid: UserUuid,
    pub company_uuid: CompanyUuid,
}

pub struct ReadOutput {
    pub data: Vec<Data>,
}

impl Input {
    pub fn state_less_check(&self) -> Error {
        let mut errr = Error::default();

        if !Id::validate(&self.user_uuid) {
            errr.user_uuid = Some(UserUuidError::Invalid);
        }

        if !Id::validate(&self.company_uuid) {
            errr.company_uuid = Some(RowIdError::Invalid);
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
                company_uuid: self.company_uuid.clone(),
            },
        )
        .await?;

        Ok(Ok {
            company_uuid: self.company_uuid.clone(),
            data: read_output.data,
        })
    }
}
