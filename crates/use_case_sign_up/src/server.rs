use crate::domain::Input;
use crate::domain::MyResult;
use crate::domain::Ok;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use anyhow::Result;
use infrastructure::jwt::JWT;
use kernel::server::DBClient;
use kernel::server::DBTransaction;
use kernel::server::SideEffects;
use kernel::types::DatabaseRead;
use kernel::types::DatabaseWrite;
use kernel::types::MyErrorTrait;

pub async fn handle_operation_generic<
    Cli: DBClient,
    DBReader: for<'a> DatabaseRead<Db<'a> = Cli::Txn<'a>, Input = ReadInput, Output = ReadOutput>,
    DBWrite: for<'a> DatabaseWrite<Db<'a> = Cli::Txn<'a>, Input = Ok>,
    Jwt: JWT,
>(
    input: &Input,
    side_effects: &mut SideEffects,
    client: &mut Cli,
    jwt: &mut Jwt,
) -> Result<MyResult> {
    let errr = input.state_less_check();

    if errr.is_there_error() {
        return Ok(Err(errr));
    }

    let mut txn = client.begin_transaction().await?;

    let result: Result<MyResult> = async {
        let errr = input.state_full_check::<DBReader>(&mut txn).await?;
        if errr.is_there_error() {
            return Ok(Err(errr));
        }
        let result = input.state_full_operation(jwt);
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
