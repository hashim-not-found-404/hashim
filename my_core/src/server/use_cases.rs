use crate::{
    accounting_domain::{
        cases::{self, MyErrorTrait},
        types,
    },
    server::server_traits::{DBClient, DBTransaction},
    utility::traits,
};
use std::collections::{HashMap, HashSet};

pub(crate) type ListOfResources = HashMap<types::UuidType, Vec<types::ResourceInfo>>;

#[derive(Default)]
pub(crate) struct SideEffects {
    pub(crate) authenticated_users: HashSet<types::UuidType>,
    pub(crate) users_to_resubscribe: HashSet<types::UuidType>,
    pub(crate) resource_to_broadcast_for_company: ListOfResources,
    pub(crate) resource_to_broadcast_for_branch: ListOfResources,
}

impl cases::sign_up::Input {
    pub(crate) async fn handle_operation<
        Id: cases::RowId,
        Auth: cases::HashedPassword,
        Jwt: cases::JWT,
        Cli: DBClient,
    >(
        &self,
        side_effects: &mut SideEffects,
        client: &mut Cli,
        jwt: &Jwt,
    ) -> Result<Result<cases::sign_up::Ok, cases::sign_up::Error>, traits::DynamicError> {
        let errr = self.state_less_check::<Id>();
        if errr.is_there_error() {
            return Ok(Err(errr));
        }

        let mut txn = client.begin_transaction().await?;
        let (is_new_uuid, is_user_id) = txn.read_sign_up(&self.new_uuid, &self.user_id).await?;

        let errr = self.state_full_check::<Id>(is_new_uuid, is_user_id);
        if errr.is_there_error() {
            let _ = txn.rollback_transaction().await?;
            return Ok(Err(errr));
        }
        let result = self.state_full_operation::<Auth, Jwt>(jwt);
        txn.write_sign_up(&result).await?;
        let _ = txn.commit_transaction().await?;

        side_effects
            .authenticated_users
            .insert(self.new_uuid.clone());

        return Ok(Ok(result));
    }
}

impl cases::sign_in::Input {
    pub(crate) async fn handle_operation<
        Auth: cases::HashedPassword,
        Jwt: cases::JWT,
        Cli: DBClient,
    >(
        &self,
        side_effects: &mut SideEffects,
        client: &mut Cli,
        jwt: &Jwt,
    ) -> Result<Result<cases::sign_in::Ok, cases::sign_in::Error>, traits::DynamicError> {
        let user_rowid_and_password_hash_and_name = client.read_sign_in(&self.user_id).await?;
        let result =
            self.state_full_check::<Auth, Jwt>(jwt, &user_rowid_and_password_hash_and_name);

        if let Ok(ok) = &result {
            side_effects
                .authenticated_users
                .insert(ok.user_uuid.clone());
            side_effects
                .users_to_resubscribe
                .insert(ok.user_uuid.clone());
        }

        return Ok(result);
    }
}

impl cases::create_company::Input {
    pub(crate) async fn handle_operation<Id: cases::RowId, Cli: DBClient>(
        &self,
        side_effects: &mut SideEffects,
        client: &mut Cli,
    ) -> Result<Result<cases::create_company::Ok, cases::create_company::Error>, traits::DynamicError>
    {
        let mut errr = self.state_less_check::<Id>();
        if !side_effects.authenticated_users.contains(&self.user_uuid) {
            errr.user_uuid = Some(types::UserUuidError::NotAuthenticated);
        }
        if errr.is_there_error() {
            return Ok(Err(errr));
        }

        let mut txn = client.begin_transaction().await?;
        let is_new_uuid_exist = txn.read_create_company(&self.new_uuid).await?;
        let errr = self.state_full_check::<Id>(is_new_uuid_exist);
        if errr.is_there_error() {
            let _ = txn.rollback_transaction().await?;
            return Ok(Err(errr));
        }

        let result = self.state_less_operation();
        txn.write_create_company(&result).await?;
        let _ = txn.commit_transaction().await?;

        return Ok(Ok(result));
    }
}

impl cases::list_company_and_branch::Input {
    pub(crate) async fn handle_operation<Id: cases::RowId, Cli: DBClient>(
        &self,
        side_effects: &mut SideEffects,
        client: &mut Cli,
    ) -> Result<
        Result<cases::list_company_and_branch::Ok, cases::list_company_and_branch::Error>,
        traits::DynamicError,
    > {
        let mut errr = self.state_less_check::<Id>();
        if !side_effects.authenticated_users.contains(&self.user_uuid) {
            errr.user_uuid = Some(types::UserUuidError::NotAuthenticated);
        }
        if errr.is_there_error() {
            return Ok(Err(errr));
        }

        let result = client.read_list_company_and_branch(&self.user_uuid).await?;

        return Ok(Ok(cases::list_company_and_branch::Ok {
            user_uuid: self.user_uuid.clone(),
            data: result,
        }));
    }
}

impl cases::create_company_branch::Input {
    pub(crate) async fn handle_operation<Id: cases::RowId, Cli: DBClient>(
        &self,
        side_effects: &mut SideEffects,
        client: &mut Cli,
    ) -> Result<
        Result<cases::create_company_branch::Ok, cases::create_company_branch::Error>,
        traits::DynamicError,
    > {
        let mut errr = self.state_less_check::<Id>();
        if !side_effects.authenticated_users.contains(&self.user_uuid) {
            errr.user_uuid = Some(types::UserUuidError::NotAuthenticated);
        }
        if errr.is_there_error() {
            return Ok(Err(errr));
        }

        let mut txn = client.begin_transaction().await?;
        let (user_roles, is_new_uuid_used, is_company_belong_exist, is_branch_name_used) = txn
            .read_create_company_branch(
                &self.new_uuid,
                &self.user_uuid,
                &self.company_belong,
                &self.branch_name,
            )
            .await?;

        let errr = self.state_full_check::<Id>(
            &user_roles,
            is_new_uuid_used,
            is_company_belong_exist,
            is_branch_name_used,
        );
        if errr.is_there_error() {
            let _ = txn.rollback_transaction().await?;
            return Ok(Err(errr));
        }

        let result = self.state_less_operation();
        txn.write_create_company_branch(&result).await?;
        let _ = txn.commit_transaction().await?;

        return Ok(Ok(result));
    }
}
