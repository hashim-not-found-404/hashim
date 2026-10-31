use crate::client::GlobalModel;
use crate::client::Intent;
use crate::client::LocalModel;
use crate::client::Message;
use use_case_home::mcp::form_sender;
use use_case_prelude_mcp::*;

const CREATE_BRANCH: &str = "create_branch";

#[derive(Debug, JsonSchema, Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateBranchArgs {
    /// form id of the form to write.
    form_id: u32,
    /// Company name. Must match an existing company visible to the user or empty if user select some company.
    #[serde(default)]
    company_name: String,
    /// Branch name. Must be non-empty.
    branch_name: String,
    /// Latitude in decimal degrees, in [-90, 90].
    latitude: f64,
    /// Longitude in decimal degrees, in [-180, 180].
    longitude: f64,
}

#[derive(Debug, Serialize)]
struct Output {}

pub fn tools() -> Vec<McpToolDef> {
    vec![McpToolDef {
        name: CREATE_BRANCH,
        description: "Create a new branch under the given company. \
                      The branch form is pre-filled; \
                      the branches list will refresh once the server responds.",
        input_schema: schema_for!(CreateBranchArgs),
    }]
}

pub fn call<'a>(input: CallInput<'a, LocalModel, impl GlobalModel>) -> Result<Value> {
    match input.tool_name {
        CREATE_BRANCH => {
            let args: CreateBranchArgs = parse_args(input.args)?;

            if args.branch_name.trim().is_empty() {
                return Err(error_to_mcp(
                    "invalid_args",
                    "branch_name must not be empty",
                ));
            }
            if !(args.latitude.is_finite() && (-90.0..=90.0).contains(&args.latitude)) {
                return Err(error_to_mcp(
                    "invalid_args",
                    format!("latitude {} out of range [-90, 90]", args.latitude),
                ));
            }
            if !(args.longitude.is_finite() && (-180.0..=180.0).contains(&args.longitude)) {
                return Err(error_to_mcp(
                    "invalid_args",
                    format!("longitude {} out of range [-180, 180]", args.longitude),
                ));
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
                Message::Intent(Intent::BranchName(args.branch_name)),
            );
            form_sender(
                commander.clone(),
                form_id,
                Message::Intent(Intent::Latitude(args.latitude)),
            );
            form_sender(
                commander,
                form_id,
                Message::Intent(Intent::Longitude(args.longitude)),
            );

            Ok(to_value(Output {})?)
        }
        _ => Err(error_to_mcp(
            "unknown_tool",
            format!("unknown tool: {}", input.tool_name),
        )),
    }
}
