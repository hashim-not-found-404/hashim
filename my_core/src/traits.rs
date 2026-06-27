use crate::prelude::*;

pub trait RowId:
    for<'a> TryFrom<&'a db_types::UuidType, Error = ()> + ToString + Clone + Hash + Eq
{
    fn to_uuid(&self) -> db_types::UuidType {
        db_types::UuidType(self.to_string())
    }
    fn generate() -> Self;
    fn get_time_as_seconds(&self) -> u64;
}

pub trait Regex {
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
    fn new() -> impl Future<Output = Self>;
    fn get_client(&self) -> impl Future<Output = Result<Self::Client, DynamicError>>;
}

pub trait DBClient {
    type RowId: RowId;
    type HashedPassword: HashedPassword;

    type Txn<'a>: DBTransaction
    where
        Self: 'a;

    fn begin_transaction(&mut self) -> impl Future<Output = Result<Self::Txn<'_>, DynamicError>>;

    fn write_nonce_if_not_used(
        &mut self,
        nonce: &Self::RowId,
    ) -> impl Future<Output = Result<bool /* is nonce used */, DynamicError>>;

    // here we just do read we dont do here any set or check

    fn read_sign_in(
        &mut self,
        user_id: &String,
    ) -> impl Future<Output = Result<Option<(Self::RowId, Self::HashedPassword)>, DynamicError>>;
    fn read_roles_for_user(
        &mut self,
        users_uuids: &HashSet<Self::RowId>,
    ) -> impl Future<Output = Result<server_methods::AllRoles<Self::RowId>, DynamicError>>;
    fn read_list_company_and_branch(
        &mut self,
        user_uuid: &Self::RowId,
    ) -> impl Future<Output = Result<Vec<ResourceInfo>, DynamicError>>;
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

    fn commit_transaction(
        self,
    ) -> impl Future<Output = Result<Result<(), domain_errors::AtCommit>, DynamicError>>;
    fn rollback_transaction(self) -> impl Future<Output = Result<(), DynamicError>>;

    fn read_sign_up(
        &mut self,
        new_uuid: &Self::RowId,
        user_id: &String,
    ) -> impl Future<
        Output = Result<
            (
                bool, /* is new_uuid exist */
                bool, /* is user_id exist */
            ),
            DynamicError,
        >,
    >;
    fn write_sign_up(
        &mut self,
        new_uuid: &Self::RowId,
        user_id: &String,
        hashed_password: &Self::HashedPassword,
        user_name: &Option<String>,
    ) -> impl Future<Output = Result<(), DynamicError>>;

    fn read_create_company(
        &mut self,
        new_uuid: &Self::RowId,
    ) -> impl Future<Output = Result<bool /* is new_uuid exist */, DynamicError>>;
    fn write_create_company(
        &mut self,
        new_uuid: &Self::RowId,
        user_uuid: &Self::RowId,
        user_role: &db_types::Role,
        company_name: &String,
        currency: &db_types::Currency,
    ) -> impl Future<Output = Result<(), DynamicError>>;

    fn read_create_company_branch(
        &mut self,
        new_uuid: &Self::RowId,
        user_uuid: &Self::RowId,
        company_belong: &Self::RowId,
        branch_name: &String,
    ) -> impl Future<
        Output = Result<
            (
                Vec<db_types::Role>, /* user roles */
                bool,                /* is new_uuid exist */
                bool,                /* is company_belong exist */
                bool,                /* is branch_name used */
            ),
            DynamicError,
        >,
    >;
    fn write_create_company_branch(
        &mut self,
        new_uuid: &Self::RowId,
        company_belong: &Self::RowId,
        branch_name: &String,
        location: &db_types::Location,
        currency: &db_types::Currency,
        user_uuid: &Self::RowId,
        user_role: &db_types::Role,
    ) -> impl Future<Output = Result<(), DynamicError>>;
}

pub trait WebSocketOp: Sized {
    fn connect(url: &str) -> impl Future<Output = Result<Self, DynamicError>>;
    fn send_bin(&self, data: &Vec<u8>) -> impl Future<Output = Result<(), DynamicError>>;
    fn receive_bin(&self) -> impl Future<Output = Result<Vec<u8>, DynamicError>>;
}

pub trait Coding {
    fn encode<T: Serialize>(data: &T) -> Vec<u8>;
    fn decode<'de, T: Deserialize<'de>>(data: &'de Vec<u8>) -> Result<T, DynamicError>;
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

    fn timeout<T, F>(duration: Duration, fut: F) -> impl Future<Output = Result<T, DynamicError>>
    where
        F: Future<Output = T>;

    fn sleep(duration: Duration) -> impl Future<Output = ()>;

    fn select<R1, R2, F1, F2>(fut1: F1, fut2: F2) -> impl Future<Output = Either<R1, R2>>
    where
        F1: Future<Output = R1>,
        F2: Future<Output = R2>;
}

pub trait Sender<T>: Clone {
    fn send(&mut self, t: T) -> impl Future<Output = Result<(), DynamicError>>;
}

pub trait Receiver<T> {
    fn recv(&mut self) -> impl Future<Output = Result<T, DynamicError>>;
}

pub trait MultiProducerSingleConsumer {
    type Sender<T>: Sender<T>;
    type Receiver<T>: Receiver<T>;
    fn channel<T>() -> (Self::Sender<T>, Self::Receiver<T>);
}

pub trait CacheIO: Sized {
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
    fn get_jwt(&self, user_uuid: &db_types::UuidType) -> impl Future<Output = Option<String>>;

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

pub trait AllServerTypes: 'static
where
    for<'a> <Self::Cli as DBClient>::Txn<'a>:
        DBTransaction<RowId = Self::Id, HashedPassword = Self::Auth>,
{
    type Rn: RandomNumber;
    type Rt: Runtime;
    type Id: RowId;
    type Mpsc: MultiProducerSingleConsumer;
    type Ed: Coding;

    type Rg: Regex;
    type Auth: HashedPassword;
    type Jwt: JWT<UserId = Self::Id, JsonWebToken = String>;

    type Db: Database<Client = Self::Cli>;
    type Cli: DBClient<RowId = Self::Id, HashedPassword = Self::Auth>;
    type Ws: WSServer;
}

pub trait AllClientTypes: Default + Clone + 'static {
    type Rn: RandomNumber;
    type Rt: Runtime;
    type Id: RowId;
    type Mpsc: MultiProducerSingleConsumer;
    type Ed: Coding;

    type Ws: WebSocketOp;
    type Ch: CacheIO;

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
