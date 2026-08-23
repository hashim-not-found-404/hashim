use crate::domain::request_response::ResourceDTO;
use crate::domain::utility::uuid::Branch;
use crate::domain::utility::uuid::Company;
use crate::domain::utility::uuid::Nonce;
use crate::domain::utility::uuid::User;
use crate::utility::traits::DynamicError;
use std::collections::HashMap;
use std::collections::HashSet;

pub(crate) type ListOfResources = HashMap<Branch, Vec<ResourceDTO>>;

#[derive(Debug, Default)]
pub(crate) struct SideEffects {
    pub(crate) authenticated_users:              HashSet<User>,
    pub(crate) users_to_resubscribe:             HashSet<User>,
    pub(crate) resource_to_broadcast_for_branch: ListOfResources,
}

pub struct TheCompaniesAndBranchesHeIn {
    pub branches_of_each_company: HashMap<Company, HashSet<Branch>>,
    pub companies:                HashMap<User, HashSet<Company>>,
    pub branches:                 HashMap<User, HashSet<Branch>>,
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
    ) -> impl Future<Output = Result<Result<(), domain_errors::AtCommit>, DynamicError>>;
    fn rollback_transaction(self) -> impl Future<Output = Result<(), DynamicError>>;
}

pub trait DatabaseWrite {
    type Db<'a>: DBTransaction;
    type Input;

    fn write(
        txn: &mut Self::Db<'_>,
        input: &Self::Input,
    ) -> impl Future<Output = Result<(), DynamicError>>;
}

pub trait DBClient {
    type Txn<'a>: DBTransaction
    where
        Self: 'a;

    fn begin_transaction(&mut self) -> impl Future<Output = Result<Self::Txn<'_>, DynamicError>>;

    fn write_nonce_if_not_used_and_return_is_nonce_used(
        &mut self,
        nonce: &Nonce,
    ) -> impl Future<Output = Result<bool, DynamicError>>;

    // here we just do read we dont do here any set or check

    fn read_roles_for_user(
        &mut self,
        users_uuids: &HashSet<User>,
    ) -> impl Future<Output = Result<TheCompaniesAndBranchesHeIn, DynamicError>>;
}
