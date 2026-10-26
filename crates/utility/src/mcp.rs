use crate::ui_effect::Commander;
use anyhow::Error;
use anyhow::Result;
use anyhow::anyhow;
use schemars::Schema;
use serde::de::DeserializeOwned;
use serde_json::Value;
use serde_json::from_value;
use serde_json::json;
use std::future::Future;
use std::pin::Pin;

pub struct McpToolDef {
    pub name: &'static str,
    pub description: &'static str,
    pub input_schema: Schema,
}

pub fn parse_args<T: DeserializeOwned>(args: Value) -> Result<T> {
    from_value(args).map_err(|e| error_to_mcp("invalid_args", e.to_string()))
}

pub fn error_to_mcp(code: &str, message: impl Into<String>) -> Error {
    anyhow!(
        "{}",
        json!({ "code": code, "message": message.into() }).to_string()
    )
}

pub trait TraitMcpProvider: Send + Sync + 'static {
    fn tools(&self) -> Vec<McpToolDef>;
    fn call(
        &self,
        tool_name: &str,
        args: Value,
    ) -> Pin<Box<dyn Future<Output = Result<Value>> + Send>>;
}

pub struct CallInput<'a, LocalModel, GlobalModel> {
    pub tool_name: &'a str,
    pub args: Value,
    pub local: &'a LocalModel,
    pub global: &'a GlobalModel,
    pub commander: &'a Commander,
}
