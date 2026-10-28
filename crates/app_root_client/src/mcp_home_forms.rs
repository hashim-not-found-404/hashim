use crate::utils::COMMANDER;
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
use use_case_home::client::FormLocalModel;
use use_case_home::client::Intent;
use use_case_home::client::Message;
use utility::mcp::McpToolDef;
use utility::mcp::TraitMcpProvider;
use utility::mcp::error_to_mcp;
use utility::mcp::parse_args;

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

impl TraitMcpProvider for TypeMcpProvider {
    fn tools(&self) -> Vec<McpToolDef> {
        vec![McpToolDef {
            name: ADD_FORM,
            description: "Open a new form . Then use the matching `create_*` tool to fill it.",
            input_schema: schema_for!(AddFormArgs),
        }]
    }

    fn call(
        &self,
        tool_name: &str,
        args: Value,
    ) -> Pin<Box<dyn Future<Output = Result<Value>> + Send>> {
        let tool_name = tool_name.to_owned();
        Box::pin(async move {
            match tool_name.as_str() {
                ADD_FORM => {
                    let AddFormArgs { form_type } = parse_args(args)?;
                    let form = map_form_type_to_local_model(form_type);

                    COMMANDER.sender_with_replier(Message::Intent(Intent::AddForm(form)));

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
        FormType::CreateBranch => Box::new(wire::create_branch_form::WrapperForm::default()),
        FormType::CreateCompany => Box::new(wire::create_company_form::WrapperForm::default()),
        FormType::CreateJournal => Box::new(wire::create_journal_form::WrapperForm::default()),
    }
}
