use crate::domain::MyResult;
use anyhow::Result;
use kernel::make_auth_check;
use kernel::server::DBTransaction;
use kernel::types::MyErrorTrait;
use kernel::types::UserUuidError;
use patterns::make_server_handler_with_write;

make_server_handler_with_write! {
    let mut errr = input.state_less_check();
    make_auth_check!(side_effects, input, errr);

    if errr.is_there_error() {
        return Ok(Err(errr).into());
    }

    let mut txn = client.begin_transaction().await?;

    let result: Result<MyResult> = async {
        let errr = input.state_full_check::<DBReader>(&mut txn).await?;

        if errr.is_there_error() {
            return Ok(Err(errr).into());
        }

        let result = input.state_less_operation();
        DBWrite::write(&mut txn, &result).await?;
        Ok(Ok(result).into())
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
