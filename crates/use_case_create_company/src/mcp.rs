use crate::client::Intent;
use crate::client::LocalModel;
use crate::client::Message;
use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;
use serde_json::Value;
use serde_json::from_value;
use serde_json::json;
use std::collections::BTreeMap;
use utility::ui_effect::MessageTrait;
use utility::ui_orchestration::UserConsent;
use utility_ui::domain::Dialog;

pub fn extract_fields(lm: &LocalModel) -> BTreeMap<String, Value> {
    let mut m = BTreeMap::new();
    m.insert("company_name".into(), json!(lm.company_name));
    m.insert("currency".into(), json!(lm.currency.as_str()));
    m
}

pub fn field_intent(lm: &LocalModel, field: &str, value: Value) -> Result<Box<dyn MessageTrait>> {
    if is_loading(lm) {
        return Err(anyhow!("form is loading; wait or call consent_form"));
    }

    if show_dialog(lm) {
        return Err(anyhow!(
            "form is waiting on a consent dialog; call consent_form first"
        ));
    }

    let intent = match field {
        "company_name" => Intent::CompanyName(from_value(value)?),
        "currency" => Intent::Currency(from_value(value)?),
        other => bail!("unknown field `{other}` for CreateCompany"),
    };
    Ok(Box::new(Message::Intent(intent)))
}

pub fn submit_intent() -> Box<dyn MessageTrait> {
    Box::new(Message::Intent(Intent::Submit))
}

pub fn consent_intent(consent: UserConsent) -> Box<dyn MessageTrait> {
    Box::new(Message::Intent(Intent::Consent(consent)))
}

pub fn is_loading(lm: &LocalModel) -> bool {
    lm.async_state.is_loading()
}

pub fn show_dialog(lm: &LocalModel) -> bool {
    matches!(lm.show_dialog, Dialog::Show)
}

pub fn errors(lm: &LocalModel) -> Vec<String> {
    let mut errs = Vec::new();
    if let Some(e) = lm.error_company_name() {
        errs.push(format!("company_name: {e:?}"));
    }
    errs
}
