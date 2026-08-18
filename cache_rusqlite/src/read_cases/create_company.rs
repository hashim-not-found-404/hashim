use crate::utility::cache_adapter;
use my_core::accounting_domain::cases;
use my_core::utility::traits;

pub struct S;

impl cases::create_company::DatabaseRead for S {
    type Db<'a> = cache_adapter::S;

    async fn read(
        _: &mut Self::Db<'_>,
        _: &cases::create_company::ReadInput,
    ) -> Result<cases::create_company::ReadOutput, traits::DynamicError> {
        unreachable!()
    }
}
