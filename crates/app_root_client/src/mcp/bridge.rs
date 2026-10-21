//! Dispatch-and-wait helper.
//!
//! `Commander::async_send_box` returns as soon as the message is enqueued on
//! the UI actor's mpsc channel — not after the reduction actually runs. Since
//! the MCP thread lives on a different OS thread than the UI thread, we can't
//! just await a result. Instead, we hand the reducer a real intent, then poll
//! the model until a caller-supplied `postcondition` becomes `true` (or we
//! time out).
//!
//! `with_model` (reading) and `with_mut` (the UI actor's writing) both go
//! through `MySignal`'s `std::sync::Mutex`, so a read never observes a torn
//! state. A read that lands mid-write simply blocks for microseconds.
//!
//! The default timeout is 10 s because some use cases show a 5 s consent
//! dialog before they finish. Tools whose postcondition can fire on dialog
//! open (submit_form, update_form_field) will therefore return early instead
//! of blocking for the full window.

use crate::model::TypeGlobalModel;
use crate::utils::async_send;
use crate::utils::with_model;
use anyhow::Result;
use anyhow::bail;
use std::time::Duration;
use std::time::Instant;
use tokio::time::sleep;
use utility::ui_effect::MessageTrait;

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
