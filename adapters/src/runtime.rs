#[cfg(not(target_arch = "wasm32"))]
pub mod m {
    use crate::prelude::*;
    use std::time::Duration;

    pub struct S;

    impl Runtime for S {
        fn spawn<F: Future>(fut: F) {
            todo!()
        }

        async fn timeout<T, F: Future<Output = T>>(
            duration: Duration,
            fut: F,
        ) -> Result<T, DynamicError> {
            todo!()
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub mod m {
    use crate::prelude::*;
    use futures::future::{Either, select};
    use gloo_net::websocket::{Message, futures::WebSocket};
    use gloo_timers::future::TimeoutFuture;
    use std::{pin::pin, time::Duration};
    use wasm_bindgen_futures::spawn_local;

    pub struct S;

    impl Runtime for S {
        fn spawn<F: Future + 'static>(fut: F) {
            // For WASM: spawn_local schedules the future on the browser's event loop
            spawn_local(async {
                fut.await;
            });
        }

        async fn timeout<T, F: Future<Output = T>>(
            duration: Duration,
            fut: F,
        ) -> Result<T, DynamicError> {
            let timeout_ms = duration.as_millis() as u32;

            let fut_pinned = pin!(fut);
            let timeout_pinned = pin!(TimeoutFuture::new(timeout_ms));

            match select(fut_pinned, timeout_pinned).await {
                Either::Left((result, _)) => Ok(result),
                Either::Right((_, _)) => Err("timeout".into()),
            }
        }
    }
}
