pub mod m {
    use crate::prelude::*;

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
