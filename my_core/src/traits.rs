use crate::prelude::*;

pub trait RowId: TryFrom<String> + Clone {
    fn generate() -> Self;
}

pub trait Functions {
    fn is_regex(s: &String) -> Result<(), String>;
}

pub trait RandomNumber {
    fn generate() -> u64;
}

pub trait HashedPassword {
    fn sign_up(password: String) -> Self;
    fn sign_in(password: String, password_hash: Self) -> bool;
}

pub trait JWT {
    type UserId: RowId;
    type JsonWebToken: From<String> + Into<String>;
    fn sign(&self, user_uuid: &Self::UserId) -> Self::JsonWebToken;
    fn validate(&self, token: Self::JsonWebToken) -> Result<Self::UserId, ()>;
}

pub trait Database {
    type Error;
    type Client: DBClient;
    async fn get_client(&self) -> Result<Self::Client, Self::Error>;
}

pub trait DBClient {
    type Error;
    type RowId: RowId;
    type HashedPassword: HashedPassword;

    type Txn<'a>: DBTransaction
    where
        Self: 'a;

    async fn begin_transaction(&mut self) -> Result<Self::Txn<'_>, Self::Error>;
    // here we just do read we dont do here any set or check

    async fn read_sign_in(
        &mut self,
        user_id: &String,
    ) -> Result<Option<(Self::RowId, Self::HashedPassword)>, Self::Error>;
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
    type Error;
    type RowId: RowId;
    type HashedPassword: HashedPassword;

    async fn commit_transaction(self) -> Result<Result<(), domain_errors::AtCommit>, Self::Error>;
    async fn rollback_transaction(self) -> Result<(), Self::Error>;

    async fn read_sign_up(
        &mut self,
        user_id: &String,
    ) -> Result<bool /* is new user */, Self::Error>;
    async fn write_sign_up(
        &mut self,
        user_id: &String,
        hashed_password: &Self::HashedPassword,
        user_name: &Option<String>,
    ) -> Result<Self::RowId /* is uuid of the user */, Self::Error>;
}

macro_rules! generate_api_backend_methods {
    ($path:ident) => {
        async fn $path(
            &self,
            input: transport_layer::Input<$path::Input>,
        ) -> Result<transport_layer::Result<$path::Result>, Self::Error>;
    };
}

pub trait BackendRouts {
    type Error;
    async fn sign_up(&self, input: sign_up::Input) -> Result<sign_up::Result, Self::Error>;
    async fn sign_in(&self, input: sign_in::Input) -> Result<sign_in::Result, Self::Error>;
    // generate_api_backend_methods!(get_all_user_roles);
    // generate_api_backend_methods!(create_company);
    // generate_api_backend_methods!(create_company_branch);
}
