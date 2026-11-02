use use_case_prelude_server::*;
use use_case_sign_in_domain::Input;
use use_case_sign_in_domain::MyResult;
use use_case_sign_in_domain::ReadInput;
use use_case_sign_in_domain::ReadOutput;

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
