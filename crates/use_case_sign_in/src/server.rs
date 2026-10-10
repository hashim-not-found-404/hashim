use crate::domain::Input;
use crate::domain::MyResult;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use anyhow::Result;
use infrastructure::authentication::Auth;
use infrastructure::jwt::JWT;
use kernel::new_types::UserUuid;
use kernel::server::DBClient;
use kernel::server::SideEffects;
use kernel::types::DatabaseRead;

pub async fn handle_operation_generic<
    Cli: DBClient,
    DBReader: for<'a> DatabaseRead<Db<'a> = Cli, Input = ReadInput, Output = ReadOutput>,
    Jwt: JWT<UserUuid>,
>(
    input: &Input,
    side_effects: &mut SideEffects<'_, Cli, Jwt>,
) -> Result<MyResult> {
    let result = input
        .state_full_operation::<DBReader, Jwt, Auth>(side_effects.client, side_effects.jwt)
        .await?;

    if let Ok(ok) = &result {
        side_effects
            .authenticated_users
            .insert(ok.user_uuid.clone());
        side_effects
            .users_to_resubscribe
            .insert(ok.user_uuid.clone());
    }

    Ok(result)
}
