use crate::model::TypeGlobalModel;
use crate::utils::COMMANDER;
use crate::utils::with_model;
use anyhow::Result;
use kernel::mcp::McpProvider;
use kernel::mcp::McpToolDef;
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;

struct HomeProvider;

impl McpProvider for HomeProvider {
    fn tools(&self) -> Vec<McpToolDef> {
        use_case_home::mcp::tools()
    }

    fn call(
        &self,
        tool_name: &str,
        args: Value,
    ) -> Pin<Box<dyn Future<Output = Result<Value>> + Send>> {
        let tool_name = tool_name.to_owned();
        Box::pin(async move {
            with_model(|model: &TypeGlobalModel| {
                use_case_home::mcp::call(&tool_name, args, &model.page_home, model, &*COMMANDER)
            })
        })
    }
}

pub(crate) fn get_all_providers() -> Vec<Box<dyn McpProvider>> {
    vec![Box::new(HomeProvider) as Box<dyn McpProvider>]
}
