use crate::cache::CacheOp;
use crate::client::check_input;
use crate::database::DataBaseOp;
use crate::domain::Input;
use crate::domain::Ok;
use anyhow::Result;
use cache::cache_adapter;
use database::db_client;
use kernel::server::SideEffects;
use kernel::server::TraitOperationServerInput;
use kernel::types::DatabaseWrite;
use std::pin::Pin;
use utility::cache::TraitOperationCacheInput;
use utility::cache::TraitOperationCacheOk;
use utility::cache::TypeOperationClientResult;
use utility::dtos::TypeOperationDTOError;
use utility::dtos::TypeOperationDTOOk;
use utility::dtos::dyn_result;

impl TraitOperationServerInput for Input {
    type Cli = db_client::S;

    fn handle_operation<'a>(
        self: Box<Self>,
        side_effects: &'a mut SideEffects,
        client: &'a mut Self::Cli,
    ) -> Pin<Box<dyn Future<Output = Result<Result<TypeOperationDTOOk, TypeOperationDTOError>>> + 'a>>
    {
        Box::pin(async move {
            let a = self
                .handle_operation_generic::<Self::Cli, DataBaseOp, DataBaseOp>(side_effects, client)
                .await?;

            Ok(dyn_result(a))
        })
    }
}

impl TraitOperationCacheInput for Input {
    type Cache = cache_adapter::S;

    fn check_input<'a>(
        &'a self,
        cache: &'a mut Self::Cache,
    ) -> Pin<Box<dyn Future<Output = TypeOperationClientResult> + 'a>> {
        Box::pin(async { check_input::<Self::Cache, CacheOp>(self, cache).await })
    }
}

impl TraitOperationCacheOk for Ok {
    type Cache = cache_adapter::S;

    fn apply_to_cache<'a>(
        &'a self,
        cache: &'a mut Self::Cache,
    ) -> Pin<Box<dyn Future<Output = ()> + 'a>> {
        Box::pin(async { CacheOp::write(cache, self).await.unwrap() })
    }
}
