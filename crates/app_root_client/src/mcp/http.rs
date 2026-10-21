//! Streamable HTTP transport for rmcp.
//!
//! rmcp ships this transport behind the `transport-streamable-http-server`
//! feature. We mount `StreamableHttpService` at `/mcp` on a minimal axum
//! router. Session management is per-connection and handled by rmcp via
//! `LocalSessionManager`; each session gets its own `McpServer` built from the
//! same `McpBridge`, so all sessions mutate the same UI model.

use crate::mcp::MCP_PATH;
use crate::mcp::McpBridge;
use crate::mcp::server::McpServer;
use anyhow::Result;
use axum::Router;
use axum::serve;
use rmcp::transport::StreamableHttpServerConfig;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::tower::StreamableHttpService;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

pub(crate) async fn serve_http(bridge: Arc<McpBridge>, addr: String) -> Result<()> {
    let service: StreamableHttpService<McpServer, LocalSessionManager> = StreamableHttpService::new(
        move || Ok(McpServer::new(bridge.clone())),
        Arc::new(LocalSessionManager::default()),
        StreamableHttpServerConfig::default(),
    );

    let router = Router::new().nest_service(MCP_PATH, service);
    let listener = TcpListener::bind(&addr).await?;
    serve(listener, router).await?;

    info!("mcp streamable-http listening on http://{addr}{MCP_PATH}");
    Ok(())
}
