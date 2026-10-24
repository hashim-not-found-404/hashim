use crate::client::GlobalModel;
use crate::client::Intent;
use crate::client::LocalModel;
use crate::client::Message;
use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;
use kernel::mcp::McpToolDef;
use serde_json::Value;
use serde_json::json;
use serde_json::to_value;
use std::sync::Arc;
use utility::ui_effect::Commander;
use utility::ui_effect::Replier;

pub fn tools() -> Vec<McpToolDef> {
    vec![
        McpToolDef {
            name: "list_forms",
            description: "List the forms currently open on the home screen.",
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpToolDef {
            name: "delete_form",
            description: "Delete a form from the home screen by its index.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "index": { "type": "integer", "minimum": 0 }
                },
                "required": ["index"]
            }),
        },
        McpToolDef {
            name: "copy_form",
            description: "Copy a form from the home screen by its index.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "index": { "type": "integer", "minimum": 0 }
                },
                "required": ["index"]
            }),
        },
    ]
}

fn dummy_replier() -> Replier {
    Arc::new(|_msg| Box::pin(async { Ok(()) }))
}

pub fn call(
    tool_name: &str,
    args: Value,
    local: &LocalModel,
    global: &impl GlobalModel,
    commander: &Commander,
) -> Result<Value> {
    match tool_name {
        "list_forms" => Ok(to_value(local.forms.clone())?),
        "delete_form" => {
            let index = args
                .get("index")
                .and_then(Value::as_u64)
                .ok_or_else(|| anyhow!("missing 'index'"))? as usize;

            commander.send(dummy_replier(), Message::Intent(Intent::DeleteForm(index)));
            Ok(json!({ "ok": true }))
        }
        "copy_form" => {
            let index = args
                .get("index")
                .and_then(Value::as_u64)
                .ok_or_else(|| anyhow!("missing 'index'"))? as usize;

            commander.send(dummy_replier(), Message::Intent(Intent::CopyForm(index)));
            Ok(json!({ "ok": true }))
        }
        _ => bail!("unknown tool: {tool_name}"),
    }
}
