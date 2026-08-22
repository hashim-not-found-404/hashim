use crate::utility::cache_adapter;
use my_core::domain::cases;
use my_core::domain::utility::types::DatabaseRead;
use my_core::utility::traits::DynamicError;

pub struct S;

impl cases::create_company::DatabaseRead for S {}

impl DatabaseRead for S {
    type Db<'a> = cache_adapter::S;
    type Input = cases::create_company::ReadInput;
    type Output = cases::create_company::ReadOutput;

    async fn read(
        _db: &mut Self::Db<'_>,
        _read_input: &Self::Input,
    ) -> Result<Self::Output, DynamicError> {
        unreachable!()
    }
}
