use crate::domain::Input;
use kernel::new_types::CompanyUuid;
use kernel::new_types::UserUuid;
use utility::cache::CacheStruct;
use utility::cache::CacheUtility;
use utility::cache::CachingStrategy;
use utility::cache::OpInputTrait;
use utility::dtos::TxnNumber;

impl OpInputTrait for Input {
    type CacheUtility;

    fn check_input(
        &self,
        cache: &mut Self::CacheUtility,
    ) -> std::pin::Pin<Box<dyn Future<Output = utility::cache::OpResult<Self::CacheUtility>>>> {
        todo!()
    }

    fn user_uuid(&self) -> Option<[u8; 16]> {
        todo!()
    }
}

pub async fn fetch<Cu: CacheUtility>(
    selected_company: CompanyUuid,
    user_uuid: UserUuid,
    mut cache: CacheStruct<Cu>,
) {
    let input = Input {
        user_uuid,
        company_uuid: selected_company,
    }
    .into();

    let txn_number = TxnNumber::default();

    cache.send_to_cache_actor(CachingStrategy::ReadServerOnly, txn_number, input).await;
}
