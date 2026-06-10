pub mod m {
    use crate::prelude::*;
    use futures::{SinkExt, channel::mpsc::UnboundedSender};
    

    pub struct S<T>(pub UnboundedSender<T>);
    impl<T> Sender<T> for S<T> {
        async fn send(&mut self, t: T) -> Result<(), DynamicError> {
            Ok(self.0.send(t).await?)
        }
    }

    impl<T> Clone for S<T> {
        fn clone(&self) -> Self {
            S(self.0.clone())
        }
    }

    impl<T> S<T> {
        pub fn new(t: UnboundedSender<T>) -> Self {
            Self(t)
        }
    }

    impl<T> PartialEq for S<T> {
        fn eq(&self, other: &Self) -> bool {
            false
        }
    }
}
