use crate::cache::CacheOp;
use crate::client::check_input;
use crate::database::DataBaseOp;
use crate::domain::Input;
use crate::domain::Ok;
use anyhow::Result;
use cache::cache_adapter;
use database::db_client;
use kernel::server::OperationsInputServer;
use kernel::server::SideEffects;
use kernel::types::DatabaseWrite;
use std::pin::Pin;
use utility::cache::OpInputTrait1;
use utility::cache::OpOkTrait1;
use utility::cache::OpResult;
use utility::dtos::TypeOperationsError;
use utility::dtos::TypeOperationsOk;
use utility::dtos::dyn_result;

impl OperationsInputServer for Input {
    type Cli = db_client::S;

    fn handle_operation<'a>(
        self: Box<Self>,
        side_effects: &'a mut SideEffects,
        client: &'a mut Self::Cli,
    ) -> Pin<Box<dyn Future<Output = Result<Result<TypeOperationsOk, TypeOperationsError>>> + 'a>>
    {
        Box::pin(async move {
            let a = self
                .handle_operation_generic::<Self::Cli, DataBaseOp, DataBaseOp>(side_effects, client)
                .await?;

            Ok(dyn_result(a))
        })
    }
}

impl OpInputTrait1 for Input {
    type Cache = cache_adapter::S;

    fn check_input<'a>(
        &'a self,
        cache: &'a mut Self::Cache,
    ) -> Pin<Box<dyn Future<Output = OpResult> + 'a>> {
        Box::pin(async { check_input::<Self::Cache, CacheOp>(self, cache).await })
    }
}

impl OpOkTrait1 for Ok {
    type Cache = cache_adapter::S;

    fn apply_to_cache<'a>(
        &'a self,
        cache: &'a mut Self::Cache,
    ) -> Pin<Box<dyn Future<Output = ()> + 'a>> {
        Box::pin(async { CacheOp::write(cache, self).await.unwrap() })
    }
}
