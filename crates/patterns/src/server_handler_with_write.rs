use proc_macro::TokenStream;
use quote::quote;

pub fn my_macro(input: TokenStream) -> TokenStream {
    let body: proc_macro2::TokenStream = input.into();

    quote! {
        pub async fn handle_operation_generic<
            Cli: kernel::server::DBClient,
            DBReader: for<'a> kernel::types::DatabaseRead<
                Db<'a> = Cli::Txn<'a>,
                Input = crate::domain::ReadInput,
                Output = crate::domain::ReadOutput
            >,
            DBWrite: for<'a> kernel::types::DatabaseWrite<
                Db<'a> = Cli::Txn<'a>,
                Input = crate::domain::Ok
            >,
        >(
            input: &crate::domain::Input,
            side_effects: &mut kernel::server::SideEffects,
            client: &mut Cli,
        ) -> anyhow::Result<crate::domain::MyResult> {#body}
    }
    .into()
}
