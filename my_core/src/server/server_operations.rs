use crate::{
    accounting_domain::decider,
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

impl ServerOperations for decider::sign_up::Input {
    type Ok = decider::sign_up::Ok;
    type Error = decider::sign_up::Error;

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

impl ServerOperations for decider::sign_in::Input {
    type Ok = decider::sign_in::Ok;
    type Error = decider::sign_in::Error;

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

impl ServerOperations for decider::create_company::Input {
    type Ok = decider::create_company::Ok;
    type Error = decider::create_company::Error;

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

impl ServerOperations for decider::list_company_and_branch::Input {
    type Ok = decider::list_company_and_branch::Ok;
    type Error = decider::list_company_and_branch::Error;

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

impl ServerOperations for decider::create_company_branch::Input {
    type Ok = decider::create_company_branch::Ok;
    type Error = decider::create_company_branch::Error;

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
