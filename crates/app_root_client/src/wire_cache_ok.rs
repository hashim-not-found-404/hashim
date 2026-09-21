use cache::cache_adapter;
use kernel::types::DatabaseWrite;
use std::pin::Pin;
use utility::cache::TraitOperationCacheOk;

#[derive(Debug, Clone)]
pub(crate) struct WrapperUseCaseGetAllAccounts(pub(crate) use_case_get_all_accounts::domain::Ok);

impl TraitOperationCacheOk for WrapperUseCaseGetAllAccounts {
    type Cache = cache_adapter::S;

    fn apply_to_cache<'a>(
        &'a self,
        cache: &'a mut Self::Cache,
    ) -> Pin<Box<dyn Future<Output = ()> + 'a>> {
        Box::pin(async {
            use_case_get_all_accounts::cache::CacheOp::write(cache, &self.0)
                .await
                .unwrap()
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct WrapperUseCaseCreateAccount(pub(crate) use_case_create_account::domain::Ok);

impl TraitOperationCacheOk for WrapperUseCaseCreateAccount {
    type Cache = cache_adapter::S;

    fn apply_to_cache<'a>(
        &'a self,
        cache: &'a mut Self::Cache,
    ) -> Pin<Box<dyn Future<Output = ()> + 'a>> {
        Box::pin(async {
            use_case_create_account::cache::CacheOp::write(cache, &self.0)
                .await
                .unwrap()
        })
    }
}
