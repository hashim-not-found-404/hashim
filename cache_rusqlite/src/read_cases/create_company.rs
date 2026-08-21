use crate::utility::cache_adapter;
use my_core::accounting_domain::cases;
use my_core::accounting_domain::utility::types::DatabaseRead;
use my_core::utility::traits;

pub struct S;

impl cases::create_company::DatabaseRead for S {}

impl DatabaseRead for S {
    type Db<'a> = cache_adapter::S;
    type Error = traits::DynamicError;
    type Input = cases::create_company::ReadInput;
    type Output = cases::create_company::ReadOutput;

    async fn read(
        _db: &mut Self::Db<'_>,
        _read_input: &Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        unreachable!()
    }
}
