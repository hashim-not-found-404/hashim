//! `rmcp::ServerHandler` for the `create_company` form only.
//!
//! Every mutation goes through the same `Commander` the UI uses; every read
//! goes through the same `with_model` the UI uses. There is one model, one
//! reducer pipeline, one intent bus.
//!
//! Only `FormLocalModel::CreateCompany` forms are visible to the AI. Other
//! kinds are ignored by read tools and rejected by `update_field` /
//! `submit_form` / `consent_form`; `delete_form` still works on them.

use crate::mcp::McpBridge;
use crate::use_case_home::client::FormEntry;
use crate::use_case_home::client::FormId;
use crate::use_case_home::client::FormLocalModel;
use crate::use_case_home::client::FormsToAdd;
use crate::use_case_home::client::Intent as HomeIntent;
use crate::use_case_home::client::Message as HomeMessage;
use anyhow::anyhow;
use rmcp::ErrorData;
use rmcp::ServerHandler;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::model::Content;
use rmcp::model::ServerCapabilities;
use rmcp::model::ServerInfo;
use rmcp::tool;
use rmcp::tool_handler;
use rmcp::tool_router;
use schemars::JsonSchema;
use schemars::Schema;
use schemars::SchemaGenerator;
use serde::Deserialize;
use serde::Serialize;
use serde_json::to_string;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::sync::Arc;
use use_case_create_company::client::LocalModel as CompanyLocalModel;
use utility::ui_effect::MessageTrait;

// -----------------------------------------------------------------------------
// DTOs
// -----------------------------------------------------------------------------

