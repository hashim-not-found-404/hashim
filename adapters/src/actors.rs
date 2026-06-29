pub mod m {
    use super::*;
    use crate::internel_prelude::*;

    pub struct S;

    impl MultiProducerSingleConsumer for S {
        type Sender<T> = mpsc_sender::m::S<T>;
        type Receiver<T> = mpsc_receiver::m::S<T>;

        fn channel<T>() -> (Self::Sender<T>, Self::Receiver<T>) {
            let (tx, rx) = futures::channel::mpsc::unbounded();
            (mpsc_sender::m::S::new(tx), mpsc_receiver::m::S::new(rx))
        }
    }
}

mod mpsc_receiver {
    pub mod m {
        use crate::internel_prelude::*;
        use futures::channel::mpsc::UnboundedReceiver;

        pub struct S<T>(pub UnboundedReceiver<T>);
        impl<T> Receiver<T> for S<T> {
            async fn recv(&mut self) -> Result<T, DynamicError> {
                Ok(self.0.recv().await?)
            }
        }

        impl<T> S<T> {
            pub fn new(t: UnboundedReceiver<T>) -> Self {
                Self(t)
            }
        }
    }
}

mod mpsc_sender {
    pub mod m {
        use crate::internel_prelude::*;
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
    }
}
