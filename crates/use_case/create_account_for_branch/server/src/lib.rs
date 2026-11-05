use use_case_create_account_for_branch_domain::Input;
use use_case_create_account_for_branch_domain::MyResult;
use use_case_create_account_for_branch_domain::Ok;
use use_case_create_account_for_branch_domain::ReadInput;
use use_case_create_account_for_branch_domain::ReadOutput;
use use_case_prelude_server::*;

pub async fn handle_operation_generic<
    Cli: DBClient,
    DBOp: for<'a> DatabaseRead<Db<'a> = Cli::Txn<'a>, Input = ReadInput, Output = ReadOutput>
        + for<'a> DatabaseWrite<Db<'a> = Cli::Txn<'a>, Input = Ok>,
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

    let mut txn = side_effects.client.begin_transaction().await?;

    let result: Result<MyResult> = async {
        let errr = input.state_full_check::<DBOp>(&mut txn).await?;
        if errr.is_there_error() {
            return Ok(Err(errr));
        }
        let result = input.state_less_operation();
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