/// Schema generator for the `UpdateFieldArgs::value` field.
///
/// `serde_json::Value` normally generates the bare schema `true` (accept any
/// JSON value). That is legal JSON Schema but is rejected or mishandled by
/// stricter MCP clients (e.g. the MCP Inspector flags it as
/// "Bare `true` used where a schema object is expected").
///
/// In practice this field is always a string: `company_name` is parsed into a
/// `String`, and `currency` is parsed into an enum whose wire form is a string
/// (`"USD"` / `"IQD"`). Declaring it as a string is both accurate and gives a
/// portable schema that strict clients render as a plain text input.
fn json_value_schema(_generator: &mut SchemaGenerator) -> Schema {
    schemars::json_schema!({
        "type": "string",
        "description": "New value for the field. `company_name` expects a string (e.g. \"Acme Corp\"); `currency` expects \"USD\" or \"IQD\"."
    })
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub(crate) struct FormSnapshot {
    pub form_id: u32,
    pub loading: bool,
    pub show_dialog: bool,
    pub fields: BTreeMap<String, serde_json::Value>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub(crate) struct OpenFormResponse {
    pub form_id: u32,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub(crate) struct UpdateFieldArgs {
    pub form_id: u32,
    pub field: String,
    #[schemars(schema_with = "json_value_schema")]
    pub value: serde_json::Value,
}

// -----------------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------------

fn form_id_to_u32(id: FormId) -> u32 {
    serde_json::to_value(id)
        .ok()
        .and_then(|v| v.as_u64())
        .map(|v| v as u32)
        .unwrap_or(0)
}

fn u32_to_form_id(n: u32) -> Option<FormId> {
    serde_json::from_value(serde_json::Value::from(n)).ok()
}

fn with_company_lm<R>(form: &FormLocalModel, f: impl FnOnce(&CompanyLocalModel) -> R) -> Option<R> {
    if let FormLocalModel::CreateCompany(lm) = form {
        Some(f(lm))
    } else {
        None
    }
}

fn try_snapshot(entry: &FormEntry) -> Option<FormSnapshot> {
    let lm = match &entry.form {
        FormLocalModel::CreateCompany(lm) => lm,
        _ => return None,
    };
    Some(FormSnapshot {
        form_id: form_id_to_u32(entry.id),
        loading: use_case_create_company::mcp::is_loading(lm),
        show_dialog: use_case_create_company::mcp::show_dialog(lm),
        fields: use_case_create_company::mcp::extract_fields(lm),
        errors: use_case_create_company::mcp::errors(lm),
    })
}

fn current_form_ids(m: &crate::model::TypeGlobalModel) -> HashSet<u32> {
    m.page_home
        .forms
        .iter()
        .map(|e| form_id_to_u32(e.id))
        .collect()
}

// -----------------------------------------------------------------------------
// Server
// -----------------------------------------------------------------------------

#[derive(Clone)]
pub(crate) struct McpServer {
    tool_router: ToolRouter<Self>,
    bridge: Arc<McpBridge>,
}

#[tool_router]
impl McpServer {
    pub(crate) fn new(bridge: Arc<McpBridge>) -> Self {
        Self {
            tool_router: Self::tool_router(),
            bridge,
        }
    }

    fn bridge(&self) -> &McpBridge {
        &self.bridge
    }

    // -------------------------------------------------------------------------
    // Mutations
    // -------------------------------------------------------------------------

    #[tool(
        description = "Open a new create_company form and return its form_id. Call `update_field` to fill in `company_name` (string) and `currency` (one of `USD`, `IQD`), then `submit_form`."
    )]
    async fn open_form(&self) -> Result<CallToolResult, ErrorData> {
        let old_ids = self.bridge().read(current_form_ids);
        let old_ids_for_check = old_ids.clone();

        let message: Box<dyn MessageTrait> = Box::new(HomeMessage::Intent(HomeIntent::AddForm(
            FormsToAdd::CreateCompany,
        )));
        self.bridge()
            .dispatch(message, move |m| {
                m.page_home
                    .forms
                    .iter()
                    .any(|e| !old_ids_for_check.contains(&form_id_to_u32(e.id)))
            })
            .await
            .map_err(err)?;

        let new_id = self
            .bridge()
            .read(|m| {
                m.page_home
                    .forms
                    .iter()
                    .find(|e| !old_ids.contains(&form_id_to_u32(e.id)))
                    .map(|e| form_id_to_u32(e.id))
            })
            .ok_or_else(|| err(anyhow!("no new form observed after open_form")))?;

        json(OpenFormResponse { form_id: new_id })
    }

    #[tool(
        description = "Update one field on a create_company form. `field` must be `company_name` (string) or `currency` (one of `USD`, `IQD`). Returns the updated form snapshot. Rejects if the form is loading or showing a consent dialog — call `consent_form` first."
    )]
    async fn update_field(
        &self,
        Parameters(args): Parameters<UpdateFieldArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let form_id = u32_to_form_id(args.form_id)
            .ok_or_else(|| err(anyhow!("invalid form_id {}", args.form_id)))?;

        let pre = self.bridge().read(|m| {
            m.page_home
                .forms
                .iter()
                .find(|e| e.id == form_id)
                .and_then(|a| Some(a.form.clone()))
        });

        let Some(form) = pre else {
            return Err(err(anyhow!("form {} not found", args.form_id)));
        };

        let message = match form {
            FormLocalModel::CreateBranch(local_model) => todo!(),
            FormLocalModel::CreateCompany(local_model) => {
                use_case_create_company::mcp::field_intent(
                    &local_model,
                    &args.field,
                    args.value.clone(),
                )
                .map_err(err)?
            }
            FormLocalModel::CreateAccount(local_model) => todo!(),
            FormLocalModel::CreateAccountForBranch(local_model) => todo!(),
            FormLocalModel::CreateJournalEntry(local_model) => todo!(),
        };

        let wrapped: Box<dyn MessageTrait> =
            Box::new(HomeMessage::FormRelated { form_id, message });

        let field = args.field.clone();
        let expected = args.value.clone();
        self.bridge()
            .dispatch(wrapped, move |m| {
                m.page_home
                    .forms
                    .iter()
                    .find(|e| e.id == form_id)
                    .and_then(|e| {
                        with_company_lm(&e.form, |lm| {
                            use_case_create_company::mcp::extract_fields(lm)
                                .get(&field)
                                .map(|v| v == &expected)
                                .unwrap_or(false)
                        })
                    })
                    .unwrap_or(false)
            })
            .await
            .map_err(err)?;

        let snap = self
            .bridge()
            .read(|m| {
                m.page_home
                    .forms
                    .iter()
                    .find(|e| e.id == form_id)
                    .and_then(try_snapshot)
            })
            .ok_or_else(|| err(anyhow!("form disappeared after update")))?;

        json(snap)
    }
}

#[tool_handler]
impl ServerHandler for McpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            instructions: Some(
                "This server exposes the accounting app's create_company form as \
                 tools. `form_id` from `open_form` is stable across reordering; \
                 use it — never a UI index. The typical flow: open_form → \
                 update_field(company_name) → update_field(currency) → \
                 submit_form. On submit, `success` means the form was \
                 auto-closed. `waiting_for_consent` means the 5s dialog \
                 appeared; call `consent_form`."
                    .into(),
            ),
            ..Default::default()
        }
    }
}

// -----------------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------------

fn err(e: anyhow::Error) -> ErrorData {
    ErrorData::internal_error(format!("{e:#}"), None)
}

fn json<T: Serialize>(v: T) -> Result<CallToolResult, ErrorData> {
    let s =
        to_string(&v).map_err(|e| ErrorData::internal_error(format!("json encode: {e}"), None))?;
    Ok(CallToolResult::success(vec![Content::text(s)]))
}
