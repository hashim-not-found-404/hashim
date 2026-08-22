use crate::accounting_domain::cases;
use crate::accounting_domain::utility::types;
use crate::accounting_domain::utility::types::MyErrorTrait;
use crate::server::utility::server_traits::DBClient;
use crate::server::utility::server_traits::SideEffects;
use crate::utility::traits::DynamicError;

impl cases::get_all_accounts::Input {
    pub(crate) async fn handle_operation<
        Id: types::RowId,
        Cli: DBClient,
        Db: for<'a> cases::get_all_accounts::DatabaseRead<Db<'a> = Cli, Error = DynamicError>,
    >(
        &self,
        side_effects: &mut SideEffects,
        client: &mut Cli,
    ) -> Result<cases::get_all_accounts::MyResult, DynamicError> {
        let mut errr = self.state_less_check::<Id>();
        if !side_effects.authenticated_users.contains(&self.user_uuid) {
            errr.user_uuid = Some(types::UserUuidError::NotAuthenticated);
        }
        if errr.is_there_error() {
            return Ok(Err(errr));
        }

        let ok = self.state_full_operation::<Db>(client).await?;

        Ok(Ok(ok))
    }
}
