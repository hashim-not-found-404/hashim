use anyhow::Result;
use deadpool_postgres::Transaction;
use kernel::server::AtCommit;
use kernel::server::DBTransaction;
use tokio_postgres::error::SqlState;
use utility::types::LogError;

pub struct S<'a> {
    pub(crate) txn: Transaction<'a>,
}

impl DBTransaction for S<'_> {
    async fn commit_transaction(self) -> Result<Result<(), AtCommit>> {
        match self.txn.commit().await {
            Ok(_) => Ok(Ok(())),
            Err(e) => {
                if get_sql_state(&e) == SqlState::T_R_SERIALIZATION_FAILURE {
                    return Ok(Err(AtCommit::DataIsChanged));
                }
                Err(e.into())
            }
        }
    }

    async fn rollback_transaction(self) -> Result<()> {
        self.txn.rollback().await.log()?;
        Ok(())
    }
}

fn get_sql_state(error: &tokio_postgres::Error) -> SqlState {
    error.as_db_error().unwrap().code().clone()
}
