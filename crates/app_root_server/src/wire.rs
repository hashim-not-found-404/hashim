use anyhow::Result;
use database::db_client;
use kernel::server::SideEffects;
use kernel::server::TraitOperationServerInput;
use std::pin::Pin;
use utility::dtos::TypeOperationDTOError;
use utility::dtos::TypeOperationDTOOk;
use utility::dtos::dyn_result;

pub(crate) struct WrapperUseCaseCreateAccount(pub(crate) use_case_create_account::domain::Input);

impl TraitOperationServerInput for WrapperUseCaseCreateAccount {
    type Cli = db_client::S;

    fn handle_operation<'a>(
        self: Box<Self>,
        side_effects: &'a mut SideEffects,
        client: &'a mut Self::Cli,
    ) -> Pin<Box<dyn Future<Output = Result<Result<TypeOperationDTOOk, TypeOperationDTOError>>> + 'a>>
    {
        Box::pin(async move {
            let a = use_case_create_account::server::handle_operation_generic::<
                Self::Cli,
                use_case_create_account::database::DataBaseOp,
                use_case_create_account::database::DataBaseOp,
            >(&self.0, side_effects, client)
            .await?;

            Ok(dyn_result(a))
        })
    }
}

pub(crate) struct WrapperUseCaseGetAllAccounts(pub(crate) use_case_get_all_accounts::domain::Input);

impl TraitOperationServerInput for WrapperUseCaseGetAllAccounts {
    type Cli = db_client::S;

    fn handle_operation<'a>(
        self: Box<Self>,
        side_effects: &'a mut SideEffects,
        client: &'a mut Self::Cli,
    ) -> Pin<Box<dyn Future<Output = Result<Result<TypeOperationDTOOk, TypeOperationDTOError>>> + 'a>>
    {
        Box::pin(async move {
            let a = use_case_get_all_accounts::server::handle_operation_generic::<
                Self::Cli,
                use_case_get_all_accounts::database::DataBaseOp,
            >(&self.0, side_effects, client)
            .await?;

            Ok(dyn_result(a))
        })
    }
}
