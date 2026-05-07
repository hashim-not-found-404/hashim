// #[cfg(target_arch = "wasm32")]
pub mod m {
    use crate::prelude::*;
    use futures::{SinkExt, channel::mpsc::UnboundedSender};
    use std::sync::{Arc, Mutex};

    pub struct S<T>(pub Arc<Mutex<UnboundedSender<T>>>);
    impl<T> Sender<T> for S<T> {
        async fn send(&self, t: T) -> Result<(), DynamicError> {
            let r = self.0.lock().unwrap().send(t).await;
            match r {
                Ok(o) => Ok(o),
                Err(e) => Err(e.into()),
            }
        }
    }

    impl<T> Clone for S<T> {
        fn clone(&self) -> Self {
            S(self.0.clone())
        }
    }

    impl<T> S<T> {
        pub fn new(t: UnboundedSender<T>) -> Self {
            Self(Arc::new(Mutex::new(t)))
        }
    }
}
