use crate::client::FormInfo;
use crate::client::GlobalModel;
use crate::client::Intent;
use crate::client::Message;
use anyhow::Result;
use anyhow::bail;
use kernel::mcp::McpToolDef;
use serde_json::Value;
use serde_json::json;
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

/// Synchronous entry point: reads the model via the trait, sends intents via
/// the (already-constructed) `Commander`. No knowledge of `app_root_client`.
pub fn call<G: GlobalModel>(
    tool_name: &str,
    args: Value,
    global: &G,
    commander: &Commander,
) -> Result<Value> {
    match tool_name {
        "list_forms" => {
            let forms: Vec<FormInfo> = global.forms();
            Ok(serde_json::to_value(forms)?)
        }
        "delete_form" => {
            let index =
                args.get("index")
                    .and_then(Value::as_u64)
                    .ok_or_else(|| anyhow::anyhow!("missing 'index'"))? as usize;
            commander.send(dummy_replier(), Message::Intent(Intent::DeleteForm(index)));
            Ok(json!({ "ok": true }))
        }
        "copy_form" => {
            let index =
                args.get("index")
                    .and_then(Value::as_u64)
                    .ok_or_else(|| anyhow::anyhow!("missing 'index'"))? as usize;
            commander.send(dummy_replier(), Message::Intent(Intent::CopyForm(index)));
            Ok(json!({ "ok": true }))
        }
        _ => bail!("unknown tool: {tool_name}"),
    }
}
