use crate::{
    accounting_domain::{
        cases::{self, MyErrorTrait},
        types,
    },
    server::{
        server_traits::{DBClient, DBTransaction},
        server_types,
    },
    utility::{traits, utils},
};

pub(crate) trait ServerOperations {
    type Ok;
    type Error;

    async fn handle_operation<
        Rn: traits::RandomNumber,
        Id: cases::RowId,
        Rg: traits::Regex,
        Auth: cases::HashedPassword,
        Jwt: cases::JWT,
        Cli: DBClient,
    >(
        &self,
        side_effects: &mut server_types::SideEffects,
        client: &mut Cli,
        jwt: &Jwt,
    ) -> Result<Result<Self::Ok, Self::Error>, utils::DynamicError>;
}

impl ServerOperations for cases::sign_up::Input {
    type Ok = cases::sign_up::Ok;
    type Error = cases::sign_up::Error;

    async fn handle_operation<
        Rn: traits::RandomNumber,
        Id: cases::RowId,
        Rg: traits::Regex,
        Auth: cases::HashedPassword,
        Jwt: cases::JWT,
        Cli: DBClient,
    >(
        &self,
        side_effects: &mut server_types::SideEffects,
        client: &mut Cli,
        jwt: &Jwt,
    ) -> Result<Result<Self::Ok, Self::Error>, utils::DynamicError> {
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

impl ServerOperations for cases::sign_in::Input {
    type Ok = cases::sign_in::Ok;
    type Error = cases::sign_in::Error;

    async fn handle_operation<
        Rn: traits::RandomNumber,
        Id: cases::RowId,
        Rg: traits::Regex,
        Auth: cases::HashedPassword,
        Jwt: cases::JWT,
        Cli: DBClient,
    >(
        &self,
        side_effects: &mut server_types::SideEffects,
        client: &mut Cli,
        jwt: &Jwt,
    ) -> Result<Result<Self::Ok, Self::Error>, utils::DynamicError> {
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

impl ServerOperations for cases::create_company::Input {
    type Ok = cases::create_company::Ok;
    type Error = cases::create_company::Error;

    async fn handle_operation<
        Rn: traits::RandomNumber,
        Id: cases::RowId,
        Rg: traits::Regex,
        Auth: cases::HashedPassword,
        Jwt: cases::JWT,
        Cli: DBClient,
    >(
        &self,
        side_effects: &mut server_types::SideEffects,
        client: &mut Cli,
        jwt: &Jwt,
    ) -> Result<Result<Self::Ok, Self::Error>, utils::DynamicError> {
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

impl ServerOperations for cases::list_company_and_branch::Input {
    type Ok = cases::list_company_and_branch::Ok;
    type Error = cases::list_company_and_branch::Error;

    async fn handle_operation<
        Rn: traits::RandomNumber,
        Id: cases::RowId,
        Rg: traits::Regex,
        Auth: cases::HashedPassword,
        Jwt: cases::JWT,
        Cli: DBClient,
    >(
        &self,
        side_effects: &mut server_types::SideEffects,
        client: &mut Cli,
        _jwt: &Jwt,
    ) -> Result<Result<Self::Ok, Self::Error>, utils::DynamicError> {
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

impl ServerOperations for cases::create_company_branch::Input {
    type Ok = cases::create_company_branch::Ok;
    type Error = cases::create_company_branch::Error;

    async fn handle_operation<
        Rn: traits::RandomNumber,
        Id: cases::RowId,
        Rg: traits::Regex,
        Auth: cases::HashedPassword,
        Jwt: cases::JWT,
        Cli: DBClient,
    >(
        &self,
        side_effects: &mut server_types::SideEffects,
        client: &mut Cli,
        _jwt: &Jwt,
    ) -> Result<Result<Self::Ok, Self::Error>, utils::DynamicError> {
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
