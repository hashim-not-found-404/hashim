use crate::{
    accounting_domain::cases,
    server::{
        server_traits::{self, DBClient, DBTransaction},
        server_types,
    },
    utility::utils,
};

pub(crate) trait ServerOperations {
    type Ok;
    type Error;

    async fn handle_operation<At: server_traits::AllServerTypes>(
        &self,
        side_effects: &mut server_types::SideEffects,
        client: &mut At::Cli,
        jwt: &At::Jwt,
    ) -> Result<Result<Self::Ok, Self::Error>, utils::DynamicError>;
}

impl ServerOperations for cases::sign_up::Input {
    type Ok = cases::sign_up::Ok;
    type Error = cases::sign_up::Error;

    async fn handle_operation<At: server_traits::AllServerTypes>(
        &self,
        side_effects: &mut server_types::SideEffects,
        client: &mut At::Cli,
        jwt: &At::Jwt,
    ) -> Result<Result<Self::Ok, Self::Error>, utils::DynamicError> {
        let mut txn = client.begin_transaction().await?;
        todo!();

        let result = todo!();

        if let Ok(Ok(resource)) = &result {
            txn.write_sign_up(resource).await?;
            let _ = txn.commit_transaction().await?;
        } else {
            let _ = txn.rollback_transaction().await?;
        }

        return result;
    }
}

impl ServerOperations for cases::sign_in::Input {
    type Ok = cases::sign_in::Ok;
    type Error = cases::sign_in::Error;

    async fn handle_operation<At: server_traits::AllServerTypes>(
        &self,
        side_effects: &mut server_types::SideEffects,
        client: &mut At::Cli,
        jwt: &At::Jwt,
    ) -> Result<Result<Self::Ok, Self::Error>, utils::DynamicError> {
        todo!();

        let result = todo!();

        return result;
    }
}

impl ServerOperations for cases::create_company::Input {
    type Ok = cases::create_company::Ok;
    type Error = cases::create_company::Error;

    async fn handle_operation<At: server_traits::AllServerTypes>(
        &self,
        side_effects: &mut server_types::SideEffects,
        client: &mut At::Cli,
        jwt: &At::Jwt,
    ) -> Result<Result<Self::Ok, Self::Error>, utils::DynamicError> {
        let mut txn = client.begin_transaction().await?;
        todo!();

        let result = todo!();

        if let Ok(Ok(resource)) = &result {
            txn.write_create_company(resource).await?;
            let _ = txn.commit_transaction().await?;
        } else {
            let _ = txn.rollback_transaction().await?;
        }

        return result;
    }
}

impl ServerOperations for cases::list_company_and_branch::Input {
    type Ok = cases::list_company_and_branch::Ok;
    type Error = cases::list_company_and_branch::Error;

    async fn handle_operation<At: server_traits::AllServerTypes>(
        &self,
        side_effects: &mut server_types::SideEffects,
        client: &mut At::Cli,
        jwt: &At::Jwt,
    ) -> Result<Result<Self::Ok, Self::Error>, utils::DynamicError> {
        todo!();

        let result = todo!();

        return result;
    }
}

impl ServerOperations for cases::create_company_branch::Input {
    type Ok = cases::create_company_branch::Ok;
    type Error = cases::create_company_branch::Error;

    async fn handle_operation<At: server_traits::AllServerTypes>(
        &self,
        side_effects: &mut server_types::SideEffects,
        client: &mut At::Cli,
        jwt: &At::Jwt,
    ) -> Result<Result<Self::Ok, Self::Error>, utils::DynamicError> {
        let mut txn = client.begin_transaction().await?;
        todo!();

        let result = todo!();

        if let Ok(Ok(resource)) = &result {
            txn.write_create_company_branch(resource).await?;
            let _ = txn.commit_transaction().await?;
        } else {
            let _ = txn.rollback_transaction().await?;
        }

        return result;
    }
}
