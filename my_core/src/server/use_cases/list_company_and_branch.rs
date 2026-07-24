use crate::accounting_domain::cases;
use crate::accounting_domain::utility::types;
use crate::accounting_domain::utility::types::MyErrorTrait;
use crate::server::utility::server_traits::DBClient;
use crate::server::utility::server_traits::SideEffects;
use crate::utility::traits;

impl cases::list_company_and_branch::Input {
    pub(crate) async fn handle_operation<
        Id: types::RowId,
        Cli: DBClient,
        Db: for<'a> cases::list_company_and_branch::DatabaseRead<Db<'a> = Cli>,
    >(
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

        let result = self.state_full_operation::<Db>(client).await?;

        return Ok(Ok(result));
    }
}
