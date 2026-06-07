use crate::prelude::*;

pub mod tables {
    pub struct User {
        pub user_name: Option<String>,
        pub user_id: String,
        pub password: String,
    }
}

pub struct StateOfPendingTxn {
    pub user: HashMap<db_types::UuidType, tables::User>,
}

impl StateOfPendingTxn {
    pub fn new() -> Self {
        Self {
            user: HashMap::with_capacity(20),
        }
    }
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
            state_of_pending_txn: StateOfPendingTxn::new(),
            cache,
        };

        for op in txns {
            op.operation.run_operation_apply(&mut state);
        }

        state
    }
}
