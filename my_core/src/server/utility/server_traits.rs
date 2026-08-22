use crate::accounting_domain::utility::resource_utils;
use crate::accounting_domain::utility::types::Role;
use crate::accounting_domain::utility::uuid::Branch;
use crate::accounting_domain::utility::uuid::Company;
use crate::accounting_domain::utility::uuid::Nonce;
use crate::accounting_domain::utility::uuid::User;
use crate::utility::traits::DynamicError;
use std::collections::HashMap;
use std::collections::HashSet;

pub(crate) type ListOfResources = HashMap<Branch, Vec<resource_utils::ResourceInfo>>;

#[derive(Debug, Default)]
pub(crate) struct SideEffects {
    pub(crate) authenticated_users:               HashSet<User>,
    pub(crate) users_to_resubscribe:              HashSet<User>,
    pub(crate) resource_to_broadcast_for_company:
        HashMap<Company, Vec<resource_utils::ResourceInfo>>,
    pub(crate) resource_to_broadcast_for_branch:  ListOfResources,
}

pub struct AllRoles {
    pub companies: HashMap<Company, HashMap<User, Vec<Role>>>,
    pub branches:  HashMap<Branch, HashMap<User, Vec<Role>>>,
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
    ) -> impl Future<Output = Result<AllRoles, DynamicError>>;
}
