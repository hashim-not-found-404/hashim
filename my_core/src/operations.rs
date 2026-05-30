use crate::prelude::*;

pub(crate) trait AuthenticationOperations: Clone {
    type Ok;
    type Err;
    // async fn state_less_check(&self) -> Result<Self::Ok, Self::Err>;
    async fn state_full_check<CH: CacheIO>(
        &self,
        state: &cache::State<CH>,
    ) -> Result<Self::Ok, Self::Err>;
    fn apply_change<CH: CacheIO>(&self, state: &mut cache::State<CH>);
    fn map_input(self) -> push_data::AuthenticationMethodInput;
    fn map_result(result: Result<Self::Ok, Self::Err>) -> push_data::AuthenticationMethodResult;
    fn unwrap(result: push_data::AuthenticationMethodResult) -> Result<Self::Ok, Self::Err>;
}

pub(crate) trait WriteOperations {
    type Ok;
    type Err;
    async fn state_full_check<CH: CacheIO>(
        &self,
        state: &cache::State<CH>,
    ) -> Result<Self::Ok, Self::Err>;
    fn apply_change<CH: CacheIO>(&self, state: &mut cache::State<CH>);
    fn map_input(self) -> push_data::WriteOperationInput;
    fn map_result(result: Result<Self::Ok, Self::Err>) -> push_data::WriteOperationResult;
    fn unwrap(result: push_data::WriteOperationResult) -> Result<Self::Ok, Self::Err>;
}

pub(crate) trait ReadOperations {
    type Ok;
    type Err;
    async fn state_full_check<CH: CacheIO>(
        &self,
        state: &cache::State<CH>,
    ) -> Result<Self::Ok, Self::Err>;
    fn map_input(self) -> push_data::ReadOperationInput;
    fn map_result(result: Result<Self::Ok, Self::Err>) -> push_data::ReadOperationResult;
    fn unwrap(result: push_data::ReadOperationResult) -> Result<Self::Ok, Self::Err>;
}

impl AuthenticationOperations for sign_up::Input {
    type Ok = sign_up::Ok;
    type Err = sign_up::Error;

    async fn state_full_check<CH: CacheIO>(
        &self,
        state: &cache::State<CH>,
    ) -> Result<Self::Ok, Self::Err> {
        let mut err = Self::Err {
            new_uuid: None,
            user_id: None,
            name: None,
        };

        for (uuid, user) in &state.state_of_pending_txn.user {
            if user.user_id == self.user_id {
                err.user_id = Some(sign_up::UserIdError::Duplicated);
            }
            if uuid == &self.new_uuid {
                err.new_uuid = Some(RowIdError::Duplicated);
            }
        }

        if err != sign_up::Error::default() {
            return Err(err);
        }

        return Ok(sign_up::Ok { jwt: String::new() });
    }

    fn apply_change<CH: CacheIO>(&self, state: &mut cache::State<CH>) {
        state.state_of_pending_txn.user.insert(
            self.new_uuid.clone(),
            cache::tables::User {
                user_name: self.name.clone(),
                user_id: self.user_id.clone(),
                password: self.password.clone(),
            },
        );
    }

    fn map_input(self) -> push_data::AuthenticationMethodInput {
        push_data::AuthenticationMethodInput::SignUp(self)
    }

    fn map_result(result: Result<Self::Ok, Self::Err>) -> push_data::AuthenticationMethodResult {
        push_data::AuthenticationMethodResult::SignUp(result)
    }

    fn unwrap(result: push_data::AuthenticationMethodResult) -> Result<Self::Ok, Self::Err> {
        if let push_data::AuthenticationMethodResult::SignUp(result) = result {
            return result;
        }
        unreachable!()
    }
}
