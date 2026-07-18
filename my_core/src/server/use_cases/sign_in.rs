use crate::{
    accounting_domain::cases::{self, utility::types},
    server::use_cases::utility::server_traits::{DBClient, SideEffects},
    utility::traits,
};

impl cases::sign_in::Input {
    pub(crate) async fn handle_operation<
        Auth: types::HashedPassword,
        Jwt: types::JWT,
        Cli: DBClient,
    >(
        &self,
        side_effects: &mut SideEffects,
        client: &mut Cli,
        jwt: &Jwt,
    ) -> Result<cases::sign_in::MyResult, traits::DynamicError> {
        let user_rowid_and_password_hash_and_name = client.read_sign_in(&self.user_id).await?;
        let result =
            self.state_full_check::<Auth, Jwt>(jwt, &user_rowid_and_password_hash_and_name);

        if let Ok(ok) = &result {
            side_effects
                .authenticated_users
                .insert(ok.user_uuid.clone());
            side_effects
                .users_to_resubscribe
                .insert(ok.user_uuid.clone());
        }

        return Ok(result);
    }
}
