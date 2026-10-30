use crate::client::GlobalModel;
use crate::client::Intent;
use crate::client::LocalModel;
use crate::client::Message;
use anyhow::Result;
use kernel::types::Currency;
use schemars::JsonSchema;
use schemars::schema_for;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use serde_json::to_value;
use use_case_home::mcp::form_sender;
use utility::mcp::CallInput;
use utility::mcp::McpToolDef;
use utility::mcp::error_to_mcp;
use utility::mcp::parse_args;

const CREATE_COMPANY: &str = "create_company";

#[derive(Debug, JsonSchema, Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateCompanyArgs {
    /// form id of the form to write.
    form_id: u32,
    /// Company name. Must be non-empty.
    company_name: String,
    /// Currency. One of: "USD", "IQD".
    currency: Currency,
}

#[derive(Debug, Serialize)]
struct Output {}

pub fn tools() -> Vec<McpToolDef> {
    vec![McpToolDef {
        name: CREATE_COMPANY,
        description: "Create a new company with the given name and currency. \
                      The company form is pre-filled; \
                      the companies list will refresh once the server responds.",
        input_schema: schema_for!(CreateCompanyArgs),
    }]
}

pub fn call<'a>(input: CallInput<'a, LocalModel, impl GlobalModel>) -> Result<Value> {
    match input.tool_name {
        CREATE_COMPANY => {
            let args: CreateCompanyArgs = parse_args(input.args)?;

            if args.company_name.trim().is_empty() {
                return Err(error_to_mcp(
                    "invalid_args",
                    "company_name must not be empty",
                ));
            }

            form_sender(
                input.commander.clone(),
                args.form_id.into(),
                Message::Intent(Intent::CompanyName(args.company_name)),
            );

            form_sender(
                input.commander.clone(),
                args.form_id.into(),
                Message::Intent(Intent::Currency(args.currency)),
            );

            Ok(to_value(Output {})?)
        }
        _ => Err(error_to_mcp(
            "unknown_tool",
            format!("unknown tool: {}", input.tool_name),
        )),
    }
}
