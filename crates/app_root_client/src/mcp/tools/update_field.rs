//! `update_form_field` — the only polymorphic write tool. It resolves
//! `form_id → current form`, builds the sub-intent through the field
//! registry, wraps it in `FormRelated`, dispatches it, and returns the
//! resulting form snapshot.

use crate::mcp::bridge::McpBridge;
use crate::mcp::field_registry;
use crate::mcp::snapshot;
use crate::mcp::snapshot::FormSnapshot;
use crate::use_case_home::client::Message as HomeMessage;
use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub(crate) struct UpdateFormFieldArgs {
    pub form_id: u32,
    pub field: String,
    pub value: serde_json::Value,
}

pub(crate) async fn update_form_field(
    bridge: &McpBridge,
    args: UpdateFormFieldArgs,
) -> Result<FormSnapshot> {
    let form_id = snapshot::u32_to_form_id(args.form_id)
        .ok_or_else(|| anyhow!("invalid form_id {}", args.form_id))?;

    // Pre-check: the reducer soft-locks forms while loading and while a consent
    // dialog is shown. Sending an intent in that state would be silently
    // dropped by the reducer; better to fail loudly here so the AI can call
    // `consent_form` first.
    let pre = bridge.read(|m| {
        m.page_home.forms.iter().find(|e| e.id == form_id).map(|e| {
            (
                snapshot::form_is_loading(&e.form),
                snapshot::form_show_dialog(&e.form),
            )
        })
    });
    let (loading, dialog) = pre.ok_or_else(|| anyhow!("form {} not found", args.form_id))?;
    if loading {
        bail!("form is currently loading; call consent_form or wait before editing fields");
    }
    if dialog {
        bail!("form is waiting on a consent dialog; call consent_form first");
    }

    // Build the sub-intent using the current form variant.
    let sub_message = bridge
        .read(|m| {
            m.page_home.forms.iter().find(|e| e.id == form_id).map(|e| {
                field_registry::build_field_intent(&e.form, &args.field, args.value.clone())
            })
        })
        .ok_or_else(|| anyhow!("form {} not found", args.form_id))??;

    let wrapped = HomeMessage::FormRelated {
        form_id,
        message: sub_message,
    };

    // Postcondition: the field reflects the expected value.
    let field = args.field.clone();
    let expected = args.value.clone();
    bridge
        .dispatch(Box::new(wrapped), move |m| {
            m.page_home
                .forms
                .iter()
                .find(|e| e.id == form_id)
                .and_then(|e| snapshot::extract_form_field(&e.form, &field))
                .map(|v| v == expected)
                .unwrap_or(false)
        })
        .await?;

    bridge
        .read(|m| {
            m.page_home
                .forms
                .iter()
                .find(|e| e.id == form_id)
                .map(|e| snapshot::extract_form_snapshot(e, m))
        })
        .ok_or_else(|| anyhow!("form disappeared after update"))
}
