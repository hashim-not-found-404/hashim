pub use anyhow::Context;
pub use anyhow::Result;
pub use cache::cache_adapter::CacheAdapter;
pub use cache::utils::MyUuidConverter;
pub use dioxus::prelude::*;
pub use kernel::new_types::BranchUuid;
pub use kernel::new_types::CompanyUuid;
pub use kernel::new_types::UserUuid;
pub use kernel::new_types::UuidType;
pub use kernel::types::DatabaseWrite;
pub use paste::paste;
pub use serde_json::Value;
pub use typetag;
pub use typetag::serde;
pub use utility::cache::CacheStruct;
pub use utility::cache::TraitOperationCacheInput;
pub use utility::cache::TraitOperationCacheOk;
pub use utility::cache::TypeOperationClientResult;
pub use utility::mcp::CallInput;
pub use utility::mcp::McpToolDef;
pub use utility::mcp::TraitMcpProvider;
pub use utility::mcp::error_to_mcp;
pub use utility::ui_effect::ApplierTrait;
pub use utility::ui_effect::Commander;
pub use utility::ui_effect::EffectorTrait;
pub use utility::ui_effect::MessageTrait;
pub use utility::ui_effect::Model;
pub use utility::ui_effect::Replier;
pub use utility::ui_effect::UiContext;
pub use utility::ui_effect::UpdaterTrait;

#[macro_export]
macro_rules! make_client_wrapper_cache {
    ($use_case_name:ident) => {
        $crate::paste! {
            pub mod cache_check {
                use std::boxed::Box;
                use std::future::Future;
                use std::pin::Pin;
                use [<use_case_ $use_case_name _cache>]::CacheOp;
                use [<use_case_ $use_case_name _client>]::check_input;
                use [<use_case_ $use_case_name _domain>]::Input;
                use use_case_prelude_wire_client::*;

                #[derive(Debug, Clone)]
                pub struct Wrapper(pub Input);

                impl TraitOperationCacheInput for Wrapper {
                    type Cache = CacheAdapter;
                    fn check_input<'a>(
                        &'a self,
                        cache: &'a mut Self::Cache,
                    ) -> Pin<Box<dyn Future<Output = Result<TypeOperationClientResult>> + 'a>> {
                        Box::pin(async { check_input::<Self::Cache, CacheOp>(&self.0, cache).await })
                    }
                }
            }

            pub mod cache_write {
                use std::boxed::Box;
                use std::future::Future;
                use std::pin::Pin;
                use [<use_case_ $use_case_name _cache>]::CacheOp;
                use [<use_case_ $use_case_name _domain>]::Ok;
                use use_case_prelude_wire_client::*;

                #[derive(Debug, Clone)]
                pub struct Wrapper(pub Ok);

                impl TraitOperationCacheOk for Wrapper {
                    type Cache = CacheAdapter;
                    fn apply_to_cache<'a>(
                        &'a self,
                        cache: &'a mut Self::Cache,
                    ) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>> {
                        Box::pin(async { CacheOp::write(cache, &self.0).await })
                    }
                }
            }
        }
    };
}
