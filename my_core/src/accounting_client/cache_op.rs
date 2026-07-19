use crate::{
    accounting_client::use_cases::client_domain::{
        cache,
        client_traits::{CacheAndServerType1, CacheAndServerType2},
    },
    accounting_domain::{
        cases::utility::{resource_utils, types},
        request_response,
    },
};

impl<Ch: cache::Cache> cache::State<Ch> {
    pub(crate) async fn new<Id: types::RowId>() -> Self {
        let cache = Ch::new().await;
        let txns = cache.get_all_txn_input().await;

        let mut state = Self {
            state_of_pending_txn: resource_utils::StateOfPendingTxn::default(),
            cache,
        };

        for op in txns {
            op.operation
                .run_operation_check_apply::<Id, Ch>(&mut state)
                .await;
        }

        state
    }
}

impl request_response::push_data::OperationsInput {
    pub(crate) async fn run_operation_check<Id: types::RowId, Ch: cache::Cache>(
        &self,
        state: &mut cache::State<Ch>,
    ) -> request_response::push_data::OperationsResult {
        match self {
            request_response::push_data::OperationsInput::SignUp(i) => {
                operation_check_handler::<_, Id, Ch>(i, state).await
            }
            request_response::push_data::OperationsInput::SignIn(i) => {
                operation_check_handler::<_, Id, Ch>(i, state).await
            }
            request_response::push_data::OperationsInput::CreateCompany(i) => {
                operation_check_handler::<_, Id, Ch>(i, state).await
            }
            request_response::push_data::OperationsInput::CreateCompanyBranch(i) => {
                operation_check_handler::<_, Id, Ch>(i, state).await
            }
            request_response::push_data::OperationsInput::ListCompanyAndBranch(i) => {
                operation_check_handler::<_, Id, Ch>(i, state).await
            }
            request_response::push_data::OperationsInput::CreateAccount(i) => {
                operation_check_handler::<_, Id, Ch>(i, state).await
            }
        }
    }

    pub(crate) async fn run_operation_check_apply<Id: types::RowId, Ch: cache::Cache>(
        &self,
        state: &mut cache::State<Ch>,
    ) {
        match self {
            request_response::push_data::OperationsInput::SignUp(i) => {
                operation_check_apply_handler::<_, Id, Ch>(i, state).await
            }
            request_response::push_data::OperationsInput::SignIn(i) => {
                operation_check_apply_handler::<_, Id, Ch>(i, state).await
            }
            request_response::push_data::OperationsInput::CreateCompany(i) => {
                operation_check_apply_handler::<_, Id, Ch>(i, state).await
            }
            request_response::push_data::OperationsInput::CreateCompanyBranch(i) => {
                operation_check_apply_handler::<_, Id, Ch>(i, state).await
            }
            request_response::push_data::OperationsInput::ListCompanyAndBranch(i) => {
                operation_check_apply_handler::<_, Id, Ch>(i, state).await
            }
            request_response::push_data::OperationsInput::CreateAccount(i) => {
                operation_check_apply_handler::<_, Id, Ch>(i, state).await
            }
        }
    }

    pub(crate) fn get_user_uuid(&self) -> Option<&types::UuidType> {
        match self {
            request_response::push_data::OperationsInput::SignUp(i) => i.user_uuid(),
            request_response::push_data::OperationsInput::SignIn(i) => i.user_uuid(),
            request_response::push_data::OperationsInput::CreateCompany(i) => i.user_uuid(),
            request_response::push_data::OperationsInput::CreateCompanyBranch(i) => i.user_uuid(),
            request_response::push_data::OperationsInput::ListCompanyAndBranch(i) => i.user_uuid(),
            request_response::push_data::OperationsInput::CreateAccount(i) => i.user_uuid(),
        }
    }
}

impl request_response::push_data::OperationsResult {
    pub(crate) fn extract_resource(&self) -> Vec<resource_utils::ResourceInfo> {
        match self {
            request_response::push_data::OperationsResult::SignUp(i) => i.extract_resource(),
            request_response::push_data::OperationsResult::SignIn(i) => i.extract_resource(),
            request_response::push_data::OperationsResult::CreateCompany(i) => i.extract_resource(),
            request_response::push_data::OperationsResult::CreateCompanyBranch(i) => {
                i.extract_resource()
            }
            request_response::push_data::OperationsResult::ListCompanyAndBranch(i) => {
                i.extract_resource()
            }
            request_response::push_data::OperationsResult::CreateAccount(i) => i.extract_resource(),
        }
    }

    pub(crate) fn is_ok(&self) -> bool {
        match self {
            request_response::push_data::OperationsResult::SignUp(i) => i.is_ok(),
            request_response::push_data::OperationsResult::SignIn(i) => i.is_ok(),
            request_response::push_data::OperationsResult::CreateCompany(i) => i.is_ok(),
            request_response::push_data::OperationsResult::CreateCompanyBranch(i) => i.is_ok(),
            request_response::push_data::OperationsResult::ListCompanyAndBranch(i) => i.is_ok(),
            request_response::push_data::OperationsResult::CreateAccount(i) => i.is_ok(),
        }
    }
}

async fn operation_check_handler<T: CacheAndServerType1, Id: types::RowId, Ch: cache::Cache>(
    input: &T,
    state: &mut cache::State<Ch>,
) -> request_response::push_data::OperationsResult {
    return input
        .state_full_operation::<Id, Ch>(state)
        .await
        .wrap_output();
}

async fn operation_check_apply_handler<
    T: CacheAndServerType1,
    Id: types::RowId,
    Ch: cache::Cache,
>(
    input: &T,
    state: &mut cache::State<Ch>,
) {
    resource_utils::apply_change(
        input
            .state_full_operation::<Id, Ch>(state)
            .await
            .extract_resource(),
        &mut state.state_of_pending_txn,
    );
}
