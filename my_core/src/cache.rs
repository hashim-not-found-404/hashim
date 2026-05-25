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
    before_apply_txn: CH,
}

impl<CH: CacheIO> State<CH> {
    pub async fn new() -> Self {
        let cache = CH::new().await.unwrap();
        let txns = cache.get_all_write_txns().await;

        let mut state = Self {
            state_of_pending_txn: StateOfPendingTxn::new(),
            before_apply_txn: cache,
        };

        for op in txns {
            op.run_txn_first_time(&mut state).await;
        }

        state
    }

    pub async fn get_jwt(&self, user_uuid: &db_types::RowIdType) -> String {
        todo!()
    }

    pub async fn write_auth_to_cache(&self, txn: &push_data::AuthenticationMethodInput) {
        todo!()
    }

    pub async fn write_txn_to_cache(
        &self,
        txn: &push_data::TxnInput<push_data::WriteOperationInput>,
    ) {
        todo!()
    }
}
