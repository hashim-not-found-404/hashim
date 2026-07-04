use crate::{
    db_types, decider,
    request_response::{ResourceInfo, push_data},
    utils,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, future::Future, time::Duration};

pub trait Regex {
    fn is_regex(s: &String) -> Result<(), String>;
}

pub trait RandomNumber {
    fn generate() -> u64;
}

pub trait Database {
    type Client: DBClient;
    fn new() -> impl Future<Output = Self>;
    fn get_client(&self) -> impl Future<Output = Result<Self::Client, utils::DynamicError>>;
}

pub trait DBClient {
    type Txn<'a>: DBTransaction
    where
        Self: 'a;

    fn begin_transaction(
        &mut self,
    ) -> impl Future<Output = Result<Self::Txn<'_>, utils::DynamicError>>;

    fn write_nonce_if_not_used(
        &mut self,
        nonce: &db_types::UuidType,
    ) -> impl Future<Output = Result<bool /* is nonce used */, utils::DynamicError>>;

    // here we just do read we dont do here any set or check

    fn read_sign_in(
        &mut self,
        user_id: &String,
    ) -> impl Future<Output = Result<Option<(db_types::UuidType, String)>, utils::DynamicError>>;
    fn read_roles_for_user(
        &mut self,
        users_uuids: &HashSet<db_types::UuidType>,
    ) -> impl Future<Output = Result<server_methods::AllRoles, utils::DynamicError>>;
    fn read_list_company_and_branch(
        &mut self,
        user_uuid: &db_types::UuidType,
    ) -> impl Future<Output = Result<Vec<ResourceInfo>, utils::DynamicError>>;
}

pub mod domain_errors {
    #[derive(Debug)]
    pub enum AtCommit {
        DataIsChanged,
    }
}

pub trait DBTransaction {
    fn commit_transaction(
        self,
    ) -> impl Future<Output = Result<Result<(), domain_errors::AtCommit>, utils::DynamicError>>;
    fn rollback_transaction(self) -> impl Future<Output = Result<(), utils::DynamicError>>;

    fn read_sign_up(
        &mut self,
        new_uuid: &db_types::UuidType,
        user_id: &String,
    ) -> impl Future<
        Output = Result<
            (
                bool, /* is new_uuid exist */
                bool, /* is user_id exist */
            ),
            utils::DynamicError,
        >,
    >;
    fn write_sign_up(
        &mut self,
        data: &decider::sign_up::Ok,
    ) -> impl Future<Output = Result<(), utils::DynamicError>>;

    fn read_create_company(
        &mut self,
        new_uuid: &db_types::UuidType,
    ) -> impl Future<Output = Result<bool /* is new_uuid exist */, utils::DynamicError>>;
    fn write_create_company(
        &mut self,
        data: &decider::create_company::Ok,
    ) -> impl Future<Output = Result<(), utils::DynamicError>>;

    fn read_create_company_branch(
        &mut self,
        new_uuid: &db_types::UuidType,
        user_uuid: &db_types::UuidType,
        company_belong: &db_types::UuidType,
        branch_name: &String,
    ) -> impl Future<
        Output = Result<
            (
                Vec<db_types::Role>, /* user roles */
                bool,                /* is new_uuid exist */
                bool,                /* is company_belong exist */
                bool,                /* is branch_name used */
            ),
            utils::DynamicError,
        >,
    >;
    fn write_create_company_branch(
        &mut self,
        data: &decider::create_company_branch::Ok,
    ) -> impl Future<Output = Result<(), utils::DynamicError>>;
}

pub trait WSClient: Sized {
    fn connect(url: &str) -> impl Future<Output = Result<Self, utils::DynamicError>>;
    fn send_bin(&self, data: &Vec<u8>) -> impl Future<Output = Result<(), utils::DynamicError>>;
    fn receive_bin(&self) -> impl Future<Output = Result<Vec<u8>, utils::DynamicError>>;
}

pub trait Coding {
    fn encode<T: Serialize>(data: &T) -> Vec<u8>;
    fn decode<'de, T: Deserialize<'de>>(data: &'de Vec<u8>) -> Result<T, utils::DynamicError>;
}

pub enum Either<L, R> {
    One(L),
    Two(R),
}

pub trait JoinHandle {
    fn abort(&mut self) -> impl Future<Output = ()>;
}

pub trait Runtime {
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

pub trait MultiProducerSingleConsumer {
    type Sender<T>: Sender<T>;
    type Receiver<T>: Receiver<T>;
    fn channel<T>() -> (Self::Sender<T>, Self::Receiver<T>);
}

pub trait Cache: Sized {
    fn new() -> impl Future<Output = Self>;

