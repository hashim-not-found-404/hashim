use serde::Deserialize;
use use_case_get_all_journals_client::get_data;
use use_case_get_all_journals_domain::Input;
use use_case_prelude_mcp::*;

const LIST_JOURNALS: &str = "list_journals";

#[derive(Debug, JsonSchema, Deserialize)]
struct ListJournalsArgs {}

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

pub async fn call<'a>(input: CallInput<'a, (), Input>) -> Result<Value> {
    match input.tool_name {
        LIST_JOURNALS => {
            let _: ListJournalsArgs = parse_args(input.args)?;

            let ok = get_data(input.cache.clone(), false, input.global.clone()).await?;

            let a = match ok {
                Ok(a) => a,
                Err(a) => return Ok(to_value(a)?),
            };

            Ok(to_value(a)?)
        }
        _ => Err(error_to_mcp(
            "unknown_tool",
            format!("unknown tool: {}", input.tool_name),
        )),
    }
}
