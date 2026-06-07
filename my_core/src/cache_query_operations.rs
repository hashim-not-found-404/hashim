use crate::prelude::*;

pub(crate) trait QueryOperations {
    type Output;
    async fn read<Ch: CacheIO>(&self, state: &cache::State<Ch>) -> Self::Output;
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

impl CacheQueryInput {
    pub(crate) async fn run_query<Ch: CacheIO>(
        &self,
        state: &cache::State<Ch>,
    ) -> CacheQueryOutput {
        match self {
            CacheQueryInput::GetUserUuid(q) => GetUserUuidInput::map_output(q.read(state).await),
        }
    }
}

// impls

pub struct GetUserUuidInput {
    pub user_id: String,
}

pub struct GetUserUuidOutput {
    pub user_uuid: db_types::UuidType,
}

impl QueryOperations for GetUserUuidInput {
    type Output = GetUserUuidOutput;

    async fn read<Ch: CacheIO>(&self, state: &cache::State<Ch>) -> Self::Output {
        for (rowid, user) in &state.state_of_pending_txn.user {
            if user.user_id == self.user_id {
                return Self::Output {
                    user_uuid: rowid.clone(),
                };
            }
        }

        let user_uuid = state.cache.read_get_user_uuid(&self.user_id).await;
        mbg!(&user_uuid);

        match user_uuid {
            Some(user_uuid) => {
                return Self::Output {
                    user_uuid: user_uuid,
                };
            }
            None => {
                return Self::Output {
                    user_uuid: db_types::UuidType(self.user_id.clone()),
                };
            }
        }
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
