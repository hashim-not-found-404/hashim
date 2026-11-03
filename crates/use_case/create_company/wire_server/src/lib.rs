use std::pin::Pin;
use use_case_create_company_database::DataBaseOp;
pub use use_case_create_company_domain::Input;
use use_case_create_company_server::handle_operation_generic;
use use_case_prelude_wire_server::*;

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
            Result::Ok(dyn_result(a))
        })
    }
}
