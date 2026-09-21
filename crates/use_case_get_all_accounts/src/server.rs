use kernel::make_auth_check;
use kernel::types::MyErrorTrait;
use kernel::types::UserUuidError;
use patterns::make_server_handler_without_write;

make_server_handler_without_write! {
    let mut errr = input.state_less_check();
    make_auth_check!(side_effects, input, errr);

    if errr.is_there_error() {
        return Ok(Err(errr));
    }

    let ok = input.state_full_operation::<DBReader>(client).await?;

    Ok(Ok(ok))
}
