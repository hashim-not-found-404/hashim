#[cfg(not(target_arch = "wasm32"))]
pub mod target {
    use my_core::utility::{
        traits::{Either, Runtime},
        utils::DynamicError,
    };
    use std::time::Duration;
    use tokio;

    pub struct S;

    impl Runtime for S {
        type JoinHandle<F> = super::join_handle::target::S<F>;

        fn abortable_spawn_local<F: Future + 'static>(fut: F) -> Self::JoinHandle<F::Output> {
            Self::JoinHandle::new(tokio::task::spawn_local(fut))
        }

        fn spawn_local<F>(fut: F)
        where
            F: Future + 'static,
        {
            tokio::task::spawn_local(fut);
        }

        async fn timeout<T, F: Future<Output = T>>(
            duration: Duration,
            fut: F,
        ) -> Result<T, DynamicError> {
            todo!()
        }

        async fn sleep(duration: Duration) {
            todo!()
        }

        async fn select<R1, R2, F1: Future<Output = R1>, F2: Future<Output = R2>>(
            fut1: F1,
            fut2: F2,
        ) -> Either<R1, R2> {
            tokio::select! {
                r1 = fut1 => {
                    Either::One(r1)
                }
                r2 = fut2 => {
                    Either::Two(r2)
                }
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub mod target {
    use super::*;
    use futures::{
        channel::oneshot,
        future::{Either as Eth, select},
    };
    use gloo_timers::future::TimeoutFuture;
    use my_core::utility::{
        traits::{Either, Runtime},
        utils::DynamicError,
    };
    use std::{pin::pin, time::Duration};
    use wasm_bindgen_futures::spawn_local;

    pub struct S;

    impl Runtime for S {
        type JoinHandle<T> = join_handle::target::S<T>;

        fn abortable_spawn_local<F: Future + 'static>(fut: F) -> Self::JoinHandle<F::Output> {
            let (sender_to_abort, mut receiver_to_abort) = oneshot::channel();
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

        fn spawn_local<F: Future + 'static>(fut: F) {
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
                Eth::Left((result, _)) => Ok(result),
                Eth::Right((_, _)) => Err("timeout".into()),
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

mod join_handle {
    #[cfg(not(target_arch = "wasm32"))]
    pub mod target {
        use my_core::utility::traits::JoinHandle;

        pub struct S<T>(pub tokio::task::JoinHandle<T>);

        impl<T> JoinHandle for S<T> {
            async fn abort(&mut self) {
                self.0.abort();
            }
        }

        impl<T> S<T> {
            pub fn new(t: tokio::task::JoinHandle<T>) -> Self {
                Self(t)
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub mod target {
        use futures::{channel::oneshot, lock::Mutex};
        use my_core::utility::traits::JoinHandle;
        use std::sync::Arc;

        pub struct S<T> {
            pub output: Arc<Mutex<Option<T>>>,
            aborter: Option<oneshot::Sender<()>>,
        }

        impl<T> JoinHandle for S<T> {
            async fn abort(&mut self) {
                match self.aborter.take() {
                    Some(s) => {
                        s.send(());
                    }
                    None => return,
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
}
