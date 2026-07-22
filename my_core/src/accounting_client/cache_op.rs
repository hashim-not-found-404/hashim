use crate::accounting_client::client_domain::cache;
use crate::accounting_client::client_domain::client_traits::ViewAndCache;
use crate::accounting_client::use_cases;
use crate::accounting_domain::cases;
use crate::accounting_domain::request_response;
use crate::accounting_domain::utility::resource_utils;
use crate::accounting_domain::utility::resource_utils::apply_change;
use crate::accounting_domain::utility::types;

pub trait DbBundle<Ch: cache::Cache>: 'static {
    type CreateAccount: for<'a> cases::create_account::DatabaseRead<Db<'a> = Ch>;
    type CreateCompany: for<'a> cases::create_company::DatabaseRead<Db<'a> = Ch>;
    type CreateCompanyBranch: for<'a> cases::create_company_branch::DatabaseRead<Db<'a> = Ch>;
    type ListCompanyAndBranch: for<'a> cases::list_company_and_branch::DatabaseRead<Db<'a> = Ch>
        + 'static;
    type SignIn: for<'a> cases::sign_in::DatabaseRead<Db<'a> = Ch>;
    type SignUp: for<'a> cases::sign_up::DatabaseRead<Db<'a> = Ch>;
}

impl<Ch: cache::Cache> cache::State<Ch> {
    pub(crate) async fn new<Id: types::RowId, Dbb: DbBundle<Ch>>() -> Self {
        let cache = Ch::new().await;
        let txns = cache.get_all_txn_input().await;

        let mut state = Self {
            state_of_pending_txn: resource_utils::StateOfPendingTxn::default(),
            cache,
        };

        for op in txns {
            op.operation
                .run_operation_check_apply::<Id, Ch, Dbb>(&mut state)
                .await;
        }

        state
    }
}

macro_rules! run_operation_check {
    ($path:ident, $name:ident, $db:ty, $i:expr, $state:expr) => {
        request_response::push_data::OperationsResult::$name(
            <use_cases::$path::ViewAndCacheType as ViewAndCache<Ch, $db>>::state_full_operation::<
                Id,
            >($i, $state)
            .await,
        )
    };
}

macro_rules! run_operation_check_apply {
    ($path:ident, $db:ty, $i:expr, $state:expr) => {
        let a= <use_cases::$path::ViewAndCacheType as ViewAndCache<Ch,$db>>::state_full_operation::<Id>($i, $state).await;
        let resources=<use_cases::$path::ViewAndCacheType as ViewAndCache<Ch,$db>>::extract_resource(&a);
        apply_change(resources,&mut $state.state_of_pending_txn);
    };
}

macro_rules! get_user_uuid {
    ($path:ident, $db:ty, $i:expr) => {
        <use_cases::$path::ViewAndCacheType as ViewAndCache<Ch, $db>>::user_uuid(&$i)
    };
}

impl request_response::push_data::OperationsInput {
    pub(crate) async fn run_operation_check<
        Id: types::RowId,
        Ch: cache::Cache,
        Dbb: DbBundle<Ch>,
    >(
        &self,
        state: &mut cache::State<Ch>,
    ) -> request_response::push_data::OperationsResult {
        match self {
            request_response::push_data::OperationsInput::SignUp(i) => {
                run_operation_check!(sign_up, SignUp, Dbb::SignUp, i, state)
            }
            request_response::push_data::OperationsInput::SignIn(i) => {
                run_operation_check!(sign_in, SignIn, Dbb::SignIn, i, state)
            }
            request_response::push_data::OperationsInput::CreateCompany(i) => {
                run_operation_check!(create_company, CreateCompany, Dbb::CreateCompany, i, state)
            }
            request_response::push_data::OperationsInput::CreateCompanyBranch(i) => {
                run_operation_check!(
                    create_company_branch,
                    CreateCompanyBranch,
                    Dbb::CreateCompanyBranch,
                    i,
                    state
                )
            }
            request_response::push_data::OperationsInput::ListCompanyAndBranch(i) => {
                run_operation_check!(
                    list_company_and_branch,
                    ListCompanyAndBranch,
                    Dbb::ListCompanyAndBranch,
                    i,
                    state
                )
            }
            request_response::push_data::OperationsInput::CreateAccount(i) => {
                run_operation_check!(create_account, CreateAccount, Dbb::CreateAccount, i, state)
            }
        }
    }

