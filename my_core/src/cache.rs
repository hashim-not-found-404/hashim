use crate::prelude::*;

pub mod tables {
    use crate::db_types;

    #[derive(Default)]
    pub struct User {
        pub name: Option<String>,
        pub id: String,
        pub password: String,
    }
    #[derive(Default)]
    pub struct Company {
        pub name: String,
        pub currency: db_types::Currency,
    }
    #[derive(Default)]
    pub struct AccessControlForCompany {
        pub data_group: db_types::UuidType,
        pub user_: db_types::UuidType,
        pub role: db_types::Role,
    }
    #[derive(Default)]
    pub struct CompanyBranch {
        pub company_belong: db_types::UuidType,
        pub name: String,
        pub location: db_types::Location,
        pub currency: db_types::Currency,
    }
    #[derive(Default)]
    pub struct AccessControlForCompanyBranch {
        pub data_group: db_types::UuidType,
        pub user_: db_types::UuidType,
        pub role: db_types::Role,
    }
}

#[derive(Default)]
pub struct StateOfPendingTxn {
    pub user: HashMap<db_types::UuidType, tables::User>,
    pub company: HashMap<db_types::UuidType, tables::Company>,
    pub access_control_for_company: HashMap<db_types::UuidType, tables::AccessControlForCompany>,
    pub company_branch: HashMap<db_types::UuidType, tables::CompanyBranch>,
    pub access_control_for_company_branch:
        HashMap<db_types::UuidType, tables::AccessControlForCompanyBranch>,
}

pub struct State<Ch: Cache> {
    pub state_of_pending_txn: StateOfPendingTxn,
    pub cache: Ch,
}

impl<Ch: Cache> State<Ch> {
    pub async fn new() -> Self {
        let cache = Ch::new().await;
        let txns = cache.get_all_txn_input().await;

        let mut state = Self {
            state_of_pending_txn: StateOfPendingTxn::default(),
            cache,
        };

        for op in txns {
            op.operation
                .run_operation_check_apply(&mut state, &mut HashSet::new())
                .await;
        }

        state
    }
}
