pub use anyhow::Result;
pub use database::db_client::DbClientAdapter;
pub use infrastructure::jwt::Jwt;
pub use kernel::server::SideEffects;
pub use kernel::server::TraitOperationServerInput;
pub use paste::paste;
pub use utility::dtos::TypeOperationDTOError;
pub use utility::dtos::TypeOperationDTOOk;
pub use utility::dtos::dyn_result;

#[macro_export]
macro_rules! make_server_handler {
    ($name:ident) => {
        $crate::paste! {
            use std::pin::Pin;
            use [<use_case_ $name _database>]::DataBaseOp;
            pub use [<use_case_ $name _domain>]::Input;
            use [<use_case_ $name _server>]::handle_operation_generic;

            pub struct Wrapper(pub Input);

            impl TraitOperationServerInput for Wrapper {
                type Cli = DbClientAdapter;
                type Jwt = Jwt;

                fn handle_operation<'a>(
                    self: Box<Self>,
                    side_effects: &'a mut SideEffects<'_, Self::Cli, Self::Jwt>,
                ) -> Pin<Box<dyn Future<Output = Result<Result<TypeOperationDTOOk, TypeOperationDTOError>>> + 'a>>
                {
                    Box::pin(async move {
                        let a =
                            handle_operation_generic::<Self::Cli, DataBaseOp, Self::Jwt>(&self.0, side_effects)
                                .await?;
                        Ok(dyn_result(a))
                    })
                }
            }
        }
    };
}
