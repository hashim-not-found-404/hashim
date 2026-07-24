use crate::accounting_domain::cases;
use crate::accounting_domain::utility::types;
use crate::server::utility::server_traits::DBClient;
use crate::server::utility::server_traits::SideEffects;
use crate::utility::traits;

impl cases::sign_in::Input {
    pub(crate) async fn handle_operation<
        Auth: types::HashedPassword,
        Jwt: types::JWT,
        Cli: DBClient,
        Db: for<'a> cases::sign_in::DatabaseRead<Db<'a> = Cli>,
    >(
        &self,
        side_effects: &mut SideEffects,
        client: &mut Cli,
        jwt: &Jwt,
    ) -> Result<cases::sign_in::MyResult, traits::DynamicError> {
        let result = self.state_full_check::<Auth, Jwt, Db>(jwt, client).await?;

        if let Ok(ok) = &result {
            side_effects.authenticated_users.insert(ok.user_uuid.clone());
            side_effects.users_to_resubscribe.insert(ok.user_uuid.clone());
        }

        return Ok(result);
    }
}
