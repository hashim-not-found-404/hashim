use crate::prelude::*;

pub mod tables {
    use crate::db_types;

    pub struct User {
        pub name: Option<String>,
        pub id: String,
        pub password: String,
    }
    pub struct Company {
        pub name: String,
        pub currency: db_types::Currency,
    }
    pub struct AccessControlForCompany {
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
}

pub struct State<Ch: CacheIO> {
    pub state_of_pending_txn: StateOfPendingTxn,
    pub cache: Ch,
}

impl<Ch: CacheIO> State<Ch> {
    pub async fn new<RN: RandomNumber>() -> Self {
        let cache = Ch::new().await;
        let txns = cache.get_all_txn_input().await;

        let mut state = Self {
            state_of_pending_txn: StateOfPendingTxn::default(),
            cache,
        };

        for op in txns {
            op.operation
                .run_operation_apply(&mut state.state_of_pending_txn);
        }

        state
    }
}
