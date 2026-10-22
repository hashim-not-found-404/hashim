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
use serde::Deserialize;
use serde::Serialize;
use serde_json::to_string;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::sync::Arc;
use use_case_create_company::client::LocalModel as CompanyLocalModel;
use utility::ui_effect::MessageTrait;
use utility::ui_orchestration::GenricAsyncState;
use utility::ui_orchestration::UserConsent;

// -----------------------------------------------------------------------------
// DTOs
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub(crate) struct FormSnapshot {
    pub form_id: u32,
    pub loading: bool,
    pub show_dialog: bool,
    pub fields: BTreeMap<String, serde_json::Value>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub(crate) struct GlobalStateView {
    pub signed_in: bool,
    pub user_id: String,
    pub user_name: Option<String>,
    pub error_count: usize,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub(crate) struct ListFormsResponse {
    pub forms: Vec<FormSnapshot>,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub(crate) struct GetFormArgs {
    pub form_id: u32,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub(crate) struct GetFormResponse {
    pub form: Option<FormSnapshot>,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub(crate) struct OpenFormResponse {
    pub form_id: u32,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub(crate) struct UpdateFieldArgs {
    pub form_id: u32,
    pub field: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub(crate) struct SubmitFormArgs {
    pub form_id: u32,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub(crate) struct DeleteFormArgs {
    pub form_id: u32,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ConsentArg {
    Wait,
    DontWait,
    Cancel,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub(crate) struct ConsentFormArgs {
    pub form_id: u32,
    pub consent: ConsentArg,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub(crate) struct ActionResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<FormSnapshot>,
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

fn has_async_failure(lm: &CompanyLocalModel) -> bool {
    matches!(lm.async_state, GenricAsyncState::Failure { .. })
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
    // Reads
    // -------------------------------------------------------------------------

    #[tool(description = "Get current user state: signed_in, user_id, user_name, error_count.")]
    async fn get_global_state(&self) -> Result<CallToolResult, ErrorData> {
        let view = self.bridge().read(|m| GlobalStateView {
            signed_in: m.user_uuid().is_some(),
            user_id: m.user_id.clone(),
            user_name: m.user_name.clone(),
            error_count: m.page_error_handler.errors.0.len(),
        });
        json(view)
    }

    #[tool(
        description = "List all currently-open create_company forms with their field values, loading state, dialog state, and errors. Forms of other kinds are not shown."
    )]
    async fn list_forms(&self) -> Result<CallToolResult, ErrorData> {
        let forms: Vec<FormSnapshot> = self
            .bridge()
            .read(|m| m.page_home.forms.iter().filter_map(try_snapshot).collect());
        json(ListFormsResponse { forms })
    }

    #[tool(
        description = "Get one create_company form by form_id. Returns `{ form: null }` if the form does not exist or is not a create_company form. After a successful submit, the form is auto-closed, so `null` is the success signal."
    )]
    async fn get_form(
        &self,
        Parameters(args): Parameters<GetFormArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let form = self.bridge().read(|m| {
            m.page_home
                .forms
                .iter()
                .find(|e| form_id_to_u32(e.id) == args.form_id)
                .and_then(try_snapshot)
        });
        json(GetFormResponse { form })
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

    #[tool(
        description = "Submit a create_company form. Returns status = one of: `success` (form auto-closed on success; snapshot is null), `failure` (server or validation rejected; see snapshot.errors), `waiting_for_consent` (5s dialog appeared; call consent_form), `loading` (still in flight; call get_form later). Rejects if the form is already loading or showing a consent dialog."
    )]
    async fn submit_form(
        &self,
        Parameters(args): Parameters<SubmitFormArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let form_id = u32_to_form_id(args.form_id)
            .ok_or_else(|| err(anyhow!("invalid form_id {}", args.form_id)))?;

        let pre = self
            .bridge()
            .read(|m| {
                m.page_home
                    .forms
                    .iter()
                    .find(|e| e.id == form_id)
                    .and_then(|e| {
                        with_company_lm(&e.form, |lm| {
                            (
                                use_case_create_company::mcp::is_loading(lm),
                                use_case_create_company::mcp::show_dialog(lm),
                            )
                        })
                    })
            })
            .ok_or_else(|| {
                err(anyhow!(
                    "form {} not found or is not a create_company form",
                    args.form_id
                ))
            })?;
        if pre.0 {
            return Err(err(anyhow!("form is loading; wait or call consent_form")));
        }
        if pre.1 {
            return Err(err(anyhow!(
                "form is waiting on a consent dialog; call consent_form first"
            )));
        }

        let sub = use_case_create_company::mcp::submit_intent();
        let wrapped: Box<dyn MessageTrait> = Box::new(HomeMessage::FormRelated {
            form_id,
            message: sub,
        });

        self.bridge()
            .dispatch(wrapped, move |m| {
                match m.page_home.forms.iter().find(|e| e.id == form_id) {
                    None => true, // form auto-closed → success
                    Some(e) => with_company_lm(&e.form, |lm| {
                        use_case_create_company::mcp::show_dialog(lm) || has_async_failure(lm)
                    })
                    .unwrap_or(true),
                }
            })
            .await
            .map_err(err)?;

        let (status, snapshot_out) =
            self.bridge().read(
                |m| match m.page_home.forms.iter().find(|e| e.id == form_id) {
                    None => ("success".to_string(), None),
                    Some(e) => match try_snapshot(e) {
                        None => ("success".to_string(), None),
                        Some(snap) => {
                            if snap.show_dialog {
                                ("waiting_for_consent".to_string(), Some(snap))
                            } else if !snap.errors.is_empty() {
                                ("failure".to_string(), Some(snap))
                            } else if snap.loading {
                                ("loading".to_string(), Some(snap))
                            } else {
                                ("submitted".to_string(), Some(snap))
                            }
                        }
                    },
                },
            );

        json(ActionResponse {
            status,
            snapshot: snapshot_out,
        })
    }

    #[tool(
        description = "Answer the 5s consent dialog on a create_company form. `consent` is one of: `wait` (block until server responds), `dont_wait` (apply to local cache only and continue offline), `cancel` (abort the pending submit). Call this whenever `get_form` reports `show_dialog: true`."
    )]
    async fn consent_form(
        &self,
        Parameters(args): Parameters<ConsentFormArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let form_id = u32_to_form_id(args.form_id)
            .ok_or_else(|| err(anyhow!("invalid form_id {}", args.form_id)))?;

        let consent = match args.consent {
            ConsentArg::Wait => UserConsent::WaitForServerResponse,
            ConsentArg::DontWait => UserConsent::DontWaitForServerResponse,
            ConsentArg::Cancel => UserConsent::CancelOperation,
        };

        let sub = use_case_create_company::mcp::consent_intent(consent);
        let wrapped: Box<dyn MessageTrait> = Box::new(HomeMessage::FormRelated {
            form_id,
            message: sub,
        });

        self.bridge()
            .dispatch(wrapped, move |m| {
                match m.page_home.forms.iter().find(|e| e.id == form_id) {
                    None => true,
                    Some(e) => with_company_lm(&e.form, |lm| {
                        !use_case_create_company::mcp::show_dialog(lm)
                    })
                    .unwrap_or(true),
                }
            })
            .await
            .map_err(err)?;

        let snapshot_out = self.bridge().read(|m| {
            m.page_home
                .forms
                .iter()
                .find(|e| e.id == form_id)
                .and_then(try_snapshot)
        });

        json(ActionResponse {
            status: "consented".into(),
            snapshot: snapshot_out,
        })
    }

    #[tool(description = "Close a form by form_id without submitting. Works for any form kind.")]
    async fn delete_form(
        &self,
        Parameters(args): Parameters<DeleteFormArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let form_id = u32_to_form_id(args.form_id)
            .ok_or_else(|| err(anyhow!("invalid form_id {}", args.form_id)))?;
        let index = self
            .bridge()
            .read(|m| m.page_home.forms.iter().position(|e| e.id == form_id))
            .ok_or_else(|| err(anyhow!("form {} not found", args.form_id)))?;

        let message: Box<dyn MessageTrait> =
            Box::new(HomeMessage::Intent(HomeIntent::DeleteForm(index)));
        self.bridge()
            .dispatch(message, move |m| {
                !m.page_home.forms.iter().any(|e| e.id == form_id)
            })
            .await
            .map_err(err)?;

        json(ActionResponse {
            status: "deleted".into(),
            snapshot: None,
        })
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
