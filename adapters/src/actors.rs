pub mod target {
    use super::{mpsc_receiver, mpsc_sender};
    use my_core::utility::traits::MultiProducerSingleConsumer;

    pub struct S;

    impl MultiProducerSingleConsumer for S {
        type Sender<T> = mpsc_sender::target::S<T>;
        type Receiver<T> = mpsc_receiver::target::S<T>;

        fn channel<T>() -> (Self::Sender<T>, Self::Receiver<T>) {
            let (tx, rx) = futures::channel::mpsc::unbounded();
            (
                mpsc_sender::target::S::new(tx),
                mpsc_receiver::target::S::new(rx),
            )
        }
    }
}

mod mpsc_receiver {
    pub mod target {
        use futures::channel::mpsc::UnboundedReceiver;
        use my_core::utility::{traits::Receiver, traits::DynamicError};

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
    pub mod target {
        use futures::{SinkExt, channel::mpsc::UnboundedSender};
        use my_core::utility::{traits::Sender, traits::DynamicError};

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
