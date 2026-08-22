use crate::domain::cases;
use crate::domain::utility::types::HashedPassword;
use crate::domain::utility::types::JWT;
use crate::server::utility::server_traits::DBClient;
use crate::server::utility::server_traits::SideEffects;
use crate::utility::traits::DynamicError;

impl cases::sign_in::Input {
    pub(crate) async fn handle_operation<
        Auth: HashedPassword,
        Jwt: JWT,
        Cli: DBClient,
        Db: for<'a> cases::sign_in::DatabaseRead<Db<'a> = Cli>,
    >(
        &self,
        side_effects: &mut SideEffects,
        client: &mut Cli,
        jwt: &Jwt,
    ) -> Result<cases::sign_in::MyResult, DynamicError> {
        let result = self.state_full_check::<Auth, Jwt, Db>(jwt, client).await?;

        if let Ok(ok) = &result {
            side_effects.authenticated_users.insert(ok.user_uuid.clone());
            side_effects.users_to_resubscribe.insert(ok.user_uuid.clone());
        }

        Ok(result)
    }
}
