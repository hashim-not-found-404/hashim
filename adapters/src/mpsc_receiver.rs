pub mod m {
    use crate::prelude::*;
    use futures::channel::mpsc::UnboundedReceiver;
    use std::sync::Mutex;

    pub struct S<T>(pub Mutex<UnboundedReceiver<T>>);
    impl<T> Receiver<T> for S<T> {
        async fn recv(&self) -> Result<T, DynamicError> {
            Ok(self.0.lock().unwrap().recv().await?)
        }
    }

    impl<T> S<T> {
        pub fn new(t: UnboundedReceiver<T>) -> Self {
            Self(Mutex::new(t))
        }
    }
}
