use cache::cache_adapter;
use std::pin::Pin;
use utility::cache::TraitOperationCacheInput;
use utility::cache::TypeOperationClientResult;

#[derive(Debug, Clone)]
pub(crate) struct WrapperUseCaseGetAllAccounts(pub(crate) use_case_get_all_accounts::domain::Input);

impl TraitOperationCacheInput for WrapperUseCaseGetAllAccounts {
    type Cache = cache_adapter::S;

    fn check_input<'a>(
        &'a self,
        cache: &'a mut Self::Cache,
    ) -> Pin<Box<dyn Future<Output = TypeOperationClientResult> + 'a>> {
        Box::pin(async {
            use_case_get_all_accounts::client::check_input::<
                Self::Cache,
                use_case_get_all_accounts::cache::CacheOp,
            >(&self.0, cache)
            .await
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct WrapperUseCaseCreateAccount(pub(crate) use_case_create_account::domain::Input);

impl TraitOperationCacheInput for WrapperUseCaseCreateAccount {
    type Cache = cache_adapter::S;

    fn check_input<'a>(
        &'a self,
        cache: &'a mut Self::Cache,
    ) -> Pin<Box<dyn Future<Output = TypeOperationClientResult> + 'a>> {
        Box::pin(async {
            use_case_create_account::client::check_input::<
                Self::Cache,
                use_case_create_account::cache::CacheOp,
            >(&self.0, cache)
            .await
        })
    }
}
