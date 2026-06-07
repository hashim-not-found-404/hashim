use crate::prelude::*;

pub(crate) trait Operations: Clone {
    type Ok;
    type Err;
    fn state_less_check(&self) -> StdResult<Self::Ok, Self::Err> {
        unreachable!("we dont have here state less check")
    }
    async fn state_full_check<Ch: CacheIO>(
        &self,
        state: &cache::State<Ch>,
    ) -> Result<Self::Ok, Self::Err>;
    fn apply_change<Ch: CacheIO>(&self, state: &mut cache::State<Ch>) {}
    fn map_input(self) -> push_data::OperationsInput;
    fn map_result(result: Result<Self::Ok, Self::Err>) -> push_data::OperationsResult;
    fn unwrap(result: push_data::OperationsResult) -> Result<Self::Ok, Self::Err>;
}

impl push_data::OperationsInput {
    pub(crate) fn run_operation_apply<Ch: CacheIO>(&self, state: &mut cache::State<Ch>) {
        match self {
            push_data::OperationsInput::SignUp(input) => input.apply_change(state),
            push_data::OperationsInput::SignIn(input) => input.apply_change(state),
            push_data::OperationsInput::CreateCompany(input) => input.apply_change(state),
            push_data::OperationsInput::CreateCompanyBranch(input) => input.apply_change(state),
        }
    }

    pub(crate) async fn run_operation_check<Ch: CacheIO>(
        &self,
        state: &mut cache::State<Ch>,
    ) -> push_data::OperationsResult {
        match self {
            push_data::OperationsInput::SignUp(input) => {
                operation_check_handler(input, state).await
            }
            push_data::OperationsInput::SignIn(input) => {
                operation_check_handler(input, state).await
            }
            push_data::OperationsInput::CreateCompany(input) => {
                operation_check_handler(input, state).await
            }
            push_data::OperationsInput::CreateCompanyBranch(input) => {
                operation_check_handler(input, state).await
            }
        }
    }

    pub(crate) async fn run_operation_check_apply<Ch: CacheIO>(
        &self,
        state: &mut cache::State<Ch>,
    ) {
        match self {
            push_data::OperationsInput::SignUp(input) => {
                operation_check_apply_handler(input, state).await
            }
            push_data::OperationsInput::SignIn(input) => {
                operation_check_apply_handler(input, state).await
            }
            push_data::OperationsInput::CreateCompany(input) => {
                operation_check_apply_handler(input, state).await
            }
            push_data::OperationsInput::CreateCompanyBranch(input) => {
                operation_check_apply_handler(input, state).await
            }
        }
    }

    pub(crate) async fn run_operation_check_apply_write<Ch: CacheIO>(
        &self,
        txn_number: u64,
        state: &mut cache::State<Ch>,
    ) -> push_data::OperationsResult {
        match self {
            push_data::OperationsInput::SignUp(input) => {
                operation_check_apply_write_handler(txn_number, input, state).await
            }
            push_data::OperationsInput::SignIn(input) => {
                operation_check_apply_write_handler(txn_number, input, state).await
            }
            push_data::OperationsInput::CreateCompany(input) => {
                operation_check_apply_write_handler(txn_number, input, state).await
            }
            push_data::OperationsInput::CreateCompanyBranch(input) => {
                operation_check_apply_write_handler(txn_number, input, state).await
            }
        }
    }
}

async fn operation_check_handler<T: Operations, Ch: CacheIO>(
    input: &T,
    state: &mut cache::State<Ch>,
) -> push_data::OperationsResult {
    let result = input.state_full_check(state).await;
    return T::map_result(result);
}

async fn operation_check_apply_handler<T: Operations, Ch: CacheIO>(
    input: &T,
    state: &mut cache::State<Ch>,
) {
    let result = input.state_full_check(state).await;

    if result.is_ok() {
        input.apply_change(state);
    }
}

