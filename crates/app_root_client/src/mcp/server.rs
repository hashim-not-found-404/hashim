//! The `rmcp` `ServerHandler`. All `#[tool]` methods live in this one
//! `#[tool_router] impl` block (rmcp only allows one per struct) and delegate
//! to helpers in `tools/`.

use crate::mcp::bridge::McpBridge;
use crate::mcp::tools;
use rmcp::ErrorData as McpError;
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
use std::sync::Arc;

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
    // Discovery (read-only)
    // -------------------------------------------------------------------------

    #[tool(
        description = "List all currently-open forms, including their current field values, loading state, consent-dialog state, and per-field errors. Call this first; the AI client is stateless between tool calls."
    )]
    async fn list_forms(&self) -> Result<CallToolResult, McpError> {
        let r = tools::discovery::list_forms(self.bridge()).map_err(err)?;
        json(r)
    }

    #[tool(
        description = "Get a single form by its form_id. Returns `{ form: null }` if the form no longer exists — which, after a submit, means it succeeded (the UI auto-closes forms on success)."
    )]
    async fn get_form(
        &self,
        Parameters(args): Parameters<tools::discovery::GetFormArgs>,
    ) -> Result<CallToolResult, McpError> {
        let r = tools::discovery::get_form(self.bridge(), args).map_err(err)?;
        json(r)
    }

    #[tool(
        description = "Get the global UI state: signed_in, user_id/user_name, navigator (SignIn/SignUp), selected company & branch, and error count."
    )]
    async fn get_global_state(&self) -> Result<CallToolResult, McpError> {
        let r = tools::discovery::get_global_state(self.bridge()).map_err(err)?;
        json(r)
    }

    #[tool(
        description = "List companies visible to the current user as `{ uuid_hex, name }` pairs. Use `select_company` to switch."
    )]
    async fn list_companies(&self) -> Result<CallToolResult, McpError> {
        let r = tools::discovery::list_companies(self.bridge()).map_err(err)?;
        json(r)
    }

    #[tool(
        description = "List branches of the currently selected company as `{ uuid_hex, name }` pairs. Use `select_branch` to switch."
    )]
    async fn list_branches(&self) -> Result<CallToolResult, McpError> {
        let r = tools::discovery::list_branches(self.bridge()).map_err(err)?;
        json(r)
    }

    #[tool(
        description = "List the current UI error list (rows shown in the error panel). Each entry has `index` (use with `dismiss_error`/`toggle_error`), `name`, `count`, `time_unix_ms`, `has_backtrace`, `is_expand`."
    )]
    async fn list_errors(&self) -> Result<CallToolResult, McpError> {
        let r = tools::discovery::list_errors(self.bridge()).map_err(err)?;
        json(r)
    }

    // -------------------------------------------------------------------------
    // Form lifecycle
    // -------------------------------------------------------------------------

    #[tool(
        description = "Open a new form. `kind` is one of: create_account, create_branch, create_company, create_account_for_branch, create_journal_entry. Returns the new form's snapshot (including its form_id) so the AI can immediately address it."
    )]
    async fn add_form(
        &self,
        Parameters(args): Parameters<tools::lifecycle::AddFormArgs>,
    ) -> Result<CallToolResult, McpError> {
        let r = tools::lifecycle::add_form(self.bridge(), args)
            .await
            .map_err(err)?;
        json(r)
    }

    #[tool(
        description = "Copy an existing form. Only supported for create_* forms with editable fields; journal-entry forms are rejected. Returns the new form's snapshot."
    )]
    async fn copy_form(
        &self,
        Parameters(args): Parameters<tools::lifecycle::CopyFormArgs>,
    ) -> Result<CallToolResult, McpError> {
        let r = tools::lifecycle::copy_form(self.bridge(), args)
            .await
            .map_err(err)?;
        json(r)
    }

    #[tool(
        description = "Close a form by form_id without submitting. Journal-entry forms are the exception — they can be deleted like any other."
    )]
    async fn delete_form(
        &self,
        Parameters(args): Parameters<tools::lifecycle::DeleteFormArgs>,
    ) -> Result<CallToolResult, McpError> {
        let r = tools::lifecycle::delete_form(self.bridge(), args)
            .await
            .map_err(err)?;
        json(r)
    }

    #[tool(
        description = "Submit a form. Returns `status` = one of: `success` (form was auto-closed on success — the snapshot is null), `failure` (form is still open with errors), `waiting_for_consent` (form is showing the 5s consent dialog; call `consent_form`), `loading` (still in flight). You generally do NOT need to submit twice."
    )]
    async fn submit_form(
        &self,
        Parameters(args): Parameters<tools::lifecycle::SubmitFormArgs>,
    ) -> Result<CallToolResult, McpError> {
        let r = tools::lifecycle::submit_form(self.bridge(), args)
            .await
            .map_err(err)?;
        json(r)
    }

    #[tool(
        description = "Answer the offline-consent dialog on a form. `consent` is one of: `wait` (block until the server responds), `dont_wait` (apply to local cache only and continue offline), `cancel` (abort the pending submit). Call this whenever `get_form` reports `show_dialog: true`."
    )]
    async fn consent_form(
        &self,
        Parameters(args): Parameters<tools::lifecycle::ConsentFormArgs>,
    ) -> Result<CallToolResult, McpError> {
        let r = tools::lifecycle::consent_form(self.bridge(), args)
            .await
            .map_err(err)?;
        json(r)
    }

    // -------------------------------------------------------------------------
    // Global selection
    // -------------------------------------------------------------------------

    #[tool(
        description = "Select the default company by its 32-char hex uuid (see `list_companies`). Clears the selected branch as a side effect."
    )]
    async fn select_company(
        &self,
        Parameters(args): Parameters<tools::lifecycle::SelectCompanyArgs>,
    ) -> Result<CallToolResult, McpError> {
        let r = tools::lifecycle::select_company(self.bridge(), args)
            .await
            .map_err(err)?;
        json(r)
    }

    #[tool(
        description = "Select the default branch by its 32-char hex uuid (see `list_branches`)."
    )]
    async fn select_branch(
        &self,
        Parameters(args): Parameters<tools::lifecycle::SelectBranchArgs>,
    ) -> Result<CallToolResult, McpError> {
        let r = tools::lifecycle::select_branch(self.bridge(), args)
            .await
            .map_err(err)?;
        json(r)
    }

    #[tool(description = "Navigate the UI to the sign-in page.")]
    async fn go_to_sign_in(&self) -> Result<CallToolResult, McpError> {
        let r = tools::lifecycle::go_to_sign_in(self.bridge())
            .await
            .map_err(err)?;
        json(r)
    }

    #[tool(description = "Navigate the UI to the sign-up page.")]
    async fn go_to_sign_up(&self) -> Result<CallToolResult, McpError> {
        let r = tools::lifecycle::go_to_sign_up(self.bridge())
            .await
            .map_err(err)?;
        json(r)
    }

    // -------------------------------------------------------------------------
    // Field update
    // -------------------------------------------------------------------------

    #[tool(
        description = "Update a single field on a form. `field` must be one of the string keys shown in the form's `fields` object (e.g. `account_name`, `is_debit`, `currency`). `value` is the JSON representation of the new value. Returns the updated form snapshot. If the form is loading or showing a consent dialog, the tool returns an error — call `consent_form` first."
    )]
    async fn update_form_field(
        &self,
        Parameters(args): Parameters<tools::update_field::UpdateFormFieldArgs>,
    ) -> Result<CallToolResult, McpError> {
        let r = tools::update_field::update_form_field(self.bridge(), args)
            .await
            .map_err(err)?;
        json(r)
    }

    // -------------------------------------------------------------------------
    // Error list management
    // -------------------------------------------------------------------------

    #[tool(
        description = "Remove the error at the given `index` (see `list_errors`). Fails if the index is out of bounds."
    )]
    async fn dismiss_error(
        &self,
        Parameters(args): Parameters<tools::lifecycle::DismissErrorArgs>,
    ) -> Result<CallToolResult, McpError> {
        let r = tools::lifecycle::dismiss_error(self.bridge(), args)
            .await
            .map_err(err)?;
        json(r)
    }

    #[tool(description = "Remove every error from the error panel.")]
    async fn dismiss_all_errors(&self) -> Result<CallToolResult, McpError> {
        let r = tools::lifecycle::dismiss_all_errors(self.bridge())
            .await
            .map_err(err)?;
        json(r)
    }

    #[tool(
        description = "Expand or collapse the error at `index`, toggling its backtrace visibility."
    )]
    async fn toggle_error(
        &self,
        Parameters(args): Parameters<tools::lifecycle::ToggleErrorArgs>,
    ) -> Result<CallToolResult, McpError> {
        let r = tools::lifecycle::toggle_error(self.bridge(), args)
            .await
            .map_err(err)?;
        json(r)
    }
}

#[tool_handler]
impl ServerHandler for McpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            instructions: Some(
                "This server exposes the accounting app's UI as tools. Forms and \
                 their field values are the primary state. The `FormId` returned by \
                 `add_form` is stable across reordering; use it — never a UI index. \
                 On `submit_form`, a `success` status means the form was auto-closed; \
                 a `waiting_for_consent` status means the app is offline or the \
                 server didn't respond within 5 s, and you should call `consent_form`."
                    .into(),
            ),
            ..Default::default()
        }
    }
}

// -----------------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------------

fn err(e: anyhow::Error) -> McpError {
    McpError::internal_error(format!("{e:#}"), None)
}

fn json<T: serde::Serialize>(v: T) -> Result<CallToolResult, McpError> {
    let s = serde_json::to_string(&v)
        .map_err(|e| McpError::internal_error(format!("json encode: {e}"), None))?;
    Ok(CallToolResult::success(vec![Content::text(s)]))
}