    fn get_all_txn_input(
        &self,
    ) -> impl Future<Output = Vec<push_data::Txn<push_data::OperationsInput>>>;
    fn write_txn_input(
        &self,
        txn: &push_data::Txn<push_data::OperationsInput>,
    ) -> impl Future<Output = ()>;
    fn write_txn_result(
        &self,
        txn: &push_data::Txn<push_data::OperationsResult>,
    ) -> impl Future<Output = ()>;
    fn mark_txn_input_as_faild(&self, txn_number: &u64) -> impl Future<Output = ()>;
    fn delete_txn_input(&self, txn_number: &u64) -> impl Future<Output = ()>;

    fn write_resource(&self, resource: &Vec<ResourceInfo>) -> impl Future<Output = ()>;
    fn get_jwt(
        &self,
        user_uuid: &db_types::UuidType,
    ) -> impl Future<Output = Option<db_types::JsonWebTokenType>>;

    fn read_sign_up(
        &self,
        new_uuid: &db_types::UuidType,
        user_id: &String,
    ) -> impl Future<
        Output = (
            bool, /* is new_uuid exist */
            bool, /* is user_id exist */
        ),
    >;
    fn read_sign_in(
        &self,
        user_id: &String,
    ) -> impl Future<
        Output = Option<(
            db_types::UuidType, /* user uuid */
            Option<String>,     /* user name */
            bool,               /* does he have jwt */
        )>,
    >;
    fn read_list_company_and_branch(
        &self,
        user_uuid: &db_types::UuidType,
    ) -> impl Future<Output = Vec<ResourceInfo>>;
    fn read_create_company_branch(
        &self,
        user_uuid: &db_types::UuidType,
        company_belong: &db_types::UuidType,
        company_branch_name: &String,
    ) -> impl Future<
        Output = (
            Vec<db_types::Role>, /* roles at company */
            bool,                /* is company exist */
            bool,                /* is branch name used */
        ),
    >;
}

pub enum WSMessage {
    Binary(Vec<u8>),
    Close,
}

pub trait WSServer {
    fn send_bin(&mut self, bin: Vec<u8>) -> impl Future<Output = Result<(), utils::DynamicError>>;
    fn receive(&mut self) -> impl Future<Output = Result<WSMessage, utils::DynamicError>>;
    fn close(self) -> impl Future<Output = Result<(), utils::DynamicError>>;
}

pub trait HashimSignal<T: Default + Clone>: Default {
    fn reset(&self) {
        self.set(T::default());
    }
    fn read(&self) -> T;
    fn set(&self, v: T);
}

pub trait AllServerTypes: 'static
where
    for<'a> <Self::Cli as DBClient>::Txn<'a>: DBTransaction,
{
    type Rn: RandomNumber;
    type Rt: Runtime;
    type Id: RowId;
    type Mpsc: MultiProducerSingleConsumer;
    type Ed: Coding;
    type Rg: Regex;

    type Auth: HashedPassword;
    type Jwt: JWT;

    type Db: Database<Client = Self::Cli>;
    type Cli: DBClient;
    type Ws: WSServer;
}

pub trait AllClientTypes: 'static + Default + Clone {
    type Rn: RandomNumber;
    type Rt: Runtime;
    type Id: RowId;
    type Mpsc: MultiProducerSingleConsumer;
    type Ed: Coding;
    type Rg: Regex;

    type Ch: Cache;
    type Ws: WSClient;

    // signals
    type String: HashimSignal<String>;
    type Dialog: HashimSignal<ui_model::Dialog>;
    type Uuid: HashimSignal<db_types::UuidType>;
    type OptionUuid: HashimSignal<Option<db_types::UuidType>>;
    type Bool: HashimSignal<bool>;
    type StringVec: HashimSignal<String>;
    type Currency: HashimSignal<db_types::Currency>;
    type Location: HashimSignal<db_types::Location>;
    type CompanyAndBranchList: HashimSignal<Vec<db_types::Company>>;

    type Navigator: HashimSignal<ui_model::Navigator>;
}
