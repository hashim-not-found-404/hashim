use crate::{db_types, request_response::*};

pub trait RowId: TryFrom<String> + Clone {
    fn generate() -> Self;
}

pub trait Functions {
    fn is_regex(s: &String) -> Result<(), String>;
}

pub trait TransactionNumber {
    fn generate() -> u64;
}

pub trait HashedPassword {
    fn sign_up(password: String) -> Self;
    fn sign_in(password: String, password_hash: Self) -> bool;
}

pub trait JWT {
    type UserId: RowId;
    type JsonWebToken: From<String> + Into<String>;
    fn sign(&self, user_id: &Self::UserId) -> Self::JsonWebToken;
    fn validate(&self, token: Self::JsonWebToken) -> Result<Self::UserId, ()>;
}

pub trait Database {
    type Error;
    type Client: DBClient;
    async fn get_client(&self) -> Result<Self::Client, Self::Error>;
}

pub trait DBClient {
    type Error;
    type Txn<'a>: DBTransaction
    where
        Self: 'a;
    async fn begin_transaction(&mut self) -> Result<Self::Txn<'_>, Self::Error>;
    // here we just do read we dont do here any set or check
    // and all the read for the chack and all of it not transaction
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

    /// return true if new
    async fn insert_transaction_if_new(
        &mut self,
        transaction_number: u64,
    ) -> Result<bool, Self::Error>;

    async fn does_he_have_access_to_here(
        &mut self,
        accepted_roles: &[custom_types::Role],
        company_or_branch: &db_types::DataGroup<Self::RowId>,
        user_id: &Self::RowId,
    ) -> Result<bool, Self::Error>;

    async fn select_user_rowid_and_password_hash(
        &mut self,
        user_id: &String,
    ) -> Result<Option<(Self::RowId, Self::HashedPassword)>, Self::Error>;
    async fn select_all_companies_and_branches_for_the_user(
        &mut self,
        user_id: &Self::RowId,
    ) -> Result<Option<Vec<custom_types::Company>>, Self::Error>;

    async fn insert_role(
        &mut self,
        row_id: &Self::RowId,
        role: &custom_types::Role,
        data_group: &db_types::DataGroup<Self::RowId>,
        user_id: &Self::RowId,
    ) -> Result<(), Self::Error>;
    async fn insert_company(
        &mut self,
        row_id: &Self::RowId,
        name: &String,
        currency: &custom_types::Currency,
    ) -> Result<(), Self::Error>;
    async fn insert_company_branch(
        &mut self,
        row_id: &Self::RowId,
        company_belong: &Self::RowId,
        name: &String,
        location: &custom_types::Location,
        currency: &custom_types::Currency,
    ) -> Result<(), Self::Error>;
    async fn insert_user(
        &mut self,
        row_id: &Self::RowId,
        name: &Option<String>,
        user_id: &String,
        hashed_password: &Self::HashedPassword,
    ) -> Result<Result<(), domain_errors::AtInsertUserId>, Self::Error>;
}

macro_rules! generate_api_backend_methods {
    ($path:ident) => {
        async fn $path(
            &self,
            input: business_layer::Input,
        ) -> business_layer::Result<$path::Ok, $path::Error, Self::Error>;
    };
}

pub trait BackendRouts {
    type Error;
    generate_api_backend_methods!(sign_up);
    generate_api_backend_methods!(sign_in);
    generate_api_backend_methods!(get_all_user_roles);
    generate_api_backend_methods!(create_company);
    generate_api_backend_methods!(create_company_branch);
}
