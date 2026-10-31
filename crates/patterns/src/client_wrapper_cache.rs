use proc_macro::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::Ident;
use syn::parse_macro_input;

pub fn my_macro(input: TokenStream) -> TokenStream {
    let use_case_name = parse_macro_input!(input as Ident);
    let crate_name = format_ident!("use_case_{}", use_case_name);
    let check_mod_name = format_ident!("cache_check_use_case_{}", use_case_name);
    let write_mod_name = format_ident!("cache_write_use_case_{}", use_case_name);

    quote! {
        mod #check_mod_name {
            use anyhow::Result;
            use cache::cache_adapter;
            use std::pin::Pin;
            use #crate_name::cache::CacheOp;
            use #crate_name::client::check_input;
            use #crate_name::domain::Input;
            use utility::cache::TraitOperationCacheInput;
            use utility::cache::TypeOperationClientResult;

            #[derive(Debug, Clone)]
            pub(crate) struct Wrapper(pub(crate) Input);

            impl TraitOperationCacheInput for Wrapper {
                type Cache = cache_adapter::CacheAdapter;

                fn check_input<'a>(
                    &'a self,
                    cache: &'a mut Self::Cache,
                ) -> Pin<Box<dyn Future<Output = Result<TypeOperationClientResult>> + 'a>> {
                    Box::pin(async { check_input::<Self::Cache, CacheOp>(&self.0, cache).await })
                }
            }
        }

        mod #write_mod_name {
            use anyhow::Result;
            use cache::cache_adapter;
            use kernel::types::DatabaseWrite;
            use std::pin::Pin;
            use #crate_name::cache::CacheOp;
            use #crate_name::domain::Ok;
            use utility::cache::TraitOperationCacheOk;

            #[derive(Debug, Clone)]
            pub(crate) struct Wrapper(pub(crate) Ok);

            impl TraitOperationCacheOk for Wrapper {
                type Cache = cache_adapter::CacheAdapter;

                fn apply_to_cache<'a>(
                    &'a self,
                    cache: &'a mut Self::Cache,
                ) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>> {
                    Box::pin(async { CacheOp::write(cache, &self.0).await })
                }
            }
        }
    }
    .into()
}
