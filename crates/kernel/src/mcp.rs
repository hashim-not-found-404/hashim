use anyhow::Result;
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;

pub struct McpToolDef {
    pub name: &'static str,
    pub description: &'static str,
    pub input_schema: Value,
}

pub trait McpProvider: Send + Sync + 'static {
    fn tools(&self) -> Vec<McpToolDef>;
    fn call(
        &self,
        tool_name: &str,
        args: Value,
    ) -> Pin<Box<dyn Future<Output = Result<Value>> + Send>>;
}
