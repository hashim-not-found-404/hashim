#[cfg(not(target_arch = "wasm32"))]
pub mod m {
    use crate::prelude::*;

    pub struct S<T>(pub tokio::task::JoinHandle<T>);

    impl<T> JoinHandel for S<T> {
        fn abort(self) {
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
pub mod m {
    use crate::prelude::*;
    use futures::lock::Mutex;
    use std::sync::Arc;
    use wasm_bindgen_futures::spawn_local;

    pub struct S<T> {
        pub output: Arc<Mutex<Option<T>>>,
        aborter: mpsc_sender::m::S<()>,
    }

    impl<T> JoinHandel for S<T> {
        fn abort(self) {
            spawn_local(async move {
                let mut aborter = self.aborter;
                aborter.send(()).await;
            });
        }
    }

    impl<T> S<T> {
        pub fn new(aborter: mpsc_sender::m::S<()>) -> Self {
            Self {
                output: Arc::default(),
                aborter,
            }
        }
    }
}
