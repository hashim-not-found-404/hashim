//! Read-only tools. No dispatch, no polling; each one takes a snapshot of the
//! model through `McpBridge::read`. Safe to call while the UI is mid-submit.

use crate::mcp::bridge::McpBridge;
use crate::mcp::snapshot;
use crate::mcp::snapshot::ErrorSnapshot;
use crate::mcp::snapshot::FormSnapshot;
use crate::mcp::snapshot::GlobalState;
use anyhow::Result;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

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
pub(crate) struct UuidName {
    pub uuid_hex: String,
    pub name: String,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub(crate) struct ListCompaniesResponse {
    pub companies: Vec<UuidName>,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub(crate) struct ListBranchesResponse {
    pub branches: Vec<UuidName>,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub(crate) struct ListErrorsResponse {
    pub errors: Vec<ErrorSnapshot>,
}

pub(crate) fn list_forms(bridge: &McpBridge) -> Result<ListFormsResponse> {
    let forms = bridge.read(snapshot::extract_forms);
    Ok(ListFormsResponse { forms })
}

pub(crate) fn get_form(bridge: &McpBridge, args: GetFormArgs) -> Result<GetFormResponse> {
    let form = bridge.read(|m| snapshot::extract_form(m, args.form_id));
    Ok(GetFormResponse { form })
}

pub(crate) fn get_global_state(bridge: &McpBridge) -> Result<GlobalState> {
    Ok(bridge.read(snapshot::extract_global_state))
}

pub(crate) fn list_companies(bridge: &McpBridge) -> Result<ListCompaniesResponse> {
    let companies = bridge.read(|m| {
        m.page_select_default_company
            .list_of_companies()
            .into_iter()
            .map(|(uuid, name)| UuidName {
                uuid_hex: snapshot::company_uuid_hex(&uuid),
                name,
            })
            .collect()
    });
    Ok(ListCompaniesResponse { companies })
}

pub(crate) fn list_branches(bridge: &McpBridge) -> Result<ListBranchesResponse> {
    let branches = bridge.read(|m| {
        m.page_select_default_company
            .list_of_branches(m)
            .into_iter()
            .map(|(uuid, name)| UuidName {
                uuid_hex: snapshot::branch_uuid_hex(&uuid),
                name,
            })
            .collect()
    });
    Ok(ListBranchesResponse { branches })
}

pub(crate) fn list_errors(bridge: &McpBridge) -> Result<ListErrorsResponse> {
    let errors = bridge.read(snapshot::extract_errors);
    Ok(ListErrorsResponse { errors })
}
