use crate::{
    accounting_domain::{cases, types},
    server::server_types,
    utility::utils,
};
use std::collections::HashSet;

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
        new_uuid: &types::UuidType,
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
        data: &cases::sign_up::Ok,
    ) -> impl Future<Output = Result<(), utils::DynamicError>>;

    fn read_create_company(
        &mut self,
        new_uuid: &types::UuidType,
    ) -> impl Future<Output = Result<bool /* is new_uuid exist */, utils::DynamicError>>;
    fn write_create_company(
        &mut self,
        data: &cases::create_company::Ok,
    ) -> impl Future<Output = Result<(), utils::DynamicError>>;

    fn read_create_company_branch(
        &mut self,
        new_uuid: &types::UuidType,
        user_uuid: &types::UuidType,
        company_belong: &types::UuidType,
        branch_name: &String,
    ) -> impl Future<
        Output = Result<
            (
                Vec<types::Role>, /* user roles */
                bool,             /* is new_uuid exist */
                bool,             /* is company_belong exist */
                bool,             /* is branch_name used */
            ),
            utils::DynamicError,
        >,
    >;
    fn write_create_company_branch(
        &mut self,
        data: &cases::create_company_branch::Ok,
    ) -> impl Future<Output = Result<(), utils::DynamicError>>;
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
        nonce: &types::UuidType,
    ) -> impl Future<Output = Result<bool /* is nonce used */, utils::DynamicError>>;

    // here we just do read we dont do here any set or check

    fn read_sign_in(
        &mut self,
        user_id: &String,
    ) -> impl Future<
        Output = Result<Option<(types::UuidType, String, Option<String>)>, utils::DynamicError>,
    >;
    fn read_roles_for_user(
        &mut self,
        users_uuids: &HashSet<types::UuidType>,
    ) -> impl Future<Output = Result<server_types::AllRoles, utils::DynamicError>>;
    fn read_list_company_and_branch(
        &mut self,
        user_uuid: &types::UuidType,
    ) -> impl Future<
        Output = Result<
            Vec<cases::list_company_and_branch::AllCompaniesThatUserInWithRoles>,
            utils::DynamicError,
        >,
    >;
}
