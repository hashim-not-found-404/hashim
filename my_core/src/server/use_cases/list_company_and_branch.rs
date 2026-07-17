use crate::{
    accounting_domain::cases::{
        self,
        utility::types::{self, MyErrorTrait},
    },
    server::use_cases::utility::server_traits::{DBClient, DBTransaction, SideEffects},
    utility::traits,
};

impl cases::list_company_and_branch::Input {
    pub(crate) async fn handle_operation<Id: types::RowId, Cli: DBClient>(
        &self,
        side_effects: &mut SideEffects,
        client: &mut Cli,
    ) -> Result<
        Result<cases::list_company_and_branch::Ok, cases::list_company_and_branch::Error>,
        traits::DynamicError,
    > {
        let mut errr = self.state_less_check::<Id>();
        if !side_effects.authenticated_users.contains(&self.user_uuid) {
            errr.user_uuid = Some(types::UserUuidError::NotAuthenticated);
        }
        if errr.is_there_error() {
            return Ok(Err(errr));
        }

        let result = client.read_list_company_and_branch(&self.user_uuid).await?;

        return Ok(Ok(cases::list_company_and_branch::Ok {
            user_uuid: self.user_uuid.clone(),
            data: result,
        }));
    }
}
