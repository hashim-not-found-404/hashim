use crate::prelude::*;

pub mod tables {
    pub struct User {
        pub user_name: Option<String>,
        pub user_id: String,
        pub password: String,
    }
}

pub struct StateOfPendingTxn {
    pub user: HashMap<db_types::RowIdType, tables::User>,
}

impl StateOfPendingTxn {
    fn new() -> Self {
        Self {
            user: HashMap::with_capacity(20),
        }
    }
}

pub struct State<CH: CacheIO> {
    pub state_of_pending_txn: StateOfPendingTxn,
    pub cache: CH,
}

impl<CH: CacheIO> State<CH> {
    pub async fn new<RN: RandomNumber>() -> Self {
        let cache = CH::new().await;
        let txns = cache.get_all_txn_input().await;

        let mut state = Self {
            state_of_pending_txn: StateOfPendingTxn::new(),
            cache,
        };

        for op in txns {
            op.operation
                .run_txn_first_time::<_, RN>(&mut state, false)
                .await;
        }

        state
    }

    pub async fn get_jwt(&self, user_uuid: &db_types::RowIdType) -> String {
        todo!()
    }
}
