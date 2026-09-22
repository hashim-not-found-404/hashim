use crate::domain::Input;
use crate::domain::MyResult;
use crate::domain::Ok;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use anyhow::Result;
use kernel::make_auth_check;
use kernel::server::DBClient;
use kernel::server::DBTransaction;
use kernel::server::SideEffects;
use kernel::types::DatabaseRead;
use kernel::types::DatabaseWrite;
use kernel::types::MyErrorTrait;
use kernel::types::UserUuidError;

pub async fn handle_operation_generic<
    Cli: DBClient,
    DBReader: for<'a> DatabaseRead<Db<'a> = Cli::Txn<'a>, Input = ReadInput, Output = ReadOutput>,
    DBWrite: for<'a> DatabaseWrite<Db<'a> = Cli::Txn<'a>, Input = Ok>,
>(
    input: &Input,
    side_effects: &mut SideEffects,
    client: &mut Cli,
) -> Result<MyResult> {
    let mut errr = input.state_less_check();
    make_auth_check!(side_effects, input, errr);

    if errr.is_there_error() {
        return Ok(Err(errr));
    }

    let mut txn = client.begin_transaction().await?;

    let result: Result<MyResult> = async {
        let errr = input.state_full_check::<DBReader>(&mut txn).await?;
        if errr.is_there_error() {
            return Ok(Err(errr));
        }
        let result = input.state_less_operation();
        DBWrite::write(&mut txn, &result).await?;
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
