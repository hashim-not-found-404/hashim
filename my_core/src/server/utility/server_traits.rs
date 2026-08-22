use crate::accounting_domain::utility::resource_utils;
use crate::accounting_domain::utility::types;
use crate::utility::traits::DynamicError;
use std::collections::HashMap;
use std::collections::HashSet;

pub(crate) type ListOfResources = HashMap<types::UuidType, Vec<resource_utils::ResourceInfo>>;

#[derive(Debug, Default)]
pub(crate) struct SideEffects {
    pub(crate) authenticated_users:               HashSet<types::UuidType>,
    pub(crate) users_to_resubscribe:              HashSet<types::UuidType>,
    pub(crate) resource_to_broadcast_for_company: ListOfResources,
    pub(crate) resource_to_broadcast_for_branch:  ListOfResources,
}

pub struct AllRoles {
    pub companies: HashMap<
        types::UuidType, // company uuid
        HashMap<
            types::UuidType, // user uuid
            Vec<types::Role>,
        >,
    >,
    pub branches: HashMap<
        types::UuidType, // branch uuid
        HashMap<
            types::UuidType, // user uuid
            Vec<types::Role>,
        >,
    >,
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

    fn write_nonce_if_not_used(
        &mut self,
        nonce: &types::UuidType,
    ) -> impl Future<Output = Result<bool /* is nonce used */, DynamicError>>;

    // here we just do read we dont do here any set or check

    fn read_roles_for_user(
        &mut self,
        users_uuids: &HashSet<types::UuidType>,
    ) -> impl Future<Output = Result<AllRoles, DynamicError>>;
}
