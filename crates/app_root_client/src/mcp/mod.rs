//! MCP-over-Streamable-HTTP server for the native client build.
//!
//! Gated by `#[cfg(all(not(target_arch = "wasm32"), feature = "mcp-server"))]`
//! on the parent `main.rs`. Start it with `--mcp`; the MCP endpoint is served
//! at `http://127.0.0.1:8083/mcp` by default (override with `MCP_ADDR`).
//!
//! Every mutation is a real `Box<dyn MessageTrait>` sent through the same
//! `Commander` the Dioxus front-end uses; every read goes through the same
//! `with_model` the front-end uses. There is one model, one reducer pipeline,
//! and one intent bus.

mod bridge;
mod field_registry;
mod http;
mod server;
mod snapshot;
mod tools;

pub(crate) use bridge::McpBridge;

use anyhow::Result;
use std::sync::Arc;

pub(crate) const DEFAULT_ADDR: &str = "127.0.0.1:8083";
pub(crate) const MCP_PATH: &str = "/mcp";

pub(crate) async fn serve(bridge: Arc<McpBridge>, addr: String) -> Result<()> {
    http::serve_http(bridge, addr).await
}
