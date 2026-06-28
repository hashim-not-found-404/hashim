use crate::decider::EventMaker;
use crate::decider::StateOp;
use crate::prelude::*;
use std::result::Result as StdResult;

enum ServerState<'a, 'b, Cli: DBClient> {
    Client(&'a mut Cli),
    Txn(&'b mut <Cli as DBClient>::Txn<'a>),
}

impl<'a, 'b, Cli: DBClient> ServerState<'a, 'b, Cli> {
    fn set_client(client: &'a mut Cli) -> Self {
        Self::Client(client)
    }

    fn set_txn(txn: &'b mut <Cli as DBClient>::Txn<'a>) -> Self {
        Self::Txn(txn)
    }

    // fn get_client(&'a mut self) -> &'a mut Cli {
    //     match self {
    //         ServerState::Client(client) => client,
    //         ServerState::Txn(_) => unreachable!(),
    //     }
    // }

    // fn get_txn(&'a mut self) -> &'a mut <Cli as DBClient>::Txn<'a> {
    //     match self {
    //         ServerState::Client(_) => unreachable!(),
    //         ServerState::Txn(txn) => txn,
    //     }
    // }
}

impl<'a, 'b, Cli: DBClient> StateOp for ServerState<'a, 'b, Cli> {
    async fn read_sign_up(
        &mut self,
        new_uuid: &db_types::UuidType,
        user_id: &String,
    ) -> StdResult<
        (
            bool, /* is new_uuid exist */
            bool, /* is user_id exist */
        ),
        DynamicError,
    > {
        match self {
            ServerState::Client(_) => unreachable!(),
            ServerState::Txn(txn) => txn.read_sign_up(new_uuid, user_id).await,
        }
    }

    async fn read_sign_in(
        &mut self,
        user_id: &String,
    ) -> StdResult<Option<(db_types::UuidType, String)>, DynamicError> {
        match self {
            ServerState::Client(client) => client.read_sign_in(user_id).await,
            ServerState::Txn(_) => unreachable!(),
        }
    }

    async fn read_create_company(
        &mut self,
        new_uuid: &db_types::UuidType,
    ) -> StdResult<bool /* is new_uuid exist */, DynamicError> {
        match self {
            ServerState::Client(_) => unreachable!(),
            ServerState::Txn(txn) => txn.read_create_company(new_uuid).await,
        }
    }

    async fn read_list_company_and_branch(
        &mut self,
        user_uuid: &db_types::UuidType,
    ) -> StdResult<Vec<ResourceInfo>, DynamicError> {
        match self {
            ServerState::Client(client) => client.read_list_company_and_branch(user_uuid).await,
            ServerState::Txn(_) => unreachable!(),
        }
    }

    async fn read_create_company_branch(
        &mut self,
        new_uuid: &db_types::UuidType,
        user_uuid: &db_types::UuidType,
        company_belong: &db_types::UuidType,
        branch_name: &String,
    ) -> StdResult<
        (
            Vec<db_types::Role>, /* user roles */
            bool,                /* is new_uuid exist */
            bool,                /* is company_belong exist */
            bool,                /* is branch_name used */
        ),
        DynamicError,
    > {
        match self {
            ServerState::Client(_) => unreachable!(),
            ServerState::Txn(txn) => {
                txn.read_create_company_branch(new_uuid, user_uuid, company_belong, branch_name)
                    .await
            }
        }
    }
}

pub(crate) trait ServerOperations {
    type Ok;
    type Error;

    async fn handle_operation<At: AllServerTypes>(
        &self,
        side_effects: &mut server_methods::SideEffects,
        client: &mut At::Cli,
        jwt: &At::Jwt,
    ) -> Result<Result<Self::Ok, Self::Error>, DynamicError>;
}

impl ServerOperations for decider::sign_up::Input {
    type Ok = decider::sign_up::Ok;
    type Error = decider::sign_up::Error;

    async fn handle_operation<At: AllServerTypes>(
        &self,
        side_effects: &mut server_methods::SideEffects,
        client: &mut At::Cli,
        jwt: &At::Jwt,
    ) -> StdResult<StdResult<Self::Ok, Self::Error>, DynamicError> {
        let mut txn = client.begin_transaction().await?;
        let mut state = ServerState::<At::Cli>::set_txn(&mut txn);

        let result = self
                .handle::<ServerState<_>, At::Rn, At::Rt, At::Id, At::Mpsc, At::Ed, At::Rg, At::Auth, At::Jwt>(
                    side_effects,
                    &mut state,
                    jwt,
                )
                .await;

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

    async fn handle_operation<At: AllServerTypes>(
        &self,
        side_effects: &mut server_methods::SideEffects,
        client: &mut At::Cli,
        jwt: &At::Jwt,
    ) -> StdResult<StdResult<Self::Ok, Self::Error>, DynamicError> {
        let mut state = ServerState::<At::Cli>::set_client(client);

        let result = self
                .handle::<ServerState<_>, At::Rn, At::Rt, At::Id, At::Mpsc, At::Ed, At::Rg, At::Auth, At::Jwt>(
                    side_effects,
                    &mut state,
                    jwt,
                )
                .await;

        return result;
    }
}

impl ServerOperations for decider::create_company::Input {
    type Ok = decider::create_company::Ok;
    type Error = decider::create_company::Error;

    async fn handle_operation<At: AllServerTypes>(
        &self,
        side_effects: &mut server_methods::SideEffects,
        client: &mut At::Cli,
        jwt: &At::Jwt,
    ) -> StdResult<StdResult<Self::Ok, Self::Error>, DynamicError> {
        let mut txn = client.begin_transaction().await?;
        let mut state = ServerState::<At::Cli>::set_txn(&mut txn);

        let result = self
                .handle::<ServerState<_>, At::Rn, At::Rt, At::Id, At::Mpsc, At::Ed, At::Rg, At::Auth, At::Jwt>(
                    side_effects,
                    &mut state,
                    jwt,
                )
                .await;

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

    async fn handle_operation<At: AllServerTypes>(
        &self,
        side_effects: &mut server_methods::SideEffects,
        client: &mut At::Cli,
        jwt: &At::Jwt,
    ) -> StdResult<StdResult<Self::Ok, Self::Error>, DynamicError> {
        let mut state = ServerState::<At::Cli>::set_client(client);

        let result = self
                .handle::<ServerState<_>, At::Rn, At::Rt, At::Id, At::Mpsc, At::Ed, At::Rg, At::Auth, At::Jwt>(
                    side_effects,
                    &mut state,
                    jwt,
                )
                .await;

        return result;
    }
}

impl ServerOperations for decider::create_company_branch::Input {
    type Ok = decider::create_company_branch::Ok;
    type Error = decider::create_company_branch::Error;

    async fn handle_operation<At: AllServerTypes>(
        &self,
        side_effects: &mut server_methods::SideEffects,
        client: &mut At::Cli,
        jwt: &At::Jwt,
    ) -> StdResult<StdResult<Self::Ok, Self::Error>, DynamicError> {
        let mut txn = client.begin_transaction().await?;
        let mut state = ServerState::<At::Cli>::set_txn(&mut txn);

        let result = self
                .handle::<ServerState<_>, At::Rn, At::Rt, At::Id, At::Mpsc, At::Ed, At::Rg, At::Auth, At::Jwt>(
                    side_effects,
                    &mut state,
                    jwt,
                )
                .await;

        if let Ok(Ok(resource)) = &result {
            txn.write_create_company_branch(resource).await?;
            let _ = txn.commit_transaction().await?;
        } else {
            let _ = txn.rollback_transaction().await?;
        }

        return result;
    }
}
