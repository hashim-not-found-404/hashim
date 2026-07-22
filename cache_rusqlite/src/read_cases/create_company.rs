use crate::utility::cache_adapter;
use crate::utility::utils::MyUuidConverter;
use my_core::accounting_domain::cases::{self};
use my_core::accounting_domain::utility::types;
use my_core::utility::traits;
use rusqlite::params;
use std::str::FromStr;

pub struct S;

impl cases::create_company::DatabaseRead for S {
    type Db<'a> = cache_adapter::S;

    async fn read(
        db: &mut Self::Db<'_>,
        read_input: &cases::create_company::ReadInput,
    ) -> Result<cases::create_company::ReadOutput, traits::DynamicError> {
        todo!()
    }
}
