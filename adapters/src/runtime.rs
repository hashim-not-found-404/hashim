#[cfg(not(target_arch = "wasm32"))]
pub mod m {
    use crate::prelude::*;
    use std::time::Duration;

    pub struct S;

    impl Runtime for S {
        type JoinHandel<F> = join_handel::m::S<F>;

        fn abortable_spawn_local<F: Future + 'static>(fut: F) -> Self::JoinHandel<F::Output> {
            Self::JoinHandel::new(tokio::task::spawn_local(fut))
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
pub mod m {
    use crate::prelude::*;
    use futures::future::{Either as Eth, select};
    use gloo_timers::future::TimeoutFuture;
    use std::{pin::pin, time::Duration};
    use wasm_bindgen_futures::spawn_local;

    pub struct S;

    impl Runtime for S {
        type JoinHandel<T> = join_handel::m::S<T>;

        fn abortable_spawn_local<F: Future + 'static>(fut: F) -> Self::JoinHandel<F::Output> {
            let (sender_to_abort, mut receiver_to_abort) = actors::m::S::channel();
            let join_handel = Self::JoinHandel::new(sender_to_abort);

            let output_place = join_handel.output.clone();
            spawn_local(async move {
                match Self::select(fut, receiver_to_abort.recv()).await {
                    Either::One(a) => *output_place.lock().await = Some(a),
                    Either::Two(_) => return,
                }
            });

            join_handel
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
