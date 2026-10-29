use anyhow::Result;
use cache::utils::MyUuidConverter;
use schemars::JsonSchema;
use schemars::schema_for;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use serde_json::to_value;
use utility::mcp::CallInput;
use utility::mcp::McpToolDef;
use utility::mcp::error_to_mcp;
use utility::mcp::parse_args;

const LIST_JOURNALS: &str = "list_journals";

#[derive(Debug, JsonSchema, Deserialize)]
struct ListJournalsArgs {}

#[derive(Debug, Serialize)]
struct JournalInfo {
    row_uuid: String,
    name: String,
    is_it_contain_quantity: bool,
}

#[derive(Debug, Serialize)]
struct ListJournalsOutput {
    journals: Vec<JournalInfo>,
}

pub fn tools() -> Vec<McpToolDef> {
    vec![McpToolDef {
        name: LIST_JOURNALS,
        description: "List the journals of the currently selected company. \
                      The list is refreshed when the app home is mounted and whenever \
                      the selected company changes; if the result is empty right after \
                      a fresh page load, call again in a moment.",
        input_schema: schema_for!(ListJournalsArgs),
    }]
}

pub fn call<'a>(input: CallInput<'a, LocalModel, impl GlobalModel>) -> Result<Value> {
    match input.tool_name {
        LIST_JOURNALS => {
            let _: ListJournalsArgs = parse_args(input.args)?;

            let journals = input
                .local
                .list()
                .into_iter()
                .map(|d| JournalInfo {
                    row_uuid: d.row_uuid.to_string(),
                    name: d.name,
                    is_it_contain_quantity: d.is_it_contain_quantity,
                })
                .collect();

            Ok(to_value(ListJournalsOutput { journals })?)
        }
        _ => Err(error_to_mcp(
            "unknown_tool",
            format!("unknown tool: {}", input.tool_name),
        )),
    }
}
