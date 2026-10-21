//! Form lifecycle and global-state tools. Each helper dispatches a real intent
//! through the shared `Commander` and returns the post-mutation snapshot.

use crate::mcp::bridge::McpBridge;
use crate::mcp::snapshot;
use crate::mcp::snapshot::FormKind;
use crate::mcp::snapshot::FormSnapshot;
use crate::mcp::snapshot::GlobalState;
use crate::use_case_home::client::FormLocalModel;
use crate::use_case_home::client::FormsToAdd;
use crate::use_case_home::client::Intent as HomeIntent;
use crate::use_case_home::client::Message as HomeMessage;
use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;
use utility::ui_effect::MessageTrait;

// -----------------------------------------------------------------------------
// Arg / response structs
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FormKindArg {
    CreateAccount,
    CreateBranch,
    CreateCompany,
    CreateAccountForBranch,
    CreateJournalEntry,
}

impl From<FormKindArg> for FormsToAdd {
    fn from(v: FormKindArg) -> Self {
        match v {
            FormKindArg::CreateAccount => FormsToAdd::CreateAccount,
            FormKindArg::CreateBranch => FormsToAdd::CreateBranch,
            FormKindArg::CreateCompany => FormsToAdd::CreateCompany,
            FormKindArg::CreateAccountForBranch => FormsToAdd::CreateAccountForBranch,
            FormKindArg::CreateJournalEntry => FormsToAdd::CreateJournalEntry,
        }
    }
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub(crate) struct AddFormArgs {
    pub kind: FormKindArg,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub(crate) struct AddFormResponse {
    pub form: FormSnapshot,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub(crate) struct CopyFormArgs {
    pub form_id: u32,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub(crate) struct CopyFormResponse {
    pub form: FormSnapshot,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub(crate) struct DeleteFormArgs {
    pub form_id: u32,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub(crate) struct SubmitFormArgs {
    pub form_id: u32,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub(crate) struct OkResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<FormSnapshot>,
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
pub(crate) struct SelectCompanyArgs {
    pub company_uuid_hex: String,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub(crate) struct SelectBranchArgs {
    pub branch_uuid_hex: String,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub(crate) struct DismissErrorArgs {
    pub index: usize,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub(crate) struct ToggleErrorArgs {
    pub index: usize,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub(crate) struct GlobalStateResponse {
    pub state: GlobalState,
}

// -----------------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------------

fn current_form_ids(bridge: &McpBridge) -> HashSet<u32> {
    bridge.read(|m| {
        m.page_home
            .forms
            .iter()
            .map(|e| snapshot::form_id_to_u32(e.id))
            .collect()
    })
}

fn find_new_form_id(bridge: &McpBridge, old_ids: &HashSet<u32>) -> Option<u32> {
    bridge.read(|m| {
        m.page_home
            .forms
            .iter()
            .find(|e| !old_ids.contains(&snapshot::form_id_to_u32(e.id)))
            .map(|e| snapshot::form_id_to_u32(e.id))
    })
}

fn snapshot_of(bridge: &McpBridge, form_id: u32) -> Option<FormSnapshot> {
    bridge.read(|m| snapshot::extract_form(m, form_id))
}

pub(crate) async fn add_form(bridge: &McpBridge, args: AddFormArgs) -> Result<AddFormResponse> {
    let old_ids = current_form_ids(bridge);
    let old_ids_for_check = old_ids.clone();

    let forms_to_add: FormsToAdd = args.kind.into();
    let message: Box<dyn MessageTrait> =
        Box::new(HomeMessage::Intent(HomeIntent::AddForm(forms_to_add)));
    bridge
        .dispatch(message, move |m| {
            m.page_home
                .forms
                .iter()
                .any(|e| !old_ids_for_check.contains(&snapshot::form_id_to_u32(e.id)))
        })
        .await?;

    let new_id =
        find_new_form_id(bridge, &old_ids).ok_or_else(|| anyhow!("no new form observed"))?;
    let form = snapshot_of(bridge, new_id)
        .ok_or_else(|| anyhow!("new form disappeared before snapshot"))?;
    Ok(AddFormResponse { form })
}

pub(crate) async fn copy_form(bridge: &McpBridge, args: CopyFormArgs) -> Result<CopyFormResponse> {
    let form_id = snapshot::u32_to_form_id(args.form_id)
        .ok_or_else(|| anyhow!("invalid form_id {}", args.form_id))?;

    let is_journal = bridge.read(|m| {
        m.page_home
            .forms
            .iter()
            .find(|e| e.id == form_id)
            .map(|e| matches!(e.form, FormLocalModel::CreateJournalEntry(_)))
            .unwrap_or(false)
    });
    if is_journal {
        bail!("copy_form is not supported for journal-entry forms");
    }

    let index = bridge
        .read(|m| m.page_home.forms.iter().position(|e| e.id == form_id))
        .ok_or_else(|| anyhow!("form {} not found", args.form_id))?;

    let old_ids = current_form_ids(bridge);
    let old_ids_for_check = old_ids.clone();

    let message: Box<dyn MessageTrait> = Box::new(HomeMessage::Intent(HomeIntent::CopyForm(index)));
    bridge
        .dispatch(message, move |m| {
            m.page_home
                .forms
                .iter()
                .any(|e| !old_ids_for_check.contains(&snapshot::form_id_to_u32(e.id)))
        })
        .await?;

    let new_id =
        find_new_form_id(bridge, &old_ids).ok_or_else(|| anyhow!("no new form observed"))?;
    let form = snapshot_of(bridge, new_id)
        .ok_or_else(|| anyhow!("new form disappeared before snapshot"))?;
    Ok(CopyFormResponse { form })
}

pub(crate) async fn delete_form(bridge: &McpBridge, args: DeleteFormArgs) -> Result<OkResponse> {
    let form_id = snapshot::u32_to_form_id(args.form_id)
        .ok_or_else(|| anyhow!("invalid form_id {}", args.form_id))?;
    let index = bridge
        .read(|m| m.page_home.forms.iter().position(|e| e.id == form_id))
        .ok_or_else(|| anyhow!("form {} not found", args.form_id))?;

    let message: Box<dyn MessageTrait> =
        Box::new(HomeMessage::Intent(HomeIntent::DeleteForm(index)));
    bridge
        .dispatch(message, move |m| {
            !m.page_home.forms.iter().any(|e| e.id == form_id)
        })
        .await?;
    Ok(OkResponse {
        status: "deleted".into(),
        snapshot: None,
    })
}

pub(crate) async fn submit_form(bridge: &McpBridge, args: SubmitFormArgs) -> Result<OkResponse> {
    let form_id = snapshot::u32_to_form_id(args.form_id)
        .ok_or_else(|| anyhow!("invalid form_id {}", args.form_id))?;

    let kind = bridge
        .read(|m| {
            m.page_home
                .forms
                .iter()
                .find(|e| e.id == form_id)
                .map(|e| snapshot::extract_form_kind(&e.form))
        })
        .ok_or_else(|| anyhow!("form {} not found", args.form_id))?;

    if let Some(reason) = bridge.read(|m| {
        m.page_home
            .forms
            .iter()
            .find(|e| e.id == form_id)
            .and_then(|e| {
                if snapshot::form_is_loading(&e.form) {
                    Some("form is currently loading; call consent_form or wait".to_string())
                } else if snapshot::form_show_dialog(&e.form) {
                    Some("form is waiting on a consent dialog; call consent_form first".to_string())
                } else {
                    None
                }
            })
    }) {
        bail!("{reason}");
    }

    let sub = build_submit_intent(kind)?;
    let wrapped: Box<dyn MessageTrait> = Box::new(HomeMessage::FormRelated {
        form_id,
        message: sub,
    });
    bridge
        .dispatch(wrapped, move |m| {
            match m.page_home.forms.iter().find(|e| e.id == form_id) {
                None => true,
                Some(e) => {
                    !snapshot::form_is_loading(&e.form)
                        || snapshot::form_show_dialog(&e.form)
                        || !snapshot::form_errors(&e.form, m).is_empty()
                }
            }
        })
        .await?;

    let (status, snapshot_out) =
        bridge.read(
            |m| match m.page_home.forms.iter().find(|e| e.id == form_id) {
                None => ("success".to_string(), None),
                Some(e) => {
                    let snap = snapshot::extract_form_snapshot(e, m);
                    if !snap.errors.is_empty() {
                        ("failure".to_string(), Some(snap))
                    } else if snap.show_dialog {
                        ("waiting_for_consent".to_string(), Some(snap))
                    } else if snap.loading {
                        ("loading".to_string(), Some(snap))
                    } else {
                        ("submitted".to_string(), Some(snap))
                    }
                }
            },
        );
    Ok(OkResponse {
        status,
        snapshot: snapshot_out,
    })
}

pub(crate) async fn consent_form(bridge: &McpBridge, args: ConsentFormArgs) -> Result<OkResponse> {
    use utility::ui_orchestration::UserConsent;
    let form_id = snapshot::u32_to_form_id(args.form_id)
        .ok_or_else(|| anyhow!("invalid form_id {}", args.form_id))?;

    let kind = bridge
        .read(|m| {
            m.page_home
                .forms
                .iter()
                .find(|e| e.id == form_id)
                .map(|e| snapshot::extract_form_kind(&e.form))
        })
        .ok_or_else(|| anyhow!("form {} not found", args.form_id))?;

    let consent = match args.consent {
        ConsentArg::Wait => UserConsent::WaitForServerResponse,
        ConsentArg::DontWait => UserConsent::DontWaitForServerResponse,
        ConsentArg::Cancel => UserConsent::CancelOperation,
    };
    let sub = build_consent_intent(kind, consent)?;
    let wrapped: Box<dyn MessageTrait> = Box::new(HomeMessage::FormRelated {
        form_id,
        message: sub,
    });
    bridge
        .dispatch(wrapped, move |m| {
            match m.page_home.forms.iter().find(|e| e.id == form_id) {
                None => true,
                Some(e) => {
                    !snapshot::form_show_dialog(&e.form) || !snapshot::form_is_loading(&e.form)
                }
            }
        })
        .await?;

    let snapshot_out = snapshot_of(bridge, args.form_id);
    Ok(OkResponse {
        status: "consented".into(),
        snapshot: snapshot_out,
    })
}

pub(crate) async fn select_company(
    bridge: &McpBridge,
    args: SelectCompanyArgs,
) -> Result<GlobalStateResponse> {
    let uuid_type = snapshot::hex_to_uuid_type(&args.company_uuid_hex)?;
    let company_uuid = kernel::new_types::CompanyUuid::from(uuid_type);
    let expected = company_uuid.clone();
    let message: Box<dyn MessageTrait> =
        Box::new(use_case_select_default_company::client::Message::Intent(
            use_case_select_default_company::client::Intent::SelectCompany(company_uuid),
        ));
    bridge
        .dispatch(message, move |m| {
            m.selected_company_uuid.as_ref() == Some(&expected)
        })
        .await?;
    let state = bridge.read(snapshot::extract_global_state);
    Ok(GlobalStateResponse { state })
}

pub(crate) async fn select_branch(
    bridge: &McpBridge,
    args: SelectBranchArgs,
) -> Result<GlobalStateResponse> {
    let uuid_type = snapshot::hex_to_uuid_type(&args.branch_uuid_hex)?;
    let branch_uuid = kernel::new_types::BranchUuid::from(uuid_type);
    let expected = branch_uuid.clone();
    let message: Box<dyn MessageTrait> =
        Box::new(use_case_select_default_company::client::Message::Intent(
            use_case_select_default_company::client::Intent::SelectBranch(branch_uuid),
        ));
    bridge
        .dispatch(message, move |m| {
            m.selected_company_branch_uuid.as_ref() == Some(&expected)
        })
        .await?;
    let state = bridge.read(snapshot::extract_global_state);
    Ok(GlobalStateResponse { state })
}

pub(crate) async fn go_to_sign_in(bridge: &McpBridge) -> Result<GlobalStateResponse> {
    let message: Box<dyn MessageTrait> =
        Box::new(crate::use_case_navigator::client::Message::Intent(
            crate::use_case_navigator::client::Intent::GoToSignIn,
        ));
    bridge
        .dispatch(message, |m| {
            m.navigator == crate::use_case_navigator::client::Navigator::SignIn
        })
        .await?;
    let state = bridge.read(snapshot::extract_global_state);
    Ok(GlobalStateResponse { state })
}

pub(crate) async fn go_to_sign_up(bridge: &McpBridge) -> Result<GlobalStateResponse> {
    let message: Box<dyn MessageTrait> =
        Box::new(crate::use_case_navigator::client::Message::Intent(
            crate::use_case_navigator::client::Intent::GoToSignUp,
        ));
    bridge
        .dispatch(message, |m| {
            m.navigator == crate::use_case_navigator::client::Navigator::SignUp
        })
        .await?;
    let state = bridge.read(snapshot::extract_global_state);
    Ok(GlobalStateResponse { state })
}

pub(crate) async fn dismiss_error(
    bridge: &McpBridge,
    args: DismissErrorArgs,
) -> Result<OkResponse> {
    let len = bridge.read(|m| m.page_error_handler.errors.0.len());
    if args.index >= len {
        bail!("error index {} out of bounds (len {})", args.index, len);
    }
    let message: Box<dyn MessageTrait> = Box::new(use_case_error_handler::client::Message::Intent(
        use_case_error_handler::client::Intent::DeleteOne(args.index),
    ));
    bridge
        .dispatch(message, move |m| m.page_error_handler.errors.0.len() < len)
        .await?;
    Ok(OkResponse {
        status: "dismissed".into(),
        snapshot: None,
    })
}

pub(crate) async fn dismiss_all_errors(bridge: &McpBridge) -> Result<OkResponse> {
    let message: Box<dyn MessageTrait> = Box::new(use_case_error_handler::client::Message::Intent(
        use_case_error_handler::client::Intent::DeleteAll,
    ));
    bridge
        .dispatch(message, |m| m.page_error_handler.errors.0.is_empty())
        .await?;
    Ok(OkResponse {
        status: "dismissed_all".into(),
        snapshot: None,
    })
}

pub(crate) async fn toggle_error(bridge: &McpBridge, args: ToggleErrorArgs) -> Result<OkResponse> {
    let len = bridge.read(|m| m.page_error_handler.errors.0.len());
    if args.index >= len {
        bail!("error index {} out of bounds (len {})", args.index, len);
    }
    let before_expand = bridge.read(|m| {
        m.page_error_handler
            .errors
            .0
            .get(args.index)
            .map(|e| e.is_expand)
            .unwrap_or(false)
    });
    let message: Box<dyn MessageTrait> = Box::new(use_case_error_handler::client::Message::Intent(
        use_case_error_handler::client::Intent::ExpandOrCollapseOne(args.index),
    ));
    bridge
        .dispatch(message, move |m| {
            m.page_error_handler
                .errors
                .0
                .get(args.index)
                .map(|e| e.is_expand != before_expand)
                .unwrap_or(true)
        })
        .await?;
    Ok(OkResponse {
        status: "toggled".into(),
        snapshot: None,
    })
}

// -----------------------------------------------------------------------------
// Sub-intent builders
// -----------------------------------------------------------------------------

fn build_submit_intent(kind: FormKind) -> Result<Box<dyn MessageTrait>> {
    Ok(match kind {
        FormKind::CreateAccount => {
            use use_case_create_account::client::Intent;
            use use_case_create_account::client::Message;
            Box::new(Message::Intent(Intent::Submit))
        }
        FormKind::CreateBranch => {
            use use_case_create_branch::client::Intent;
            use use_case_create_branch::client::Message;
            Box::new(Message::Intent(Intent::Submit))
        }
        FormKind::CreateCompany => {
            use use_case_create_company::client::Intent;
            use use_case_create_company::client::Message;
            Box::new(Message::Intent(Intent::Submit))
        }
        FormKind::CreateAccountForBranch => {
            use use_case_create_account_for_branch::client::Intent;
            use use_case_create_account_for_branch::client::Message;
            Box::new(Message::Intent(Intent::Submit))
        }
        FormKind::CreateJournalEntry => {
            use use_case_create_journal_entry::client::Intent;
            use use_case_create_journal_entry::client::Message;
            Box::new(Message::Intent(Intent::Submit))
        }
    })
}

fn build_consent_intent(
    kind: FormKind,
    consent: utility::ui_orchestration::UserConsent,
) -> Result<Box<dyn MessageTrait>> {
    Ok(match kind {
        FormKind::CreateAccount => {
            use use_case_create_account::client::Intent;
            use use_case_create_account::client::Message;
            Box::new(Message::Intent(Intent::Consent(consent)))
        }
        FormKind::CreateBranch => {
            use use_case_create_branch::client::Intent;
            use use_case_create_branch::client::Message;
            Box::new(Message::Intent(Intent::Consent(consent)))
        }
        FormKind::CreateCompany => {
            use use_case_create_company::client::Intent;
            use use_case_create_company::client::Message;
            Box::new(Message::Intent(Intent::Consent(consent)))
        }
        FormKind::CreateAccountForBranch => {
            use use_case_create_account_for_branch::client::Intent;
            use use_case_create_account_for_branch::client::Message;
            Box::new(Message::Intent(Intent::Consent(consent)))
        }
        FormKind::CreateJournalEntry => {
            use use_case_create_journal_entry::client::Intent;
            use use_case_create_journal_entry::client::Message;
            Box::new(Message::Intent(Intent::Consent(consent)))
        }
    })
}
