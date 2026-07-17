use crate::{
    accounting_client::use_cases::client_domain::{
        cache,
        client_traits::{CacheAndServerType1, CacheAndServerType2},
    },
    accounting_domain::{cases, request_response, types},
    utility::utils::MyUpSert,
};

impl<Ch: cache::Cache> cache::State<Ch> {
    pub(crate) async fn new<Id: cases::RowId>() -> Self {
        let cache = Ch::new().await;
        let txns = cache.get_all_txn_input().await;

        let mut state = Self {
            state_of_pending_txn: cache::tables::StateOfPendingTxn::default(),
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
    pub(crate) async fn run_operation_check<Id: cases::RowId, Ch: cache::Cache>(
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
        }
    }

    pub(crate) async fn run_operation_check_apply<Id: cases::RowId, Ch: cache::Cache>(
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
        }
    }

    pub(crate) async fn run_operation_check_apply_write<Id: cases::RowId, Ch: cache::Cache>(
        &self,
        txn_number: u64,
        state: &mut cache::State<Ch>,
    ) -> request_response::push_data::OperationsResult {
        state
            .cache
            .write_txn_input(&request_response::push_data::Txn {
                txn_number,
                operation: self.clone(),
            })
            .await;

        match self {
            request_response::push_data::OperationsInput::SignUp(i) => {
                operation_check_apply_write_handler::<_, Id, Ch>(i, state).await
            }
            request_response::push_data::OperationsInput::SignIn(i) => {
                operation_check_apply_write_handler::<_, Id, Ch>(i, state).await
            }
            request_response::push_data::OperationsInput::CreateCompany(i) => {
                operation_check_apply_write_handler::<_, Id, Ch>(i, state).await
            }
            request_response::push_data::OperationsInput::CreateCompanyBranch(i) => {
                operation_check_apply_write_handler::<_, Id, Ch>(i, state).await
            }
            request_response::push_data::OperationsInput::ListCompanyAndBranch(i) => {
                operation_check_apply_write_handler::<_, Id, Ch>(i, state).await
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
        }
    }
}

impl request_response::push_data::OperationsResult {
    pub(crate) fn extract_resource(&self) -> Vec<types::ResourceInfo> {
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
        }
    }

    pub(crate) fn is_ok(&self) -> bool {
        match self {
            request_response::push_data::OperationsResult::SignUp(i) => i.is_ok(),
            request_response::push_data::OperationsResult::SignIn(i) => i.is_ok(),
            request_response::push_data::OperationsResult::CreateCompany(i) => i.is_ok(),
            request_response::push_data::OperationsResult::CreateCompanyBranch(i) => i.is_ok(),
            request_response::push_data::OperationsResult::ListCompanyAndBranch(i) => i.is_ok(),
        }
    }
}

async fn operation_check_handler<T: CacheAndServerType1, Id: cases::RowId, Ch: cache::Cache>(
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
    Id: cases::RowId,
    Ch: cache::Cache,
>(
    input: &T,
    state: &mut cache::State<Ch>,
) {
    apply_change(
        input
            .state_full_operation::<Id, Ch>(state)
            .await
            .extract_resource(),
        &mut state.state_of_pending_txn,
    );
}

async fn operation_check_apply_write_handler<
    T: CacheAndServerType1,
    Id: cases::RowId,
    Ch: cache::Cache,
>(
    input: &T,
    state: &mut cache::State<Ch>,
) -> request_response::push_data::OperationsResult {
    let result = input.state_full_operation::<Id, Ch>(state).await;

    apply_change(result.extract_resource(), &mut state.state_of_pending_txn);

    return result.wrap_output();
}

pub(crate) fn apply_change(
    resources: Vec<types::ResourceInfo>,
    state: &mut cache::tables::StateOfPendingTxn,
) {
    for resource in resources {
        let row_uuid = resource.row_uuid;

        match resource.resource {
            types::Resource::Jwt(_) => {}
            types::Resource::TableUserFieldName(r) => {
                state.user.upsert(row_uuid, |table| table.name = Some(r))
            }
            types::Resource::TableUserFieldId(r) => {
                state.user.upsert(row_uuid, |table| table.id = r)
            }
            types::Resource::TableCompanyFieldName(r) => {
                state.company.upsert(row_uuid, |table| table.name = r)
            }
            types::Resource::TableCompanyBranchFieldName(r) => state
                .company_branch
                .upsert(row_uuid, |table| table.name = r),
            types::Resource::TableCompanyBranchFieldCompanyBelong(r) => state
                .company_branch
                .upsert(row_uuid, |table| table.company_belong = r),
            types::Resource::TableCompanyBranchFieldCurrency(r) => state
                .company_branch
                .upsert(row_uuid, |table| table.currency = r),
            types::Resource::TableCompanyBranchFieldLocation(r) => state
                .company_branch
                .upsert(row_uuid, |table| table.location = r),
            types::Resource::TableCompanyFieldCurrency(r) => {
                state.company.upsert(row_uuid, |table| table.currency = r)
            }
            types::Resource::TableAccessControlForCompanyFieldRole(r) => state
                .access_control_for_company
                .upsert(row_uuid, |table| table.role = r),
            types::Resource::TableAccessControlForCompanyFieldUser(r) => state
                .access_control_for_company
                .upsert(row_uuid, |table| table.user_ = r),
            types::Resource::TableAccessControlForCompanyFieldDataGroup(r) => state
                .access_control_for_company
                .upsert(row_uuid, |table| table.data_group = r),
            types::Resource::TableAccessControlForCompanyBranchFieldRole(r) => state
                .access_control_for_company_branch
                .upsert(row_uuid, |table| table.role = r),
            types::Resource::TableAccessControlForCompanyBranchFieldUser(r) => state
                .access_control_for_company_branch
                .upsert(row_uuid, |table| table.user_ = r),
            types::Resource::TableAccessControlForCompanyBranchFieldDataGroup(r) => state
                .access_control_for_company_branch
                .upsert(row_uuid, |table| table.data_group = r),
        }
    }
}
