use crate::prelude::*;

pub(crate) trait QueryOperations {
    type Output;
    async fn read<CH: CacheIO>(&self, state: &cache::State<CH>) -> Self::Output;
    fn map_input(self) -> CacheQueryInput;
    fn map_output(result: Self::Output) -> CacheQueryOutput;
    fn unwrap(result: CacheQueryOutput) -> Self::Output;
}

pub enum CacheQueryInput {
    GetUserUuid(GetUserUuidInput),
}

pub enum CacheQueryOutput {
    GetUserUuid(GetUserUuidOutput),
}

pub struct GetUserUuidInput {
    pub user_id: String,
}

pub struct GetUserUuidOutput {
    pub user_uuid: db_types::RowIdType,
}

impl QueryOperations for GetUserUuidInput {
    type Output = GetUserUuidOutput;

    async fn read<CH: CacheIO>(&self, state: &cache::State<CH>) -> Self::Output {
        for (rowid, user) in &state.state_of_pending_txn.user {
            if user.user_id == self.user_id {
                return Self::Output {
                    user_uuid: rowid.clone(),
                };
            }
        }

        todo!("make the query from cache");
        return Self::Output {
            user_uuid: db_types::RowIdType("todo".to_string()),
        };
    }

    fn map_input(self) -> CacheQueryInput {
        CacheQueryInput::GetUserUuid(self)
    }

    fn map_output(result: Self::Output) -> CacheQueryOutput {
        CacheQueryOutput::GetUserUuid(result)
    }

    fn unwrap(result: CacheQueryOutput) -> Self::Output {
        if let CacheQueryOutput::GetUserUuid(a) = result {
            return a;
        }
        unreachable!()
    }
}