    pub(crate) async fn run_operation_check_apply<
        Id: types::RowId,
        Ch: cache::Cache,
        Dbb: DbBundle<Ch>,
    >(
        &self,
        state: &mut cache::State<Ch>,
    ) {
        match self {
            request_response::push_data::OperationsInput::SignUp(i) => {
                run_operation_check_apply!(sign_up, Dbb::SignUp, i, state);
            }
            request_response::push_data::OperationsInput::SignIn(i) => {
                run_operation_check_apply!(sign_in, Dbb::SignIn, i, state);
            }
            request_response::push_data::OperationsInput::CreateCompany(i) => {
                run_operation_check_apply!(create_company, Dbb::CreateCompany, i, state);
            }
            request_response::push_data::OperationsInput::CreateCompanyBranch(i) => {
                run_operation_check_apply!(
                    create_company_branch,
                    Dbb::CreateCompanyBranch,
                    i,
                    state
                );
            }
            request_response::push_data::OperationsInput::ListCompanyAndBranch(i) => {
                run_operation_check_apply!(
                    list_company_and_branch,
                    Dbb::ListCompanyAndBranch,
                    i,
                    state
                );
            }
            request_response::push_data::OperationsInput::CreateAccount(i) => {
                run_operation_check_apply!(create_account, Dbb::CreateAccount, i, state);
            }
        }
    }

    pub(crate) fn get_user_uuid<Ch: cache::Cache, Dbb: DbBundle<Ch>>(
        &self,
    ) -> Option<&types::UuidType> {
        match self {
            request_response::push_data::OperationsInput::SignUp(i) => {
                get_user_uuid!(sign_up, Dbb::SignUp, i)
            }
            request_response::push_data::OperationsInput::SignIn(i) => {
                get_user_uuid!(sign_in, Dbb::SignIn, i)
            }
            request_response::push_data::OperationsInput::CreateCompany(i) => {
                get_user_uuid!(create_company, Dbb::CreateCompany, i)
            }
            request_response::push_data::OperationsInput::CreateCompanyBranch(i) => {
                get_user_uuid!(create_company_branch, Dbb::CreateCompanyBranch, i)
            }
            request_response::push_data::OperationsInput::ListCompanyAndBranch(i) => {
                get_user_uuid!(list_company_and_branch, Dbb::ListCompanyAndBranch, i)
            }
            request_response::push_data::OperationsInput::CreateAccount(i) => {
                get_user_uuid!(create_account, Dbb::CreateAccount, i)
            }
        }
    }
}

macro_rules! extract_resource {
    ($path:ident, $db:ty, $i:expr) => {
        <use_cases::$path::ViewAndCacheType as ViewAndCache<Ch, $db>>::extract_resource($i)
    };
}

impl request_response::push_data::OperationsResult {
    pub(crate) fn extract_resource<Ch: cache::Cache, Dbb: DbBundle<Ch>>(
        &self,
    ) -> Vec<resource_utils::ResourceInfo> {
        match self {
            request_response::push_data::OperationsResult::SignIn(i) => {
                extract_resource!(sign_in, Dbb::SignIn, i)
            }
            request_response::push_data::OperationsResult::SignUp(i) => {
                extract_resource!(sign_up, Dbb::SignUp, i)
            }
            request_response::push_data::OperationsResult::CreateCompany(i) => {
                extract_resource!(create_company, Dbb::CreateCompany, i)
            }
            request_response::push_data::OperationsResult::CreateCompanyBranch(i) => {
                extract_resource!(create_company_branch, Dbb::CreateCompanyBranch, i)
            }
            request_response::push_data::OperationsResult::ListCompanyAndBranch(i) => {
                extract_resource!(list_company_and_branch, Dbb::ListCompanyAndBranch, i)
            }
            request_response::push_data::OperationsResult::CreateAccount(i) => {
                extract_resource!(create_account, Dbb::CreateAccount, i)
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
            request_response::push_data::OperationsResult::CreateAccount(i) => i.is_ok(),
        }
    }
}
