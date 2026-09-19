use crate::database::Db;
use crate::domain::Input;
use anyhow::Result;
use database::db_client;
use kernel::server::OperationsInputServer;
use kernel::server::SideEffects;
use std::pin::Pin;
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
            let a = self.handle_operation_generic::<Self::Cli, Db>(side_effects, client).await?;

            Ok(dyn_result(a))
        })
    }
}
