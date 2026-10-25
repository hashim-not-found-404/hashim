use crate::client::GlobalModel;
use crate::client::Intent;
use crate::client::LocalModel;
use crate::client::Message;
use anyhow::Result;
use kernel::mcp::CallInput;
use kernel::mcp::McpToolDef;
use kernel::mcp::error_to_mcp;
use kernel::mcp::parse_args;
use kernel::types::Currency;
use schemars::JsonSchema;
use schemars::schema_for;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use serde_json::to_value;
use std::sync::Arc;
use use_case_home::client::FormId;
use utility::ui_effect::Commander;
use utility::ui_effect::Replier;
use utility::ui_effect::new_replier;

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
struct Output {
    /// `true` once the intent was accepted onto the commander queue.
    /// Does NOT mean the reducer has already applied the change — call
    /// `list_forms` again to confirm.
    is_ok: bool,
}

pub fn tools() -> Vec<McpToolDef> {
    vec![McpToolDef {
        name: CREATE_COMPANY,
        description: "Create a new company with the given name and currency. \
                      The company form is pre-filled and auto-submitted; \
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

            input.commander.send(
                form_replier(input.commander.clone(), args.form_id.into()),
                Message::Intent(Intent::CompanyName(args.company_name)),
            );

            Ok(to_value(Output { is_ok: true })?)
        }
        _ => Err(error_to_mcp(
            "unknown_tool",
            format!("unknown tool: {}", input.tool_name),
        )),
    }
}

fn form_replier(commander: Commander, form_id: FormId) -> Replier {
    let replier: Replier = new_replier(move |msg| {
        let commander = commander.clone();

        async move {
            let replier = forward_to_commander(commander.clone());

            let msg1 = Box::new(use_case_home::client::Message::FormRelated {
                form_id,
                message: msg,
            });

            commander.async_send_box(replier, msg1).await
        }
    });
    replier
}

fn forward_to_commander(commander: Commander) -> Replier {
    new_replier(move |msg| {
        let commander = commander.clone();
        async move {
            let replier = forward_to_commander(commander.clone());
            commander.async_send_box(replier, msg).await
        }
    })
}
