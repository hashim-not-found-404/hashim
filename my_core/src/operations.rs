use crate::prelude::*;

pub(crate) trait Operations: Clone {
    type Ok;
    type Err;
    fn state_less_check(&self) -> Result<Self::Ok, Self::Err> {
        unreachable!("we dont have here state less check")
    }
    async fn state_full_check<Ch: CacheIO>(
        &self,
        state: &cache::State<Ch>,
    ) -> Result<Self::Ok, Self::Err>;
    fn apply_change(&self, state: &mut cache::StateOfPendingTxn) {
        unreachable!("we dont have here apply")
    }
    fn map_input(self) -> Input;
    fn map_result(result: Result<Self::Ok, Self::Err>) -> Output;
    fn unwrap(result: Output) -> Result<Self::Ok, Self::Err>;
}

#[derive(Deserialize, Serialize)]
pub enum Input {
    SignUp(sign_up::Input),
    SignIn(sign_in::Input),
    CreateCompany(create_company::Input),
    CreateCompanyBranch(create_company_branch::Input),
    GetUserUuid(GetUserUuidInput),
    ListCompanyAndBranch(list_company_and_branch::Input),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Output {
    SignUp(sign_up::Result),
    SignIn(sign_in::Result),
    CreateCompany(create_company::Result),
    CreateCompanyBranch(create_company_branch::Result),
    GetUserUuid(GetUserUuidOutput),
    ListCompanyAndBranch(list_company_and_branch::Result),
}

impl Input {
    pub(crate) fn run_operation_apply(&self, state: &mut cache::StateOfPendingTxn) {
        match self {
            Input::SignUp(i) => i.apply_change(state),
            Input::SignIn(i) => i.apply_change(state),
            Input::CreateCompany(i) => i.apply_change(state),
            Input::CreateCompanyBranch(i) => i.apply_change(state),
            Input::GetUserUuid(i) => i.apply_change(state),
            Input::ListCompanyAndBranch(i) => i.apply_change(state),
        }
    }

    pub(crate) async fn run_operation_check<Ch: CacheIO>(
        &self,
        state: &mut cache::State<Ch>,
    ) -> Output {
        match self {
            Input::SignUp(i) => operation_check_handler(i, state).await,
            Input::SignIn(i) => operation_check_handler(i, state).await,
            Input::CreateCompany(i) => operation_check_handler(i, state).await,
            Input::CreateCompanyBranch(i) => operation_check_handler(i, state).await,
            Input::GetUserUuid(i) => operation_check_handler(i, state).await,
            Input::ListCompanyAndBranch(i) => operation_check_handler(i, state).await,
        }
    }

    pub(crate) async fn run_operation_check_apply<Ch: CacheIO>(
        &self,
        state: &mut cache::State<Ch>,
    ) {
        match self {
            Input::SignUp(i) => operation_check_apply_handler(i, state).await,
            Input::SignIn(i) => operation_check_apply_handler(i, state).await,
            Input::CreateCompany(i) => operation_check_apply_handler(i, state).await,
            Input::CreateCompanyBranch(i) => operation_check_apply_handler(i, state).await,
            Input::GetUserUuid(i) => operation_check_apply_handler(i, state).await,
            Input::ListCompanyAndBranch(i) => operation_check_apply_handler(i, state).await,
        }
    }

    pub(crate) async fn run_operation_check_apply_write<Ch: CacheIO>(
        &self,
        txn_number: u64,
        state: &mut cache::State<Ch>,
    ) -> Output {
        match self {
            Input::SignUp(i) => operation_check_apply_write_handler(txn_number, i, state).await,
            Input::SignIn(i) => operation_check_apply_write_handler(txn_number, i, state).await,
            Input::CreateCompany(i) => {
                operation_check_apply_write_handler(txn_number, i, state).await
            }
            Input::CreateCompanyBranch(i) => {
                operation_check_apply_write_handler(txn_number, i, state).await
            }
            Input::GetUserUuid(i) => {
                operation_check_apply_write_handler(txn_number, i, state).await
            }
            Input::ListCompanyAndBranch(i) => {
                operation_check_apply_write_handler(txn_number, i, state).await
            }
        }
    }

