use crate::domain::request_response::ResourceDTO;
use crate::domain::utility::new_types::BranchUuid;
use crate::domain::utility::new_types::CompanyUuid;
use crate::domain::utility::new_types::NonceUuid;
use crate::domain::utility::new_types::UserUuid;
use crate::utility::traits::DynamicError;
use std::collections::HashMap;
use std::collections::HashSet;

pub(crate) type ListOfResources = HashMap<BranchUuid, Vec<ResourceDTO>>;

#[derive(Debug, Default)]
pub(crate) struct SideEffects {
    pub(crate) authenticated_users:              HashSet<UserUuid>,
    pub(crate) users_to_resubscribe:             HashSet<UserUuid>,
    pub(crate) resource_to_broadcast_for_branch: ListOfResources,
}

pub struct TheCompaniesAndBranchesHeIn {
    pub branches_of_each_company: HashMap<CompanyUuid, HashSet<BranchUuid>>,
    pub companies:                HashMap<UserUuid, HashSet<CompanyUuid>>,
    pub branches:                 HashMap<UserUuid, HashSet<BranchUuid>>,
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

pub trait DBClient {
    type Txn<'a>: DBTransaction
    where
        Self: 'a;

    fn begin_transaction(&mut self) -> impl Future<Output = Result<Self::Txn<'_>, DynamicError>>;

    fn write_nonce_if_not_used_and_return_is_nonce_used(
        &mut self,
        nonce: &NonceUuid,
    ) -> impl Future<Output = Result<bool, DynamicError>>;

    // here we just do read we dont do here any set or check

    fn read_roles_for_user(
        &mut self,
        users_uuids: &HashSet<UserUuid>,
    ) -> impl Future<Output = Result<TheCompaniesAndBranchesHeIn, DynamicError>>;
}

#[macro_export]
macro_rules! make_auth_check {
    ($side_effects:expr, $self:expr, $errr:expr) => {
        if !$side_effects.authenticated_users.contains(&$self.user_uuid) {
            $errr.user_uuid = Some(UserUuidError::NotAuthenticated);
        }
    };
}
