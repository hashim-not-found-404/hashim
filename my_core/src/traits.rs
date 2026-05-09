use crate::prelude::*;

pub trait RowId {}

pub trait Functions {
    fn is_regex(s: &String) -> Result<(), String>;
}

pub trait RandomNumber {
    fn generate() -> u64;
}

pub trait HashedPassword {
    fn sign_up(password: &String) -> Self;
    fn sign_in(password: &String, password_hash: &Self) -> bool;
}

pub trait JWT {
    type UserId: RowId;
    type JsonWebToken: From<String> + Into<String>;
    fn sign(&self, user_uuid: &Self::UserId) -> Self::JsonWebToken;
    fn validate(&self, token: Self::JsonWebToken) -> Option<Self::UserId>;
}

pub trait Database {
    type Client: DBClient;
    async fn get_client(&self) -> Result<Self::Client, DynamicError>;
}

pub trait DBClient {
    type RowId: RowId;
    type HashedPassword: HashedPassword;

    type Txn<'a>: DBTransaction
    where
        Self: 'a;

    async fn begin_transaction(&mut self) -> Result<Self::Txn<'_>, DynamicError>;

    // here we just do read we dont do here any set or check

    async fn read_sign_in(
        &mut self,
        user_id: &String,
    ) -> Result<Option<(Self::RowId, Self::HashedPassword)>, DynamicError>;
}

pub mod domain_errors {
    #[derive(Debug)]
    pub enum AtCommit {
        DataIsChanged,
    }
    #[derive(Debug)]
    pub enum AtInsertUserId {
        DuplicatedUserId,
    }
}

pub trait DBTransaction {
    type RowId: RowId;
    type HashedPassword: HashedPassword;

    async fn commit_transaction(self) -> Result<Result<(), domain_errors::AtCommit>, DynamicError>;
    async fn rollback_transaction(self) -> Result<(), DynamicError>;

    async fn read_sign_up(
        &mut self,
        user_id: &String,
    ) -> Result<bool /* is new user */, DynamicError>;
    async fn write_sign_up(
        &mut self,
        user_id: &String,
        hashed_password: &Self::HashedPassword,
        user_name: &Option<String>,
    ) -> Result<Self::RowId /* is uuid of the user */, DynamicError>;
}

pub trait WebSocketOp: Sized {
    async fn connect(url: &str) -> Result<Self, DynamicError>;
    async fn send_bin(&self, data: &Vec<u8>) -> Result<(), DynamicError>;
    async fn receive_bin(&self) -> Result<Vec<u8>, DynamicError>;
}

pub trait Coding {
    fn encode<T: Serialize>(data: &T) -> Vec<u8>;
    fn decode<'de, T: Deserialize<'de>>(data: &'de Vec<u8>) -> Result<T, DynamicError>;
}

pub enum Either<L, R> {
    Left(L),
    Right(R),
}

pub trait Runtime {
    fn spawn<F: Future + 'static>(fut: F);
    async fn timeout<T, F: Future<Output = T>>(
        duration: Duration,
        fut: F,
    ) -> Result<T, DynamicError>;
    async fn sleep(duration: Duration);
    async fn select<L, R, F1: Future<Output = L>, F2: Future<Output = R>>(
        fut1: F1,
        fut2: F2,
    ) -> Either<L, R>;
}

pub trait Sender<T>: Clone {
    async fn send(&self, t: T) -> Result<(), DynamicError>;
}

pub trait Receiver<T> {
    async fn recv(&self) -> Result<T, DynamicError>;
}

pub trait MultiProducerSingleConsumer {
    type Sender<T>: Sender<T>;
    type Receiver<T>: Receiver<T>;
    fn channel<T>() -> (Self::Sender<T>, Self::Receiver<T>);
}

pub trait WAMP: Sized {
    type Sender<T>: Sender<T>;
    fn new(sender_to_error: Self::Sender<DynamicError>) -> Self;
    async fn connect_to_url(&self, url: &String);
    async fn close(self);

    async fn send_and_receive<SendType: Serialize, ReceiveType: for<'de> Deserialize<'de>>(
        &self,
        path: &String,
        payload: &SendType,
        timeout_in_secs: u32,
    ) -> Result<ReceiveType, DynamicError>;

    async fn receive_and_send<SendType: Serialize, ReceiveType: for<'de> Deserialize<'de>>(
        &self,
        path: &String,
        operation: impl AsyncFn(ReceiveType) -> SendType,
    ) -> Result<(), DynamicError>;

    async fn send_only<SendType: Serialize>(
        &self,
        path: &String,
        payload: &SendType,
    ) -> Result<(), DynamicError>;

    async fn receive_only<ReceiveType: for<'de> Deserialize<'de>>(
        &self,
        path: &String,
        operation: impl AsyncFn(ReceiveType) + 'static,
    );
}

pub trait CacheIO: Sized {
    async fn new() -> Result<Self, DynamicError>;
    async fn write_data(&self, data: &data_receiver::Input) -> Result<(), DynamicError>;
}
