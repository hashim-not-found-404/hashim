use crate::utility::cache_adapter;
use my_core::domain::use_cases;
use my_core::domain::utility::types::DatabaseRead;
use my_core::domain::utility::types::DatabaseWrite;
use my_core::utility::traits::DynamicError;

pub struct S;

impl use_cases::get_all_accounts::DatabaseRead for S {}

impl DatabaseRead for S {
    type Db<'a> = cache_adapter::S;
    type Input = use_cases::get_all_accounts::ReadInput;
    type Output = use_cases::get_all_accounts::ReadOutput;

    async fn read(
        db: &mut Self::Db<'_>,
        input: &Self::Input,
    ) -> Result<Self::Output, DynamicError> {
        todo!()
    }
}

impl DatabaseWrite for S {
    type Db<'a> = cache_adapter::S;
    type Input = use_cases::get_all_accounts::Ok;

    async fn write(txn: &mut Self::Db<'_>, input: &Self::Input) -> Result<(), DynamicError> {
        todo!()
    }
}
