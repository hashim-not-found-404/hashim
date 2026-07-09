use crate::utility::utils;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub trait Coding {
    fn encode<T: Serialize>(data: &T) -> Vec<u8>;
    fn decode<'de, T: Deserialize<'de>>(data: &'de Vec<u8>) -> Result<T, utils::DynamicError>;
}

pub trait Regex: 'static {
    fn is_regex(s: &String) -> Result<(), String>;
}

pub trait RandomNumber: 'static {
    fn generate() -> u64;
}

pub enum Either<L, R> {
    One(L),
    Two(R),
}

pub trait JoinHandle {
    fn abort(&mut self) -> impl Future<Output = ()>;
}

pub trait Runtime: 'static {
    type JoinHandle<T>: JoinHandle;

    #[must_use = "this `output` you may want to abort"]
    fn abortable_spawn_local<F>(fut: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + 'static;

    fn spawn_local<F>(fut: F)
    where
        F: Future + 'static;

    fn timeout<T, F>(
        duration: Duration,
        fut: F,
    ) -> impl Future<Output = Result<T, utils::DynamicError>>
    where
        F: Future<Output = T>;

    fn sleep(duration: Duration) -> impl Future<Output = ()>;

    fn select<R1, R2, F1, F2>(fut1: F1, fut2: F2) -> impl Future<Output = Either<R1, R2>>
    where
        F1: Future<Output = R1>,
        F2: Future<Output = R2>;
}

pub trait Sender<T>: Clone {
    fn send(&mut self, t: T) -> impl Future<Output = Result<(), utils::DynamicError>>;
}

pub trait Receiver<T> {
    fn recv(&mut self) -> impl Future<Output = Result<T, utils::DynamicError>>;
}

pub trait MultiProducerSingleConsumer: 'static {
    type Sender<T>: Sender<T>;
    type Receiver<T>: Receiver<T>;
    fn channel<T>() -> (Self::Sender<T>, Self::Receiver<T>);
}