async fn operation_check_apply_write_handler<T: Operations, Ch: CacheIO>(
    txn_number: u64,
    input: &T,
    state: &mut cache::State<Ch>,
) -> push_data::OperationsResult {
    let result = input.state_full_check(state).await;

    if result.is_ok() {
        input.apply_change(state);

        state
            .cache
            .write_txn_input(&push_data::Txn {
                txn_number,
                operation: input.clone().map_input(),
            })
            .await;
    }

    return T::map_result(result);
}

// all imples down

impl Operations for sign_up::Input {
    type Ok = sign_up::Ok;
    type Err = sign_up::Error;

    async fn state_full_check<Ch: CacheIO>(
        &self,
        state: &cache::State<Ch>,
    ) -> Result<Self::Ok, Self::Err> {
        let (mut is_new_uuid_exist, mut is_user_id_exist) = state
            .cache
            .read_sign_up(&self.new_uuid, &self.user_id)
            .await;

        for (uuid, user) in &state.state_of_pending_txn.user {
            if user.user_id == self.user_id {
                is_user_id_exist = true;
            }
            if uuid == &self.new_uuid {
                is_new_uuid_exist = true;
            }
        }

        let mut err = Self::Err {
            new_uuid: None,
            user_id: None,
            name: None,
        };

        if is_user_id_exist {
            err.user_id = Some(sign_up::UserIdError::Duplicated);
        }
        if is_new_uuid_exist {
            err.new_uuid = Some(RowIdError::Duplicated);
        }

        if err != sign_up::Error::default() {
            return Err(err);
        }

        return Ok(sign_up::Ok);
    }

    fn apply_change<Ch: CacheIO>(&self, state: &mut cache::State<Ch>) {
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

    async fn state_full_check<Ch: CacheIO>(
        &self,
        state: &cache::State<Ch>,
    ) -> Result<Self::Ok, Self::Err> {
        let mut password = None;

        for (_, user) in &state.state_of_pending_txn.user {
            if user.user_id == self.user_id {
                password = Some(user.password.clone());
            }
        }
        match password {
            Some(password) => {
                if password == self.password {
                    return Ok(sign_in::Ok);
                } else {
                    return Err(sign_in::Error {
                        user_id: None,
                        password: Some(sign_in::PasswordError::WrongPassword),
                    });
                }
            }
            None => Ok(sign_in::Ok),
        }
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

impl Operations for create_company::Input {
    type Ok = create_company::Ok;
    type Err = create_company::Error;

    async fn state_full_check<Ch: CacheIO>(
        &self,
        state: &cache::State<Ch>,
    ) -> StdResult<Self::Ok, Self::Err> {
        Ok(create_company::Ok)
    }

    fn map_input(self) -> push_data::OperationsInput {
        push_data::OperationsInput::CreateCompany(self)
    }

    fn map_result(result: Result<Self::Ok, Self::Err>) -> push_data::OperationsResult {
        push_data::OperationsResult::CreateCompany(result)
    }

    fn unwrap(result: push_data::OperationsResult) -> Result<Self::Ok, Self::Err> {
        if let push_data::OperationsResult::CreateCompany(result) = result {
            return result;
        }
        unreachable!()
    }
}

impl Operations for create_company_branch::Input {
    type Ok = create_company_branch::Ok;
    type Err = create_company_branch::Error;

    async fn state_full_check<Ch: CacheIO>(
        &self,
        state: &cache::State<Ch>,
    ) -> StdResult<Self::Ok, Self::Err> {
        todo!()
    }

    fn map_input(self) -> push_data::OperationsInput {
        push_data::OperationsInput::CreateCompanyBranch(self)
    }

    fn map_result(result: Result<Self::Ok, Self::Err>) -> push_data::OperationsResult {
        push_data::OperationsResult::CreateCompanyBranch(result)
    }

    fn unwrap(result: push_data::OperationsResult) -> Result<Self::Ok, Self::Err> {
        if let push_data::OperationsResult::CreateCompanyBranch(result) = result {
            return result;
        }
        unreachable!()
    }
}
