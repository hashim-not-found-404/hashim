// #[cfg(not(target_arch = "wasm32"))]
// pub mod m {
//     use crate::prelude::*;
//     use std::time::Duration;

//     pub struct S;

//     impl Runtime for S {
//         fn spawn<F: Future>(fut: F) {
//             todo!()
//         }

//         async fn timeout<T, F: Future<Output = T>>(
//             duration: Duration,
//             fut: F,
//         ) -> Result<T, DynamicError> {
//             todo!()
//         }

//         async fn sleep(duration: Duration) {
//             todo!()
//         }

//         async fn select<L, R, F1: Future<Output = L>, F2: Future<Output = R>>(
//             fut1: F1,
//             fut2: F2,
//         ) -> Eather<L, R> {
//             todo!()
//         }
//     }
// }

// #[cfg(target_arch = "wasm32")]
pub mod m {
    use crate::prelude::*;
    use futures::future::{Either as Eth, select};
    use gloo_timers::future::TimeoutFuture;
    use my_core::traits;
    use std::{pin::pin, time::Duration};
    use wasm_bindgen_futures::spawn_local;

    pub struct S;

    impl Runtime for S {
        fn spawn<F: Future + 'static>(fut: F) {
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
