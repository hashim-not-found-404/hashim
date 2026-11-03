use use_case_prelude_server::*;
use use_case_sign_up_domain::Input;
use use_case_sign_up_domain::MyResult;
use use_case_sign_up_domain::Ok;
use use_case_sign_up_domain::ReadInput;
use use_case_sign_up_domain::ReadOutput;

pub async fn handle_operation_generic<
    Cli: DBClient,
    DBOp: for<'a> DatabaseRead<Db<'a> = Cli::Txn<'a>, Input = ReadInput, Output = ReadOutput>
        + for<'a> DatabaseWrite<Db<'a> = Cli::Txn<'a>, Input = Ok>,
    Jwt: JWT<UserUuid>,
>(
    input: &Input,
    side_effects: &mut SideEffects<'_, Cli, Jwt>,
) -> Result<MyResult> {
    let errr = input.state_less_check();

    if errr.is_there_error() {
        return Ok(Err(errr));
    }

    let mut txn = side_effects.client.begin_transaction().await?;

    let result: Result<MyResult> = async {
        let errr = input.state_full_check::<DBOp>(&mut txn).await?;
        if errr.is_there_error() {
            return Ok(Err(errr));
        }
        let result = input.state_full_operation::<Jwt, Auth>(side_effects.jwt);
        DBOp::write(&mut txn, &result).await?;
        Ok(Ok(result))
    }
    .await;

    if let Ok(a) = &result {
        if a.is_ok() {
            let _ = txn.commit_transaction().await?;
        }
    } else {
        txn.rollback_transaction().await?;
    }

    result
}
