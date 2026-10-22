//! MCP-over-Streamable-HTTP server, native-only and opt-in via `--mcp`.
//!
//! Gated by `#[cfg(all(not(target_arch = "wasm32"), feature = "mcp"))]` on the
//! parent `main.rs`. Start it with `--mcp`; the endpoint is served at
//! `http://127.0.0.1:8083/mcp` by default (override with `MCP_ADDR`).

mod server;

use crate::model::TypeGlobalModel;
use crate::utils::async_send;
use crate::utils::with_model;
use anyhow::Result;
use anyhow::bail;
use axum::Router;
use axum::serve as axum_serve;
use rmcp::transport::StreamableHttpServerConfig;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::tower::StreamableHttpService;
use server::McpServer;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use tokio::net::TcpListener;
use tokio::time::sleep;
use tracing::info;
use utility::ui_effect::MessageTrait;

pub(crate) const DEFAULT_ADDR: &str = "127.0.0.1:8083";
pub(crate) const MCP_PATH: &str = "/mcp";

/// Dispatch-and-wait helper. Sends a real intent through the same `Commander`
/// the UI uses, then polls the model until a caller-supplied `postcondition`
/// becomes true (or times out). Reads use `with_model` directly.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct McpBridge;

impl McpBridge {
    pub(crate) async fn dispatch(
        &self,
        message: Box<dyn MessageTrait>,
        postcondition: impl Fn(&TypeGlobalModel) -> bool,
    ) -> Result<()> {
        self.dispatch_with_timeout(message, postcondition, Duration::from_secs(10))
            .await
    }

    pub(crate) async fn dispatch_with_timeout(
        &self,
        message: Box<dyn MessageTrait>,
        postcondition: impl Fn(&TypeGlobalModel) -> bool,
        timeout: Duration,
    ) -> Result<()> {
        async_send(message).await?;
        // Give the reducer on the UI thread a chance to run before polling.
        sleep(Duration::from_millis(20)).await;
        let start = Instant::now();
        loop {
            if with_model(|m| postcondition(m)) {
                return Ok(());
            }
            if start.elapsed() >= timeout {
                bail!("postcondition not observed within {timeout:?}");
            }
            sleep(Duration::from_millis(5)).await;
        }
    }

    pub(crate) fn read<T>(&self, f: impl FnOnce(&TypeGlobalModel) -> T) -> T {
        with_model(f)
    }
}

pub(crate) async fn serve(bridge: Arc<McpBridge>, addr: String) -> Result<()> {
    let service: StreamableHttpService<McpServer, LocalSessionManager> = StreamableHttpService::new(
        move || Ok(McpServer::new(bridge.clone())),
        Arc::new(LocalSessionManager::default()),
        StreamableHttpServerConfig::default(),
    );
    let router = Router::new().nest_service(MCP_PATH, service);
    let listener = TcpListener::bind(&addr).await?;

    info!("mcp streamable-http listening on http://{addr}{MCP_PATH}");
    axum_serve(listener, router).await?;
    Ok(())
}
