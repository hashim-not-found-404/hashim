use crate::accounting_domain::request_response::messages::ResourcesDTO;
use crate::accounting_domain::utility::types::Role;
use crate::accounting_domain::utility::types::UuidType;
use crate::utility::traits::DynamicError;
use std::collections::HashMap;
use std::collections::HashSet;

// #[derive(Debug, Clone, Default, Eq, Hash, PartialEq)]
pub type CompanyUuid = UuidType;

// #[derive(Debug, Clone, Default, Eq, Hash, PartialEq)]
pub type BranchUuid = UuidType;

// #[derive(Debug, Clone, Default, Eq, Hash, PartialEq)]
pub type UserUuid = UuidType;

pub(crate) type ListOfResources = HashMap<BranchUuid, Vec<ResourcesDTO>>;

#[derive(Debug, Default)]
pub(crate) struct SideEffects {
    pub(crate) authenticated_users:              HashSet<UserUuid>,
    pub(crate) users_to_resubscribe:             HashSet<UserUuid>,
    pub(crate) resource_to_broadcast_for_branch: ListOfResources,
}

pub struct AllRoles {
    pub companies: HashMap<CompanyUuid, HashMap<UserUuid, Vec<Role>>>,
    pub branches:  HashMap<BranchUuid, HashMap<UserUuid, Vec<Role>>>,
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
    type Txn<'a>: DBTransaction;
    type Input;

    fn write(
        txn: &mut Self::Txn<'_>,
        input: &Self::Input,
    ) -> impl Future<Output = Result<(), DynamicError>>;
}

pub trait DBClient {
    type Txn<'a>: DBTransaction
    where
        Self: 'a;

    fn begin_transaction(&mut self) -> impl Future<Output = Result<Self::Txn<'_>, DynamicError>>;

    fn write_nonce_if_not_used(
        &mut self,
        nonce: &UuidType,
    ) -> impl Future<Output = Result<bool /* is nonce used */, DynamicError>>;

    // here we just do read we dont do here any set or check

    fn read_roles_for_user(
        &mut self,
        users_uuids: &HashSet<UserUuid>,
    ) -> impl Future<Output = Result<AllRoles, DynamicError>>;
}
