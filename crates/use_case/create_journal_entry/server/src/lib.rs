use use_case_create_journal_entry_domain::Input;
use use_case_create_journal_entry_domain::MyResult;
use use_case_create_journal_entry_domain::Ok;
use use_case_create_journal_entry_domain::ReadInput;
use use_case_create_journal_entry_domain::ReadOutput;
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
        let result = input.state_full_check::<DBOp>(&mut txn).await?;
        let Ok(ok) = result else {
            return Ok(Err(errr));
        };
        DBOp::write(&mut txn, &ok).await?;
        Ok(Ok(ok))
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
