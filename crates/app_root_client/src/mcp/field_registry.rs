//! Field-name → intent mapping. The **only** place in `mcp/` that constructs a
//! concrete use-case message from a `serde_json::Value`.
//!
//! Consequences:
//!
//! * Renaming `Intent::AccountName` in `use_case_create_account` fails this
//!   file at compile time — exactly what we want.
//! * Adding a new field to a `LocalModel` (and its matching intent) requires
//!   adding an arm here; that mirrors how `app_ui.rs::FormInstance` requires
//!   adding a component arm when a new form kind lands.
//!
//! We deliberately use the *name-based* intents (`CompanyName`, `AccountName`)
//! rather than the index-based ones (`SelectedCompany(idx)`, `SelectedAccount(idx)`).
//! The MCP tools take names/hex-ids, not UI indices; using name intents means
//! we never have to reconstruct the filtered-list index that the UI uses.

use crate::use_case_home::client::FormLocalModel;
use anyhow::Result;
use anyhow::bail;
use utility::ui_effect::MessageTrait;

pub(crate) fn build_field_intent(
    form: &FormLocalModel,
    field: &str,
    value: serde_json::Value,
) -> Result<Box<dyn MessageTrait>> {
    match form {
        FormLocalModel::CreateAccount(_) => build_create_account(field, value),
        FormLocalModel::CreateBranch(_) => build_create_branch(field, value),
        FormLocalModel::CreateCompany(_) => build_create_company(field, value),
        FormLocalModel::CreateAccountForBranch(_) => build_create_account_for_branch(field, value),
        FormLocalModel::CreateJournalEntry(_) => bail!(
            "editing journal-entry fields over MCP is not supported yet \
             (the reducer's clone_user_input / is_submit_ok are todo!())"
        ),
    }
}

fn build_create_account(field: &str, value: serde_json::Value) -> Result<Box<dyn MessageTrait>> {
    use use_case_create_account::client::Intent;
    use use_case_create_account::client::Message;
    let intent = match field {
        "account_name" => Intent::AccountName(serde_json::from_value(value)?),
        "is_debit" => Intent::IsDebit(serde_json::from_value(value)?),
        "is_permanent_account" => Intent::IsPermanentAccount(serde_json::from_value(value)?),
        "unit_of_measurement_of_quantity" => {
            Intent::UnitOfMeasurementOfQuantity(serde_json::from_value(value)?)
        }
        // The UI's `selected_company_name` field is backed by `Intent::CompanyName`.
        "company_name" | "selected_company_name" => {
            Intent::CompanyName(serde_json::from_value(value)?)
        }
        other => bail!("unknown field `{other}` for CreateAccount"),
    };
    Ok(Box::new(Message::Intent(intent)))
}

fn build_create_branch(field: &str, value: serde_json::Value) -> Result<Box<dyn MessageTrait>> {
    use use_case_create_branch::client::Intent;
    use use_case_create_branch::client::Message;
    let intent = match field {
        "company_name" => Intent::CompanyName(serde_json::from_value(value)?),
        "branch_name" => Intent::BranchName(serde_json::from_value(value)?),
        "currency" => Intent::Currency(serde_json::from_value(value)?),
        "latitude" => Intent::Latitude(serde_json::from_value(value)?),
        "longitude" => Intent::Longitude(serde_json::from_value(value)?),
        other => bail!("unknown field `{other}` for CreateBranch"),
    };
    Ok(Box::new(Message::Intent(intent)))
}

fn build_create_company(field: &str, value: serde_json::Value) -> Result<Box<dyn MessageTrait>> {
    use use_case_create_company::client::Intent;
    use use_case_create_company::client::Message;
    let intent = match field {
        "company_name" => Intent::CompanyName(serde_json::from_value(value)?),
        "currency" => Intent::Currency(serde_json::from_value(value)?),
        other => bail!("unknown field `{other}` for CreateCompany"),
    };
    Ok(Box::new(Message::Intent(intent)))
}

fn build_create_account_for_branch(
    field: &str,
    value: serde_json::Value,
) -> Result<Box<dyn MessageTrait>> {
    use use_case_create_account_for_branch::client::Intent;
    use use_case_create_account_for_branch::client::Message;
    let intent = match field {
        "account_name" => Intent::AccountName(serde_json::from_value(value)?),
        "outflow_type" => Intent::OutflowType(serde_json::from_value(value)?),
        "inflow_type" => Intent::InflowType(serde_json::from_value(value)?),
        other => bail!("unknown field `{other}` for CreateAccountForBranch"),
    };
    Ok(Box::new(Message::Intent(intent)))
}
