use crate::prelude::*;

pub trait RowId: for<'a> TryFrom<&'a String, Error = ()> + ToString + Clone + Hash + Eq {
    fn generate() -> Self;
    fn get_time_as_seconds(&self) -> u64;
}

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
    fn new() -> Self;
    fn sign(&self, user_uuid: &Self::UserId) -> Self::JsonWebToken;
    fn validate(&self, token: Self::JsonWebToken) -> Option<Self::UserId>;
}

pub trait Database {
    type Client: DBClient;
    async fn new() -> Self;
    async fn get_client(&self) -> Result<Self::Client, DynamicError>;
}

pub trait DBClient {
    type RowId: RowId;
    type HashedPassword: HashedPassword;

    type Txn<'a>: DBTransaction
    where
        Self: 'a;

    async fn begin_transaction(&mut self) -> Result<Self::Txn<'_>, DynamicError>;

    async fn write_nonce_if_not_used(
        &mut self,
        nonce: &Self::RowId,
    ) -> Result<bool /* is nonce used */, DynamicError>;

    // here we just do read we dont do here any set or check

    async fn read_sign_in(
        &mut self,
        user_id: &String,
    ) -> Result<Option<(Self::RowId, Self::HashedPassword)>, DynamicError>;
    async fn read_roles_for_user(
        &mut self,
        users_uuids: &HashSet<Self::RowId>,
    ) -> Result<server_methods::AllRoles<Self::RowId>, DynamicError>;
}

pub mod domain_errors {
    #[derive(Debug)]
    pub enum AtCommit {
        DataIsChanged,
    }
}

pub trait DBTransaction {
    type RowId: RowId;
    type HashedPassword: HashedPassword;

    async fn commit_transaction(self) -> Result<Result<(), domain_errors::AtCommit>, DynamicError>;
    async fn rollback_transaction(self) -> Result<(), DynamicError>;

    async fn read_sign_up(
        &mut self,
        new_uuid: &Self::RowId,
        user_id: &String,
    ) -> Result<
        (
            bool, /* is new_uuid exist */
            bool, /* is user_id exist */
        ),
        DynamicError,
    >;
    async fn write_sign_up(
        &mut self,
        new_uuid: &Self::RowId,
        user_id: &String,
        hashed_password: &Self::HashedPassword,
        user_name: &Option<String>,
    ) -> Result<(), DynamicError>;

    async fn read_create_company(
        &mut self,
        new_uuid: &Self::RowId,
    ) -> Result<bool /* is new_uuid exist */, DynamicError>;
    async fn write_create_company(
        &mut self,
        resource_to_broadcast: &mut Vec<ResourceInfo>,
        new_uuid: &Self::RowId,
        user_uuid: &Self::RowId,
        user_role: &db_types::Role,
        company_name: &String,
        currency: &db_types::Currency,
    ) -> Result<(), DynamicError>;

    async fn read_create_company_branch(
        &mut self,
        new_uuid: &Self::RowId,
        company_belong: &Self::RowId,
        branch_name: &String,
    ) -> Result<
        (
            bool, /* is new_uuid exist */
            bool, /* is company_belong exist */
            bool, /* is branch_name used */
        ),
        DynamicError,
    >;
    async fn write_create_company_branch(
        &mut self,
        resource_to_broadcast: &mut Vec<ResourceInfo>,
        new_uuid: &Self::RowId,
        company_belong: &Self::RowId,
        branch_name: &String,
        location: &db_types::Location,
        currency: &db_types::Currency,
        user_uuid: &Self::RowId,
        user_role: &db_types::Role,
    ) -> Result<(), DynamicError>;
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
    One(L),
    Two(R),
}

pub trait Runtime {
    fn spawn_local<F>(fut: F)
    where
        F: Future + 'static;

    async fn timeout<T, F>(duration: Duration, fut: F) -> Result<T, DynamicError>
    where
        F: Future<Output = T>;

    async fn sleep(duration: Duration);

    async fn select<R1, R2, F1, F2>(fut1: F1, fut2: F2) -> Either<R1, R2>
    where
        F1: Future<Output = R1>,
        F2: Future<Output = R2>;
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

pub trait CacheIO: Sized {
    async fn new() -> Self;

    async fn get_all_txn_input(&self) -> Vec<push_data::Txn<push_data::OperationsInput>>;
    async fn write_txn_input(&self, txn: &push_data::Txn<push_data::OperationsInput>);
    async fn write_txn_result(&self, txn: &push_data::Txn<push_data::OperationsResult>);
    async fn delete_txn_input(&self, txn_number: &u64);

    async fn write_resource(&self, resource: &Vec<ResourceInfo>);
    async fn get_jwt(&self, user_uuid: &db_types::RowIdType) -> Option<String>;

    async fn read_sign_up(
        &self,
        new_uuid: &db_types::RowIdType,
        user_id: &db_types::RowIdType,
    ) -> (
        bool, /* is new_uuid exist */
        bool, /* is user_id exist */
    );
}
