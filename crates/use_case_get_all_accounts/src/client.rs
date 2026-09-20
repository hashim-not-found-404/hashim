use crate::domain::Error;
use crate::domain::Input;
use crate::domain::Ok;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use kernel::client::Cache;
use kernel::new_types::CompanyUuid;
use kernel::new_types::UserUuid;
use kernel::types::DatabaseRead;
use kernel::types::MyErrorTrait;
use std::ops::Deref;
use std::sync::Arc;
use utility::cache::CacheStruct;
use utility::cache::CachingStrategy;
use utility::cache::Subscribe;
use utility::cache::TraitOperationClientError;
use utility::cache::TraitOperationClientInput;
use utility::cache::TraitOperationClientOk;
use utility::cache::TypeOperationClientError;
use utility::cache::TypeOperationClientInput;
use utility::cache::TypeOperationClientOk;
use utility::cache::TypeOperationClientResult;
use utility::dtos::TxnNumber;

impl TraitOperationClientOk for Ok {
    fn subs_to_poke(&self) -> &'static [Subscribe] {
        todo!()
    }
}

impl TraitOperationClientError for Error {
    fn subs_to_poke(&self) -> &'static [Subscribe] {
        todo!()
    }
}

impl TraitOperationClientInput for Input {
    fn user_uuid(&self) -> Option<[u8; 16]> {
        Some(*self.user_uuid.deref().deref())
    }
}

pub async fn check_input<
    Ch: Cache,
    DBReader: for<'a> DatabaseRead<Db<'a> = Ch, Input = ReadInput, Output = ReadOutput>,
>(
    input: &Input,
    cache: &mut Ch,
) -> TypeOperationClientResult {
    let errr = input.state_less_check();

    if errr.is_there_error() {
        return Err(Box::new(errr));
    }

    let ok = input.state_full_operation::<DBReader>(cache).await.unwrap();

    Ok(Box::new(ok))
}

pub async fn fetch(selected_company: CompanyUuid, user_uuid: UserUuid, mut cache: CacheStruct) {
    let input = Input {
        user_uuid,
        company_uuid: selected_company,
    };

    let input: TypeOperationClientInput = Arc::new(input);

    let txn_number = TxnNumber::default();

    cache.send_to_cache_actor(CachingStrategy::ReadServerOnly, txn_number, input).await;
}
