mod app_ui;
#[cfg(not(target_arch = "wasm32"))]
mod mcp_wire;
mod model;
mod use_case_navigator;
mod utils;
mod wire;

use anyhow::Result;
use dioxus::launch;

#[cfg(target_arch = "wasm32")]
fn main() {
    use console_error_panic_hook::set_once;
    set_once();

    use tracing_subscriber::prelude::*;
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .without_time()
                .with_target(false)
                .with_level(false)
                .with_ansi(false)
                .with_writer(tracing_web::MakeWebConsoleWriter::new()),
        )
        .init();

    launch(app_ui::App);
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<()> {
    use std::sync::mpsc::channel;
    use std::thread::Builder;

    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .with_level(false)
        .with_writer(std::io::stderr)
        .init();

    let (init_done_tx, init_done_rx) = channel::<()>();
    Builder::new()
        .name("local-executor".into())
        .spawn(move || -> Result<()> {
            use crate::utils::init_commander_and_model;
            use std::future::pending;
            use tokio::runtime::Builder;
            use tokio::task::LocalSet;

            let rt = Builder::new_current_thread().enable_all().build()?;

            let local = LocalSet::new();
            local.block_on(&rt, async move {
                init_commander_and_model();
                spawn_mcp_server();
                let _ = init_done_tx.send(());

                pending::<()>().await;
            });

            Ok(())
        })?;

    init_done_rx.recv()?;

    launch(app_ui::App);

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn spawn_mcp_server() {
    use infrastructure::runtime::Rt;
    use infrastructure::runtime::Runtime;
    use kernel::mcp::McpProviderRegistration;
    use rmcp::ErrorData;
    use rmcp::RoleServer;
    use rmcp::ServerHandler;
    use rmcp::model::CallToolRequestParams;
    use rmcp::model::CallToolResponse;
    use rmcp::model::CallToolResult;
    use rmcp::model::ContentBlock;
    use rmcp::model::ListToolsResult;
    use rmcp::model::PaginatedRequestParams;
    use rmcp::model::ServerCapabilities;
    use rmcp::model::ServerConfig;
    use rmcp::model::Tool;
    use rmcp::service::RequestContext;
    use rmcp::transport::streamable_http_server::StreamableHttpServerConfig;
    use rmcp::transport::streamable_http_server::StreamableHttpService;
    use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
    use std::borrow::Cow;
    use std::sync::Arc;

    #[derive(Clone)]
    struct Handler;

    impl ServerHandler for Handler {
        fn get_info(&self) -> ServerConfig {
            let mut config = ServerConfig::default();
            config.capabilities = ServerCapabilities::builder().enable_tools().build();
            config
        }

        async fn list_tools(
            &self,
            _request: Option<PaginatedRequestParams>,
            _context: RequestContext<RoleServer>,
        ) -> Result<ListToolsResult, ErrorData> {
            let mut tools = Vec::new();
            for reg in inventory::iter::<McpProviderRegistration> {
                let provider = (reg.make)();
                for def in provider.tools() {
                    let schema_obj = def.input_schema.as_object().cloned().unwrap_or_default();
                    tools.push(Tool::new(
                        Cow::Borrowed(def.name),
                        Cow::Borrowed(def.description),
                        Arc::new(schema_obj),
                    ));
                }
            }
            Ok(ListToolsResult::with_all_items(tools))
        }

        async fn call_tool(
            &self,
            request: CallToolRequestParams,
            _context: RequestContext<RoleServer>,
        ) -> Result<CallToolResponse, ErrorData> {
            let args = match request.arguments {
                Some(m) => serde_json::Value::Object(m),
                None => serde_json::json!({}),
            };

            for reg in inventory::iter::<McpProviderRegistration> {
                let provider = (reg.make)();
                for def in provider.tools() {
                    if def.name == request.name {
                        let result: CallToolResult = match provider.call(&request.name, args).await
                        {
                            Ok(v) => CallToolResult::success(vec![ContentBlock::text(
                                serde_json::to_string(&v).unwrap_or_default(),
                            )]),
                            Err(e) => {
                                CallToolResult::error(vec![ContentBlock::text(format!("{e:?}"))])
                            }
                        };
                        return Ok(result.into());
                    }
                }
            }

            Ok(CallToolResult::error(vec![ContentBlock::text("unknown tool")]).into())
        }
    }

    Rt::spawn_local(async move {
        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 8083));

        let service = StreamableHttpService::new(
            || Ok(Handler),
            LocalSessionManager::default().into(),
            StreamableHttpServerConfig::default(),
        );

        let router = axum::Router::new().nest_service("/mcp", service);

        let listener = match tokio::net::TcpListener::bind(addr).await {
            Ok(l) => l,
            Err(e) => {
                eprintln!("MCP HTTP server: failed to bind {addr}: {e}");
                return;
            }
        };

        eprintln!("MCP Streamable HTTP server listening on http://{addr}/mcp");
        let _ = axum::serve(listener, router).await;
    });
}
