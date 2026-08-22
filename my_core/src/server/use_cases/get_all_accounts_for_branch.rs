use crate::domain::cases;
use crate::domain::utility::types::MyErrorTrait;
use crate::domain::utility::types::RowId;
use crate::domain::utility::types::UserUuidError;
use crate::server::utility::server_traits::DBClient;
use crate::server::utility::server_traits::SideEffects;
use crate::utility::traits::DynamicError;

impl cases::get_all_accounts_for_branch::Input {
    pub(crate) async fn handle_operation<
        Id: RowId,
        Cli: DBClient,
        Db: for<'a> cases::get_all_accounts_for_branch::DatabaseRead<Db<'a> = Cli>,
    >(
        &self,
        side_effects: &mut SideEffects,
        client: &mut Cli,
    ) -> Result<cases::get_all_accounts_for_branch::MyResult, DynamicError> {
        let mut errr = self.state_less_check::<Id>();
        if !side_effects.authenticated_users.contains(&self.user_uuid) {
            errr.user_uuid = Some(UserUuidError::NotAuthenticated);
        }
        if errr.is_there_error() {
            return Ok(Err(errr));
        }

        let ok = self.state_full_operation::<Db>(client).await?;

        Ok(Ok(ok))
    }
}
