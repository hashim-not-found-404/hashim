use std::future::Future;
use std::pin::Pin;
pub use use_case_get_all_journals_domain::Error;
pub use use_case_get_all_journals_domain::Input;
pub use use_case_get_all_journals_domain::Ok;
use use_case_get_all_journals_mcp::call;
use use_case_get_all_journals_mcp::tools;
use use_case_prelude_wire_client::*;

make_client_wrapper_cache!(get_all_journals);

pub trait ModelAccess {
    fn user_uuid(&self) -> Option<UserUuid>;
}

pub struct TypeMcpProvider;

impl<Mdl> TraitMcpProvider<Mdl> for TypeMcpProvider
where
    Mdl: Model + ModelAccess + 'static,
{
    fn tools(&self) -> Vec<McpToolDef> {
        tools()
    }

    fn call<'a>(
        &'a self,
        tool_name: &str,
        args: Value,
        model: &'a Mdl,
        commander: &'a Commander,
        cache: &'a CacheStruct,
    ) -> Pin<Box<dyn Future<Output = Result<Value>> + Send + 'a>> {
        let tool_name = tool_name.to_owned();
        Box::pin(async move {
            let input = Input {
                user_uuid: model.user_uuid().context(error_to_mcp(
                    "not_authenticated",
                    "user_uuid is not set in the model",
                ))?,
            };

            call(CallInput {
                tool_name: &tool_name,
                args,
                local: &(),
                global: &input,
                commander,
                cache,
            })
            .await
        })
    }
}
