use crate::prelude::*;

pub(crate) trait Operations: Clone {
    type Ok;
    type Err;
    fn state_less_check(&self) -> StdResult<Self::Ok, Self::Err> {
        unreachable!("we dont have here state less check")
    }
    async fn state_full_check<CH: CacheIO>(
        &self,
        state: &cache::State<CH>,
    ) -> Result<Self::Ok, Self::Err>;
    fn apply_change<CH: CacheIO>(&self, state: &mut cache::State<CH>);
    fn map_input(self) -> push_data::OperationsInput;
    fn map_result(result: Result<Self::Ok, Self::Err>) -> push_data::OperationsResult;
    fn unwrap(result: push_data::OperationsResult) -> Result<Self::Ok, Self::Err>;
}

impl push_data::OperationsInput {
    pub async fn run_operation<CH: CacheIO, RN: RandomNumber>(
        &self,
        state: &mut cache::State<CH>,
        store_txn: bool,
    ) -> push_data::OperationsResult {
        match self {
            push_data::OperationsInput::SignUp(input) => {
                operation_handler::<_, _, RN>(input, state, store_txn).await
            }
            push_data::OperationsInput::SignIn(input) => {
                operation_handler::<_, _, RN>(input, state, store_txn).await
            }
            push_data::OperationsInput::CreateCompany(input) => {
                // operation_handler::<_, _, RN>(input, state, store_txn).await
                todo!()
            }
            push_data::OperationsInput::CreateCompanyBranch(input) => {
                // operation_handler::<_, _, RN>(input, state, store_txn).await
                todo!()
            }
        }
    }
}

async fn operation_handler<T: operations::Operations, CH: CacheIO, RN: RandomNumber>(
    input: &T,
    state: &mut cache::State<CH>,
    store_txn: bool,
) -> push_data::OperationsResult {
    let result = input.state_full_check(state).await;

    if result.is_ok() {
        input.apply_change(state);

        if store_txn {
            state
                .cache
                .write_txn_input(&push_data::Txn {
                    txn_number: RN::generate(),
                    operation: input.clone().map_input(),
                })
                .await;
        }
    }

    return T::map_result(result);
}

// all imples down

impl Operations for sign_up::Input {
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

    fn map_input(self) -> push_data::OperationsInput {
        push_data::OperationsInput::SignUp(self)
    }

    fn map_result(result: Result<Self::Ok, Self::Err>) -> push_data::OperationsResult {
        push_data::OperationsResult::SignUp(result)
    }

    fn unwrap(result: push_data::OperationsResult) -> Result<Self::Ok, Self::Err> {
        if let push_data::OperationsResult::SignUp(result) = result {
            return result;
        }
        unreachable!()
    }
}

impl Operations for sign_in::Input {
    type Ok = sign_in::Ok;
    type Err = sign_in::Error;

    async fn state_full_check<CH: CacheIO>(
        &self,
        state: &cache::State<CH>,
    ) -> Result<Self::Ok, Self::Err> {
        todo!() // TODO
    }

    fn apply_change<CH: CacheIO>(&self, state: &mut cache::State<CH>) {
        todo!()
    }

    fn map_input(self) -> push_data::OperationsInput {
        push_data::OperationsInput::SignIn(self)
    }

    fn map_result(result: Result<Self::Ok, Self::Err>) -> push_data::OperationsResult {
        push_data::OperationsResult::SignIn(result)
    }

    fn unwrap(result: push_data::OperationsResult) -> Result<Self::Ok, Self::Err> {
        if let push_data::OperationsResult::SignIn(result) = result {
            return result;
        }
        unreachable!()
    }
}
