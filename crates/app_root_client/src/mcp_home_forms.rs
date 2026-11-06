use crate::model::TypeGlobalModel;
use crate::wire;
use anyhow::Result;
use schemars::JsonSchema;
use schemars::schema_for;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use serde_json::to_value;
use std::future::Future;
use std::pin::Pin;
use use_case_home_wire_client::FormLocalModel;
use use_case_home_wire_client::Intent;
use use_case_home_wire_client::Message;
use utility::cache::CacheStruct;
use utility::mcp::McpToolDef;
use utility::mcp::TraitMcpProvider;
use utility::mcp::error_to_mcp;
use utility::mcp::parse_args;
use utility::ui_effect::Commander;

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "snake_case")]
enum FormType {
    CreateBranch,
    CreateCompany,
    CreateJournal,
}

const ADD_FORM: &str = "add_form";

#[derive(Debug, JsonSchema, Deserialize)]
#[serde(deny_unknown_fields)]
struct AddFormArgs {
    /// Form type name. Call `list_form_types` to enumerate valid values.
    form_type: FormType,
}

#[derive(Debug, Serialize)]
struct Output {}

pub(crate) struct TypeMcpProvider;

impl TraitMcpProvider<TypeGlobalModel> for TypeMcpProvider {
    fn tools(&self) -> Vec<McpToolDef> {
        vec![McpToolDef {
            name: ADD_FORM,
            description: "Open a new form . Then use the matching `create_*` tool to fill it.",
            input_schema: schema_for!(AddFormArgs),
        }]
    }

    fn call<'a>(
        &'a self,
        tool_name: &str,
        args: Value,
        _model: &'a TypeGlobalModel,
        commander: &'a Commander,
        cache: &'a CacheStruct,
    ) -> Pin<Box<dyn Future<Output = Result<Value>> + Send + 'a>> {
        let tool_name = tool_name.to_owned();
        let _ = cache;
        Box::pin(async move {
            match tool_name.as_str() {
                ADD_FORM => {
                    let AddFormArgs { form_type } = parse_args(args)?;
                    let form = map_form_type_to_local_model(form_type);

                    commander.sender_with_replier(Message::Intent(Intent::AddForm(form)));

                    Ok(to_value(Output {})?)
                }
                _ => Err(error_to_mcp(
                    "unknown_tool",
                    format!("unknown tool: {tool_name}"),
                )),
            }
        })
    }
}

fn map_form_type_to_local_model(form_type: FormType) -> Box<dyn FormLocalModel> {
    match form_type {
        FormType::CreateBranch => Box::new(wire::BranchForm::default()),
        FormType::CreateCompany => Box::new(wire::CompanyForm::default()),
        FormType::CreateJournal => Box::new(wire::JournalForm::default()),
    }
}
