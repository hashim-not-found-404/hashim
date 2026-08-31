use utility::types::DynamicError;

pub trait Sender<T>: Clone {
    fn send(&mut self, t: T) -> impl Future<Output = Result<(), DynamicError>>;
}

pub trait Receiver<T> {
    fn recv(&mut self) -> impl Future<Output = Result<T, DynamicError>>;
}

pub trait MultiProducerSingleConsumer: 'static {
    type Sender<T>: Sender<T>;
    type Receiver<T>: Receiver<T>;
    fn channel<T>() -> (Self::Sender<T>, Self::Receiver<T>);
}

#[cfg(feature = "infrastructure")]
pub mod target {
    use super::MultiProducerSingleConsumer;
    use super::mpsc_receiver;
    use super::mpsc_sender;
    use futures::channel::mpsc::unbounded;

    pub struct S;

    impl MultiProducerSingleConsumer for S {
        type Receiver<T> = mpsc_receiver::S<T>;
        type Sender<T> = mpsc_sender::S<T>;

        fn channel<T>() -> (Self::Sender<T>, Self::Receiver<T>) {
            let (tx, rx) = unbounded();
            (mpsc_sender::S::new(tx), mpsc_receiver::S::new(rx))
        }
    }
}

#[cfg(feature = "infrastructure")]
mod mpsc_receiver {
    use super::Receiver;
    use futures::channel::mpsc::UnboundedReceiver;
    use utility::types::DynamicError;

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

#[cfg(feature = "infrastructure")]
mod mpsc_sender {
    use super::Sender;
    use futures::SinkExt;
    use futures::channel::mpsc::UnboundedSender;
    use utility::types::DynamicError;

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
