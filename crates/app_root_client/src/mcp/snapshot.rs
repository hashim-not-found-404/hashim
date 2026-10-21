//! Read-only DTOs and extractors. These are what tools return to the AI. We
//! deliberately do not hand back the raw `TypeGlobalModel` — its shape leaks
//! the reducer hierarchy and forces the AI to reason about enum wrappers.
//!
//! Two identifiers appear here that are `pub(crate)` in other modules:
//!
//! * `use_case_home::client::FormId(u32)` — round-tripped via serde so we
//!   don't need to open up the newtype.
//! * `kernel::new_types::CompanyUuid` / `BranchUuid` — encoded as 32-char hex
//!   so the AI can pass them back in tool calls without worrying about
//!   byte-array encoding.

use crate::model::TypeGlobalModel;
use crate::use_case_home::client::FormEntry;
use crate::use_case_home::client::FormId;
use crate::use_case_home::client::FormLocalModel;
use anyhow::Result;
use kernel::new_types::BranchUuid;
use kernel::new_types::CompanyUuid;
use kernel::new_types::UuidType;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt::Write;
use std::ops::Deref;
use utility_ui::domain::Dialog;

// -----------------------------------------------------------------------------
// DTOs
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Eq, JsonSchema, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FormKind {
    CreateAccount,
    CreateBranch,
    CreateCompany,
    CreateAccountForBranch,
    CreateJournalEntry,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub(crate) struct FormSnapshot {
    pub form_id: u32,
    pub kind: FormKind,
    pub loading: bool,
    pub show_dialog: bool,
    pub fields: BTreeMap<String, serde_json::Value>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub(crate) struct GlobalState {
    pub signed_in: bool,
    pub user_id: String,
    pub user_name: Option<String>,
    pub navigator: String,
    pub selected_company_hex: Option<String>,
    pub selected_company_name: Option<String>,
    pub selected_branch_hex: Option<String>,
    pub selected_branch_name: Option<String>,
    pub error_count: usize,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub(crate) struct ErrorSnapshot {
    pub index: usize,
    pub name: String,
    pub count: u8,
    pub time_unix_ms: u64,
    pub has_backtrace: bool,
    pub is_expand: bool,
}

// -----------------------------------------------------------------------------
// FormId <-> u32 (wire representation)
// -----------------------------------------------------------------------------

pub(crate) fn form_id_to_u32(id: FormId) -> u32 {
    serde_json::to_value(id)
        .ok()
        .and_then(|v| v.as_u64())
        .map(|v| v as u32)
        .unwrap_or(0)
}

pub(crate) fn u32_to_form_id(n: u32) -> Option<FormId> {
    serde_json::from_value(serde_json::Value::from(n)).ok()
}

// -----------------------------------------------------------------------------
// Uuid hex <-> UuidType
// -----------------------------------------------------------------------------

fn hex_bytes(bytes: &[u8; 16]) -> String {
    let mut s = String::with_capacity(32);
    for b in bytes {
        let _ = write!(&mut s, "{:02x}", b);
    }
    s
}

pub(crate) fn company_uuid_hex(uuid: &CompanyUuid) -> String {
    hex_bytes(uuid.deref().deref())
}

pub(crate) fn branch_uuid_hex(uuid: &BranchUuid) -> String {
    hex_bytes(uuid.deref().deref())
}

pub(crate) fn hex_to_uuid_type(s: &str) -> Result<UuidType> {
    let s = s.trim();
    if s.len() != 32 {
        anyhow::bail!("expected 32 hex characters, got {}", s.len());
    }
    let mut out = [0u8; 16];
    for i in 0..16 {
        let byte = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16)
            .map_err(|e| anyhow::anyhow!("invalid hex byte at {}: {e}", i * 2))?;
        out[i] = byte;
    }
    Ok(UuidType::from(out))
}

// -----------------------------------------------------------------------------
// Extractors
// -----------------------------------------------------------------------------

pub(crate) fn extract_global_state(m: &TypeGlobalModel) -> GlobalState {
    let signed_in = m.user_uuid().is_some();
    let navigator = match m.navigator {
        crate::use_case_navigator::client::Navigator::SignIn => "SignIn",
        crate::use_case_navigator::client::Navigator::SignUp => "SignUp",
    }
    .to_string();

    let selected_company_hex = m
        .selected_company_uuid
        .as_ref()
        .map(|u| company_uuid_hex(u));
    let selected_company_name = m.page_select_default_company.selected_company_name(m);
    let selected_branch_hex = m
        .selected_company_branch_uuid
        .as_ref()
        .map(|u| branch_uuid_hex(u));
    let selected_branch_name = m
        .page_select_default_company
        .selected_company_branch_name(m);

    GlobalState {
        signed_in,
        user_id: m.user_id.clone(),
        user_name: m.user_name.clone(),
        navigator,
        selected_company_hex,
        selected_company_name,
        selected_branch_hex,
        selected_branch_name,
        error_count: m.page_error_handler.errors.0.len(),
    }
}

pub(crate) fn extract_forms(m: &TypeGlobalModel) -> Vec<FormSnapshot> {
    m.page_home
        .forms
        .iter()
        .map(|e| extract_form_snapshot(e, m))
        .collect()
}

pub(crate) fn extract_form(m: &TypeGlobalModel, form_id: u32) -> Option<FormSnapshot> {
    m.page_home
        .forms
        .iter()
        .find(|e| form_id_to_u32(e.id) == form_id)
        .map(|e| extract_form_snapshot(e, m))
}

pub(crate) fn extract_form_snapshot(entry: &FormEntry, m: &TypeGlobalModel) -> FormSnapshot {
    FormSnapshot {
        form_id: form_id_to_u32(entry.id),
        kind: extract_form_kind(&entry.form),
        loading: form_is_loading(&entry.form),
        show_dialog: form_show_dialog(&entry.form),
        fields: extract_form_fields(&entry.form),
        errors: form_errors(&entry.form, m),
    }
}

pub(crate) fn extract_form_kind(form: &FormLocalModel) -> FormKind {
    match form {
        FormLocalModel::CreateAccount(_) => FormKind::CreateAccount,
        FormLocalModel::CreateBranch(_) => FormKind::CreateBranch,
        FormLocalModel::CreateCompany(_) => FormKind::CreateCompany,
        FormLocalModel::CreateAccountForBranch(_) => FormKind::CreateAccountForBranch,
        FormLocalModel::CreateJournalEntry(_) => FormKind::CreateJournalEntry,
    }
}

pub(crate) fn extract_errors(m: &TypeGlobalModel) -> Vec<ErrorSnapshot> {
    m.page_error_handler
        .errors
        .0
        .iter()
        .enumerate()
        .map(|(i, e)| ErrorSnapshot {
            index: i,
            name: e.name.clone(),
            count: e.number_of_errors,
            time_unix_ms: e.time_unix_ms,
            has_backtrace: e.back_trace.is_some(),
            is_expand: e.is_expand,
        })
        .collect()
}

pub(crate) fn extract_form_fields(form: &FormLocalModel) -> BTreeMap<String, serde_json::Value> {
    use serde_json::json;
    let mut m = BTreeMap::new();
    match form {
        FormLocalModel::CreateAccount(lm) => {
            m.insert("account_name".into(), json!(lm.account_name));
            m.insert("is_debit".into(), json!(lm.is_debit));
            m.insert(
                "is_permanent_account".into(),
                json!(lm.is_permanent_account),
            );
            m.insert(
                "unit_of_measurement_of_quantity".into(),
                json!(lm.unit_of_measurement_of_quantity),
            );
            m.insert(
                "selected_company_name".into(),
                json!(lm.selected_company_name),
            );
        }
        FormLocalModel::CreateBranch(lm) => {
            m.insert("company_name".into(), json!(lm.company_name));
            m.insert("branch_name".into(), json!(lm.branch_name));
            m.insert("currency".into(), json!(lm.currency.as_str()));
            m.insert("latitude".into(), json!(lm.location.latitude));
            m.insert("longitude".into(), json!(lm.location.longitude));
        }
        FormLocalModel::CreateCompany(lm) => {
            m.insert("company_name".into(), json!(lm.company_name));
            m.insert("currency".into(), json!(lm.currency.as_str()));
        }
        FormLocalModel::CreateAccountForBranch(lm) => {
            m.insert("account_name".into(), json!(lm.account_name));
            m.insert("outflow_type".into(), json!(lm.outflow_type.as_str()));
            m.insert("inflow_type".into(), json!(lm.inflow_type.as_str()));
        }
        FormLocalModel::CreateJournalEntry(lm) => {
            m.insert("shared_entry_id".into(), json!(lm.shared_entry_id));
            m.insert("is_loading".into(), json!(lm.is_loading));
            m.insert(
                "some_account_are_not_inferred".into(),
                json!(lm.some_account_are_not_inferred),
            );
            m.insert(
                "error_container_is_empty".into(),
                json!(lm.error_container_is_empty),
            );
            m.insert(
                "not_all_entry_inferred".into(),
                json!(lm.not_all_entry_inferred),
            );
            m.insert("double_entries".into(), json!(lm.double_entries));
        }
    }
    m
}

pub(crate) fn extract_form_field(form: &FormLocalModel, field: &str) -> Option<serde_json::Value> {
    extract_form_fields(form).remove(field)
}

pub(crate) fn form_is_loading(form: &FormLocalModel) -> bool {
    match form {
        FormLocalModel::CreateAccount(lm) => lm.async_state.is_loading(),
        FormLocalModel::CreateBranch(lm) => lm.async_state.is_loading(),
        FormLocalModel::CreateCompany(lm) => lm.async_state.is_loading(),
        FormLocalModel::CreateAccountForBranch(lm) => lm.async_state.is_loading(),
        FormLocalModel::CreateJournalEntry(lm) => lm.is_loading,
    }
}

pub(crate) fn form_show_dialog(form: &FormLocalModel) -> bool {
    let is_show = |d: &Dialog| matches!(d, Dialog::Show);
    match form {
        FormLocalModel::CreateAccount(lm) => is_show(&lm.show_dialog),
        FormLocalModel::CreateBranch(lm) => is_show(&lm.show_dialog),
        FormLocalModel::CreateCompany(lm) => is_show(&lm.show_dialog),
        FormLocalModel::CreateAccountForBranch(lm) => is_show(&lm.show_dialog),
        FormLocalModel::CreateJournalEntry(lm) => is_show(&lm.show_dialog),
    }
}

pub(crate) fn form_errors(form: &FormLocalModel, m: &TypeGlobalModel) -> Vec<String> {
    let mut errs = Vec::new();
    match form {
        FormLocalModel::CreateAccount(lm) => {
            if let Some(e) = lm.error_account_name() {
                errs.push(format!("account_name: {e:?}"));
            }
            if let Some(e) = lm.error_company_name(m) {
                errs.push(format!("company: {e:?}"));
            }
        }
        FormLocalModel::CreateBranch(lm) => {
            if let Some(e) = lm.error_branch_name() {
                errs.push(format!("branch_name: {e:?}"));
            }
            if let Some(e) = lm.error_location() {
                errs.push(format!("location: {e:?}"));
            }
            if let Some(e) = lm.company_name_error(m) {
                errs.push(format!("company: {e:?}"));
            }
        }
        FormLocalModel::CreateCompany(lm) => {
            if let Some(e) = lm.error_company_name() {
                errs.push(format!("company_name: {e:?}"));
            }
        }
        FormLocalModel::CreateAccountForBranch(_) => {}
        FormLocalModel::CreateJournalEntry(lm) => {
            if lm.some_account_are_not_inferred {
                errs.push("some accounts are not inferred".into());
            }
            if lm.error_container_is_empty {
                errs.push("container is empty".into());
            }
            if lm.not_all_entry_inferred {
                errs.push("not all entries inferred".into());
            }
        }
    }
    errs
}
