use crate::domain::Ok;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use anyhow::Result;
use cache::cache_adapter;
use kernel::types::DatabaseRead;
use kernel::types::DatabaseWrite;

pub struct S;

impl DatabaseRead for S {
    type Db = cache_adapter::S;
    type Input = ReadInput;
    type Output = ReadOutput;

    async fn read(db: &mut Self::Db, input: &Self::Input) -> Result<Self::Output> {
        todo!()
    }
}

impl DatabaseWrite for S {
    type Db = cache_adapter::S;
    type Input = Ok;

    async fn write(txn: &mut Self::Db, input: &Self::Input) -> Result<()> {
        todo!()
    }
}
