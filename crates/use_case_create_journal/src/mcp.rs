use crate::client::GlobalModel;
use crate::client::Intent;
use crate::client::LocalModel;
use crate::client::Message;
use anyhow::Result;
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

const CREATE_JOURNAL: &str = "create_journal";

#[derive(Debug, JsonSchema, Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateJournalArgs {
    /// form id of the form to write.
    form_id: u32,
    /// Company name. Must match an existing company visible to the user
    /// or empty to use the currently selected company.
    #[serde(default)]
    company_name: String,
    /// Journal name. Must be non-empty and unique within the company.
    name: String,
    /// Whether this journal tracks quantity in addition to amount.
    /// Quantity-tracking journals allow non-zero `quantity` on their entries;
    /// amount-only journals force `quantity = 0`.
    is_it_contain_quantity: bool,
}

#[derive(Debug, Serialize)]
struct Output {
    /// `true` once the intents were queued.
    /// Does NOT mean the server has responded — the journals list will
    /// refresh via the subscription broadcast on success.
    is_ok: bool,
}

pub fn tools() -> Vec<McpToolDef> {
    vec![McpToolDef {
        name: CREATE_JOURNAL,
        description: "Create a new journal under the given company. \
                      The journal form is pre-filled; \
                      the journals list will refresh once the server responds.",
        input_schema: schema_for!(CreateJournalArgs),
    }]
}

pub fn call<'a>(input: CallInput<'a, LocalModel, impl GlobalModel>) -> Result<Value> {
    match input.tool_name {
        CREATE_JOURNAL => {
            let args: CreateJournalArgs = parse_args(input.args)?;

            if args.name.trim().is_empty() {
                return Err(error_to_mcp("invalid_args", "name must not be empty"));
            }

            let form_id = args.form_id.into();
            let commander = input.commander.clone();

            form_sender(
                commander.clone(),
                form_id,
                Message::Intent(Intent::CompanyName(args.company_name)),
            );
            form_sender(
                commander.clone(),
                form_id,
                Message::Intent(Intent::Name(args.name)),
            );
            form_sender(
                commander,
                form_id,
                Message::Intent(Intent::IsItContainQuantity(args.is_it_contain_quantity)),
            );

            Ok(to_value(Output { is_ok: true })?)
        }
        _ => Err(error_to_mcp(
            "unknown_tool",
            format!("unknown tool: {}", input.tool_name),
        )),
    }
}