    pub(crate) fn get_user_uuid(&self) -> Option<&db_types::UuidType> {
        match self {
            Input::SignUp(i) => Some(&i.new_uuid),
            Input::SignIn(_) => None,
            Input::CreateCompany(i) => Some(&i.user_uuid),
            Input::CreateCompanyBranch(i) => Some(&i.user_uuid),
            Input::GetUserUuid(_) => None,
            Input::ListCompanyAndBranch(i) => Some(&i.user_uuid),
        }
    }

    pub(crate) fn map_to_server_input_type(&self) -> push_data::OperationsInput {
        match self {
            Input::SignUp(i) => push_data::OperationsInput::SignUp(i.clone()),
            Input::SignIn(i) => push_data::OperationsInput::SignIn(i.clone()),
            Input::CreateCompany(i) => push_data::OperationsInput::CreateCompany(i.clone()),
            Input::CreateCompanyBranch(i) => {
                push_data::OperationsInput::CreateCompanyBranch(i.clone())
            }
            Input::GetUserUuid(_) => unreachable!(),
            Input::ListCompanyAndBranch(i) => {
                push_data::OperationsInput::ListCompanyAndBranch(i.clone())
            }
        }
    }
}

impl push_data::OperationsResult {
    pub(crate) fn map_to_client_output_type(&self) -> Output {
        match self {
            push_data::OperationsResult::SignUp(i) => Output::SignUp(i.clone()),
            push_data::OperationsResult::SignIn(i) => Output::SignIn(i.clone()),
            push_data::OperationsResult::CreateCompany(i) => Output::CreateCompany(i.clone()),
            push_data::OperationsResult::CreateCompanyBranch(i) => {
                Output::CreateCompanyBranch(i.clone())
            }
            push_data::OperationsResult::ListCompanyAndBranch(i) => {
                Output::ListCompanyAndBranch(i.clone())
            }
        }
    }
}

async fn operation_check_handler<T: Operations, Ch: CacheIO>(
    input: &T,
    state: &mut cache::State<Ch>,
) -> Output {
    let result = input.state_full_check(state).await;
    return T::map_result(result);
}

async fn operation_check_apply_handler<T: Operations, Ch: CacheIO>(
    input: &T,
    state: &mut cache::State<Ch>,
) {
    let result = input.state_full_check(state).await;

    if result.is_ok() {
        input.apply_change(&mut state.state_of_pending_txn);
    }
}

async fn operation_check_apply_write_handler<T: Operations, Ch: CacheIO>(
    txn_number: u64,
    input: &T,
    state: &mut cache::State<Ch>,
) -> Output {
    let result = input.state_full_check(state).await;

    if result.is_ok() {
        input.apply_change(&mut state.state_of_pending_txn);

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
            if user.id == self.user_id {
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

    fn apply_change(&self, state: &mut cache::StateOfPendingTxn) {
        state.user.insert(
            self.new_uuid.clone(),
            cache::tables::User {
                name: self.name.clone(),
                id: self.user_id.clone(),
                password: self.password.clone(),
            },
        );
    }

    fn map_input(self) -> Input {
        Input::SignUp(self)
    }

    fn map_result(result: Result<Self::Ok, Self::Err>) -> Output {
        Output::SignUp(result)
    }

    fn unwrap(result: Output) -> Result<Self::Ok, Self::Err> {
        if let Output::SignUp(result) = result {
            return result;
        }
        unreachable!("{:?}", result)
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
            if user.id == self.user_id {
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

    fn apply_change(&self, state: &mut cache::StateOfPendingTxn) {}

    fn map_input(self) -> Input {
        Input::SignIn(self)
    }

    fn map_result(result: Result<Self::Ok, Self::Err>) -> Output {
        Output::SignIn(result)
    }

    fn unwrap(result: Output) -> Result<Self::Ok, Self::Err> {
        if let Output::SignIn(result) = result {
            return result;
        }
        unreachable!("{:?}", result)
    }
}

impl Operations for create_company::Input {
    type Ok = create_company::Ok;
    type Err = create_company::Error;

    async fn state_full_check<Ch: CacheIO>(
        &self,
        state: &cache::State<Ch>,
    ) -> Result<Self::Ok, Self::Err> {
        Ok(create_company::Ok)
    }

    fn apply_change(&self, state: &mut cache::StateOfPendingTxn) {
        state.company.insert(
            self.new_uuid.clone(),
            cache::tables::Company {
                name: self.company_name.clone(),
                currency: self.currency.clone(),
            },
        );

        state.access_control_for_company.insert(
            self.new_uuid.clone(),
            cache::tables::AccessControlForCompany {
                data_group: self.new_uuid.clone(),
                user_: self.user_uuid.clone(),
                role: db_types::Role::Manager,
            },
        );
    }

    fn map_input(self) -> Input {
        Input::CreateCompany(self)
    }

    fn map_result(result: Result<Self::Ok, Self::Err>) -> Output {
        Output::CreateCompany(result)
    }

    fn unwrap(result: Output) -> Result<Self::Ok, Self::Err> {
        if let Output::CreateCompany(result) = result {
            return result;
        }
        unreachable!("{:?}", result)
    }
}

impl Operations for create_company_branch::Input {
    type Ok = create_company_branch::Ok;
    type Err = create_company_branch::Error;

    async fn state_full_check<Ch: CacheIO>(
        &self,
        state: &cache::State<Ch>,
    ) -> Result<Self::Ok, Self::Err> {
        todo!()
    }

    fn apply_change(&self, state: &mut cache::StateOfPendingTxn) {
        todo!("add to table company branch and table access control");
    }

    fn map_input(self) -> Input {
        Input::CreateCompanyBranch(self)
    }

    fn map_result(result: Result<Self::Ok, Self::Err>) -> Output {
        Output::CreateCompanyBranch(result)
    }

    fn unwrap(result: Output) -> Result<Self::Ok, Self::Err> {
        if let Output::CreateCompanyBranch(result) = result {
            return result;
        }
        unreachable!("{:?}", result)
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct GetUserUuidInput {
    pub user_id: String,
}

pub type GetUserUuidOutput = Result<db_types::UuidType, ()>;

impl Operations for GetUserUuidInput {
    type Ok = db_types::UuidType;
    type Err = ();

    async fn state_full_check<Ch: CacheIO>(
        &self,
        state: &cache::State<Ch>,
    ) -> Result<Self::Ok, Self::Err> {
        for (rowid, user) in &state.state_of_pending_txn.user {
            if user.id == self.user_id {
                return Ok(rowid.clone());
            }
        }

        state
            .cache
            .read_get_user_uuid(&self.user_id)
            .await
            .ok_or(())
    }

    fn map_input(self) -> Input {
        Input::GetUserUuid(self)
    }

    fn map_result(result: Result<Self::Ok, Self::Err>) -> Output {
        Output::GetUserUuid(result)
    }

    fn unwrap(result: Output) -> Result<Self::Ok, Self::Err> {
        if let Output::GetUserUuid(a) = result {
            return a;
        }
        unreachable!("{:?}", result)
    }
}

impl Operations for list_company_and_branch::Input {
    type Ok = list_company_and_branch::Ok;
    type Err = list_company_and_branch::Error;

    async fn state_full_check<Ch: CacheIO>(
        &self,
        state: &cache::State<Ch>,
    ) -> Result<Self::Ok, Self::Err> {
        let mut list_of_companies = db_types::ListOfCompanies::new();
        for (rowid, row) in &state.state_of_pending_txn.access_control_for_company {
            if row.user_ == self.user_uuid {
                let company_uuid = state.state_of_pending_txn.company.get(&row.data_group);

                list_of_companies.push(db_types::Company {
                    uuid: row.data_group.clone(),
                    name: company_uuid.unwrap().name.clone(),
                    role: row.role.clone(),
                    branches: Vec::new(),
                });
            }
        }

        // TODO make it read also from cache
        Ok(Self::Ok {
            list: list_of_companies,
        })
    }

    fn map_input(self) -> Input {
        Input::ListCompanyAndBranch(self)
    }

    fn map_result(result: Result<Self::Ok, Self::Err>) -> Output {
        Output::ListCompanyAndBranch(result)
    }

    fn unwrap(result: Output) -> Result<Self::Ok, Self::Err> {
        if let Output::ListCompanyAndBranch(a) = result {
            return a;
        }
        unreachable!("{:?}", result)
    }
}
