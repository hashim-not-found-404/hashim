use anyhow::Result;
use std::time::Duration;

pub enum Either<L, R> {
    One(L),
    Two(R),
}

pub trait JoinHandle {
    fn abort(&mut self) -> impl Future<Output = ()>;
}

pub trait Runtime: 'static {
    type JoinHandle<T>: JoinHandle;

    #[must_use = "this `output` you may want to abort"]
    fn abortable_spawn_local<F>(fut: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + 'static;

    fn spawn_local<F>(fut: F)
    where
        F: Future + 'static;

    fn timeout<T, F>(duration: Duration, fut: F) -> impl Future<Output = Result<T>>
    where
        F: Future<Output = T>;

    fn sleep(duration: Duration) -> impl Future<Output = ()>;

    fn select<R1, R2, F1, F2>(fut1: F1, fut2: F2) -> impl Future<Output = Either<R1, R2>>
    where
        F1: Future<Output = R1>,
        F2: Future<Output = R2>;
}

pub type Rt = target::S;
pub type Jh<T> = join_handle::S<T>;

// ============================================================================
// NATIVE (non-wasm) — dedicated LocalSet thread
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
pub mod target {
    use super::Either;
    use super::Runtime;
    use anyhow::Result;
    use std::time::Duration;

    pub struct S;

    impl Runtime for S {
        type JoinHandle<F> = super::join_handle::S<F>;

        fn abortable_spawn_local<F>(fut: F) -> Self::JoinHandle<F::Output>
        where
            F: Future + 'static,
        {
            let handle = tokio::task::spawn_local(fut);
            super::join_handle::S::new(handle)
        }

        fn spawn_local<F>(fut: F)
        where
            F: Future + 'static,
        {
            tokio::task::spawn_local(fut);
        }

        async fn timeout<T, F: Future<Output = T>>(duration: Duration, fut: F) -> Result<T> {
            Ok(tokio::time::timeout(duration, fut).await?)
        }

        async fn sleep(duration: Duration) {
            tokio::time::sleep(duration).await;
        }

        async fn select<R1, R2, F1: Future<Output = R1>, F2: Future<Output = R2>>(
            fut1: F1,
            fut2: F2,
        ) -> Either<R1, R2> {
            tokio::select! {
                r1 = fut1 => Either::One(r1),
                r2 = fut2 => Either::Two(r2),
            }
        }
    }
}

// ============================================================================
// WASM — unchanged behavior
// ============================================================================

#[cfg(target_arch = "wasm32")]
pub mod target {
    use super::Either;
    use super::Runtime;
    use super::*;
    use anyhow::Result;
    use anyhow::bail;
    use futures::channel::oneshot;
    use futures::future::Either as Eth;
    use futures::future::select;
    use gloo_timers::future::TimeoutFuture;
    use std::pin::pin;
    use std::time::Duration;
    use wasm_bindgen_futures::spawn_local;

    pub struct S;

    impl Runtime for S {
        type JoinHandle<T> = join_handle::S<T>;

        fn abortable_spawn_local<F>(fut: F) -> Self::JoinHandle<F::Output>
        where
            F: Future + 'static,
        {
            let (sender_to_abort, receiver_to_abort) = oneshot::channel();
            let join_handle = Self::JoinHandle::new(sender_to_abort);

            let output_place = join_handle.output.clone();
            spawn_local(async move {
                match Self::select(fut, receiver_to_abort).await {
                    Either::One(a) => *output_place.lock().await = Some(a),
                    Either::Two(_) => return,
                }
            });

            join_handle
        }

        fn spawn_local<F>(fut: F)
        where
            F: Future + 'static,
        {
            spawn_local(async {
                fut.await;
            });
        }

        async fn timeout<T, F: Future<Output = T>>(duration: Duration, fut: F) -> Result<T> {
            let timeout_ms = duration.as_millis() as u32;
            let fut_pinned = pin!(fut);
            let timeout_pinned = pin!(TimeoutFuture::new(timeout_ms));
            match select(fut_pinned, timeout_pinned).await {
                Eth::Left((result, _)) => Ok(result),
                Eth::Right((_, _)) => bail!("timeout"),
            }
        }

        async fn sleep(duration: Duration) {
            gloo_timers::future::sleep(duration).await;
        }

        async fn select<L, R, F1: Future<Output = L>, F2: Future<Output = R>>(
            fut1: F1,
            fut2: F2,
        ) -> Either<L, R> {
            match select(pin!(fut1), pin!(fut2)).await {
                Eth::Left((result, _)) => Either::One(result),
                Eth::Right((result, _)) => Either::Two(result),
            }
        }
    }
}

// ============================================================================
// Join handle
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
mod join_handle {
    use super::JoinHandle;

    pub struct S<T>(pub(super) tokio::task::JoinHandle<T>);

    impl<T> JoinHandle for S<T> {
        async fn abort(&mut self) {
            self.0.abort();
        }
    }

    impl<T> S<T> {
        pub(super) fn new(t: tokio::task::JoinHandle<T>) -> Self {
            Self(t)
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod join_handle {
    use super::JoinHandle;
    use futures::channel::oneshot;
    use futures::lock::Mutex;
    use std::sync::Arc;

    pub struct S<T> {
        pub output: Arc<Mutex<Option<T>>>,
        aborter: Option<oneshot::Sender<()>>,
    }

    impl<T> JoinHandle for S<T> {
        async fn abort(&mut self) {
            if let Some(s) = self.aborter.take() {
                let _ = s.send(());
            }
        }
    }

    impl<T> S<T> {
        pub fn new(aborter: oneshot::Sender<()>) -> Self {
            Self {
                output: Arc::default(),
                aborter: Some(aborter),
            }
        }
    }
}
