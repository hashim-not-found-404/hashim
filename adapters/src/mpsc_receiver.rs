// #[cfg(target_arch = "wasm32")]
pub mod m {
    use crate::prelude::*;
    use futures::channel::mpsc::UnboundedReceiver;
    use std::sync::Mutex;

    pub struct S<T>(pub Mutex<UnboundedReceiver<T>>);
    impl<T> Receiver<T> for S<T> {
        async fn recv(&self) -> Result<T, DynamicError> {
            let r = self.0.lock().unwrap().recv().await;
            match r {
                Ok(o) => Ok(o),
                Err(e) => Err(e.into()),
            }
        }
    }

    impl<T> S<T> {
        pub fn new(t: UnboundedReceiver<T>) -> Self {
            Self(Mutex::new(t))
        }
    }
}
