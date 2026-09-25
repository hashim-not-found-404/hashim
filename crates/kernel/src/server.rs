use crate::new_types::BranchUuid;
use crate::new_types::CompanyUuid;
use crate::new_types::NonceUuid;
use crate::new_types::UserUuid;
use anyhow::Result;
use infrastructure::jwt::JWT;
use std::collections::HashMap;
use std::collections::HashSet;
use std::pin::Pin;
use utility::dtos::TypeOperationDTOError;
use utility::dtos::TypeOperationDTOInput;
use utility::dtos::TypeOperationDTOOk;
use utility::dtos::TypeOperationDTOResource;

pub struct TheCompaniesAndBranchesHeIn {
    pub branches_of_each_company: HashMap<CompanyUuid, HashSet<BranchUuid>>,
    pub companies: HashMap<UserUuid, HashSet<CompanyUuid>>,
    pub branches: HashMap<UserUuid, HashSet<BranchUuid>>,
}

pub enum AtCommit {
    DataIsChanged,
}

pub trait DBTransaction {
    fn commit_transaction(self) -> impl Future<Output = Result<Result<(), AtCommit>>>;
    fn rollback_transaction(self) -> impl Future<Output = Result<()>>;
}

pub trait DBClient {
    type Txn<'a>: DBTransaction
    where
        Self: 'a;

    fn begin_transaction(&mut self) -> impl Future<Output = Result<Self::Txn<'_>>>;

    fn write_nonce_if_not_used_and_return_is_nonce_used(
        &mut self,
        nonce: &NonceUuid,
    ) -> impl Future<Output = Result<bool>>;

    // here we just do read we dont do here any set or check

    fn read_roles_for_user(
        &mut self,
        users_uuids: &HashSet<UserUuid>,
    ) -> impl Future<Output = Result<TheCompaniesAndBranchesHeIn>>;
}

pub trait Database: Sized + 'static {
    type Client: DBClient;
    fn new() -> impl Future<Output = Result<Self>>;
    fn get_client(&self) -> impl Future<Output = Result<Self::Client>>;
}

pub(crate) type ListOfResources = HashMap<BranchUuid, Vec<TypeOperationDTOResource>>;

#[derive(Debug, Default)]
pub struct SideEffects {
    pub authenticated_users: HashSet<UserUuid>,
    pub users_to_resubscribe: HashSet<UserUuid>,
    pub resource_to_broadcast_for_branch: ListOfResources,
}

#[macro_export]
macro_rules! make_auth_check {
    ($side_effects:expr, $self:expr, $errr:expr) => {
        if !$side_effects.authenticated_users.contains(&$self.user_uuid) {
            $errr.user_uuid = Some(UserUuidError::NotAuthenticated);
        }
    };
}

pub enum WSMessage {
    Binary(Vec<u8>),
    Close,
}

pub trait WSServer: 'static {
    fn send_bin(&mut self, bin: Vec<u8>) -> impl Future<Output = Result<()>>;
    fn receive(&mut self) -> impl Future<Output = Result<WSMessage>>;
    fn close(self) -> impl Future<Output = Result<()>>;
}

pub trait TraitOperationServerInput {
    type Cli: DBClient;
    type Jwt: JWT;

    fn handle_operation<'a>(
        self: Box<Self>,
        side_effects: &'a mut SideEffects,
        client: &'a mut Self::Cli,
        jwt: &'a Self::Jwt,
    ) -> Pin<Box<dyn Future<Output = Result<Result<TypeOperationDTOOk, TypeOperationDTOError>>> + 'a>>;
}

pub trait CastDTOToServer {
    type Cli: DBClient;
    type Jwt: JWT;

    fn cast_input(
        v: TypeOperationDTOInput,
    ) -> Box<dyn TraitOperationServerInput<Cli = Self::Cli, Jwt = Self::Jwt>>;
}
