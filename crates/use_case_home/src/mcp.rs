use crate::client::FormEntry;
use crate::client::GlobalModel;
use crate::client::Intent;
use crate::client::LocalModel;
use crate::client::Message;
use anyhow::Result;
use kernel::mcp::McpToolDef;
use kernel::mcp::error_to_mcp;
use kernel::mcp::parse_args;
use schemars::JsonSchema;
use schemars::schema_for;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use serde_json::to_value;
use std::sync::Arc;
use utility::ui_effect::Commander;
use utility::ui_effect::Replier;

// ─────────────────────────────────────────────────────────────────────────────
// Tool names
// ─────────────────────────────────────────────────────────────────────────────

const LIST_FORMS: &str = "list_forms";
const DELETE_FORM: &str = "delete_form";
const COPY_FORM: &str = "copy_form";

// ─────────────────────────────────────────────────────────────────────────────
// Typed arguments — one struct per tool. `deny_unknown_fields` makes the
// parser reject `{"indx": 3}` and `{"index": 3, "extra": 1}`; `JsonSchema`
// derives the schema the client sees from the same source of truth.
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, JsonSchema, Deserialize)]
#[serde(deny_unknown_fields)]
struct ListFormsArgs {}

#[derive(Debug, JsonSchema, Deserialize)]
#[serde(deny_unknown_fields)]
struct DeleteFormArgs {
    /// form id of the form to delete.
    form_id: u32,
}

#[derive(Debug, JsonSchema, Deserialize)]
#[serde(deny_unknown_fields)]
struct CopyFormArgs {
    /// form id of the form to copy.
    form_id: u32,
}

// ─────────────────────────────────────────────────────────────────────────────
// Typed outputs — stable shapes, decoupled from `FormEntry`'s typetag payload.
// Not deriving `JsonSchema` here because `kernel::mcp::McpToolDef` has no
// `output_schema` field yet; when it grows one, these structs are ready to
// derive it without touching `mcp_server`.
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct ListFormsOutput {
    forms: Vec<FormEntry>,
}

#[derive(Debug, Serialize)]
struct MutationOutput {
    /// `true` once the intent was accepted onto the commander queue.
    /// Does NOT mean the reducer has already applied the change — call
    /// `list_forms` again to confirm.
    is_ok: bool,
}

// ─────────────────────────────────────────────────────────────────────────────
// Tools — schemas derived from the arg structs above, not hand-written.
// Changing `DeleteFormArgs.index: usize` to `u32` (or adding a field)
// automatically changes what the client sees in `tools/list`.
// ─────────────────────────────────────────────────────────────────────────────

pub fn tools() -> Vec<McpToolDef> {
    vec![
        McpToolDef {
            name: LIST_FORMS,
            description: "List the forms currently open on the home screen.",
            input_schema: schema_for!(ListFormsArgs),
        },
        McpToolDef {
            name: DELETE_FORM,
            description: "Delete a form from the home screen by its index.",
            input_schema: schema_for!(DeleteFormArgs),
        },
        McpToolDef {
            name: COPY_FORM,
            description: "Copy a form from the home screen by its index.",
            input_schema: schema_for!(CopyFormArgs),
        },
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// Dispatch
// ─────────────────────────────────────────────────────────────────────────────

fn dummy_replier() -> Replier {
    Arc::new(|_msg| Box::pin(async { Ok(()) }))
}

pub fn call(
    tool_name: &str,
    args: Value,
    local: &LocalModel,
    _global: &impl GlobalModel,
    commander: &Commander,
) -> Result<Value> {
    match tool_name {
        LIST_FORMS => {
            let _: ListFormsArgs = parse_args(args)?;

            let output = ListFormsOutput {
                forms: local.forms.clone(),
            };
            Ok(to_value(output)?)
        }
        DELETE_FORM => {
            let DeleteFormArgs { form_id } = parse_args(args)?;

            commander.send(
                dummy_replier(),
                Message::Intent(Intent::DeleteForm(form_id.into())),
            );

            Ok(to_value(MutationOutput { is_ok: true })?)
        }
        COPY_FORM => {
            let CopyFormArgs { form_id } = parse_args(args)?;

            commander.send(
                dummy_replier(),
                Message::Intent(Intent::CopyForm(form_id.into())),
            );

            Ok(to_value(MutationOutput { is_ok: true })?)
        }
        _ => Err(error_to_mcp(
            "unknown_tool",
            format!("unknown tool: {tool_name}"),
        )),
    }
}
