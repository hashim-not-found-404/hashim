use crate::domain::Input;
use crate::domain::MyResult;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use use_case_prelude_server::*;

pub async fn handle_operation_generic<
    Cli: DBClient,
    DBReader: for<'a> DatabaseRead<Db<'a> = Cli, Input = ReadInput, Output = ReadOutput>,
    Jwt: JWT<UserUuid>,
>(
    input: &Input,
    side_effects: &mut SideEffects<'_, Cli, Jwt>,
) -> Result<MyResult> {
    let mut errr = input.state_less_check();
    make_auth_check!(side_effects, input, errr);

    if errr.is_there_error() {
        return Ok(Err(errr));
    }

    input
        .state_full_operation::<DBReader>(side_effects.client)
        .await
}
