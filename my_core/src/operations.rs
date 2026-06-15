use crate::prelude::*;

pub(crate) trait OperationsInput: Clone {
    type Result: OperationResult;
    fn state_less_check(&self) -> Self::Result {
        unreachable!("we dont have here state less check")
    }
    async fn state_full_check<Ch: CacheIO>(&self, state: &cache::State<Ch>) -> Self::Result;
    fn apply_change(&self, state: &mut cache::StateOfPendingTxn) {
        unreachable!("we dont have here apply")
    }
    fn map_input(self) -> Input;
}

pub(crate) trait OperationResult {
    fn is_ok(&self) -> bool;
    fn map_to_resource(&self) -> Vec<ResourceInfo>;
    fn map_result_to_output(self) -> Output;
    fn map_output_to_result(result: Output) -> Self;
}

#[derive(Deserialize, Serialize)]
pub enum Input {
    SignUp(sign_up::Input),
    SignIn(sign_in::Input),
    CreateCompany(create_company::Input),
    CreateCompanyBranch(create_company_branch::Input),
    ListCompanyAndBranch(list_company_and_branch::Input),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Output {
    SignUp(sign_up::Result),
    SignIn(sign_in::Result),
    CreateCompany(create_company::Result),
    CreateCompanyBranch(create_company_branch::Result),
    ListCompanyAndBranch(list_company_and_branch::Result),
}

impl Input {
    pub(crate) fn run_operation_apply(&self, state: &mut cache::StateOfPendingTxn) {
        match self {
            Input::SignUp(i) => i.apply_change(state),
            Input::SignIn(i) => i.apply_change(state),
            Input::CreateCompany(i) => i.apply_change(state),
            Input::CreateCompanyBranch(i) => i.apply_change(state),
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
            Input::ListCompanyAndBranch(i) => {
                push_data::OperationsInput::ListCompanyAndBranch(i.clone())
            }
        }
    }
}

impl Output {
    pub(crate) fn extract_resource(&self) -> Vec<ResourceInfo> {
        match self {
            Output::SignUp(i) => i.map_to_resource(),
            Output::SignIn(i) => i.map_to_resource(),
            Output::CreateCompany(i) => i.map_to_resource(),
            Output::CreateCompanyBranch(i) => i.map_to_resource(),
            Output::ListCompanyAndBranch(i) => i.map_to_resource(),
        }
    }

    pub(crate) fn is_ok(&self) -> bool {
        match self {
            Output::SignUp(i) => i.is_ok(),
            Output::SignIn(i) => i.is_ok(),
            Output::CreateCompany(i) => i.is_ok(),
            Output::CreateCompanyBranch(i) => i.is_ok(),
            Output::ListCompanyAndBranch(i) => i.is_ok(),
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

async fn operation_check_handler<T: OperationsInput, Ch: CacheIO>(
    input: &T,
    state: &mut cache::State<Ch>,
) -> Output {
    return input.state_full_check(state).await.map_result_to_output();
}

async fn operation_check_apply_handler<T: OperationsInput, Ch: CacheIO>(
    input: &T,
    state: &mut cache::State<Ch>,
) {
    if input.state_full_check(state).await.is_ok() {
        input.apply_change(&mut state.state_of_pending_txn);
    }
}

async fn operation_check_apply_write_handler<T: OperationsInput, Ch: CacheIO>(
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

    return result.map_result_to_output();
}

// all imples down

impl OperationsInput for sign_up::Input {
    type Result = sign_up::Result;

    async fn state_full_check<Ch: CacheIO>(&self, state: &cache::State<Ch>) -> Self::Result {
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

        let mut err = sign_up::Error {
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

        return Ok(sign_up::Ok {
            user_uuid: self.new_uuid.clone(),
            jwt: String::new(),
            user_id: self.user_id.clone(),
            user_name: self.name.clone(),
        });
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
}

impl OperationResult for sign_up::Result {
    fn is_ok(&self) -> bool {
        self.is_ok()
    }

    fn map_to_resource(&self) -> Vec<ResourceInfo> {
        match self {
            Ok(ok) => {
                let mut resource = Vec::with_capacity(3);

                resource.push(ResourceInfo {
                    row_uuid: ok.user_uuid.clone(),
                    resource: server_methods::Resource::Jwt(ok.jwt.clone()),
                });
                resource.push(ResourceInfo {
                    row_uuid: ok.user_uuid.clone(),
                    resource: server_methods::Resource::UserId(ok.user_id.clone()),
                });

                if let Some(name) = &ok.user_name {
                    resource.push(ResourceInfo {
                        row_uuid: ok.user_uuid.clone(),
                        resource: server_methods::Resource::UserName(name.clone()),
                    });
                }

                resource
            }
            Err(_) => Vec::new(),
        }
    }

    fn map_result_to_output(self) -> Output {
        Output::SignUp(self)
    }

    fn map_output_to_result(result: Output) -> Self {
        if let Output::SignUp(result) = result {
            return result;
        }
        unreachable!("{:?}", result)
    }
}

impl OperationsInput for sign_in::Input {
    type Result = sign_in::Result;

    async fn state_full_check<Ch: CacheIO>(&self, state: &cache::State<Ch>) -> Self::Result {
        let user_uuid_and_is_jwt_exist = state.cache.read_sign_in(&self.user_id).await;

        if let Some((user_uuid, user_name, is_jwt_exist)) = user_uuid_and_is_jwt_exist {
            if is_jwt_exist {
                return Ok(sign_in::Ok {
                    user_uuid,
                    jwt: String::new(),
                    user_name: user_name,
                });
            }
        }

        let mut password = None;
        let mut user_uuid = None;
        let mut user_name = None;

        for (rowid, user) in &state.state_of_pending_txn.user {
            if user.id == self.user_id {
                password = Some(user.password.clone());
                user_uuid = Some(rowid);
                user_name = user.name.clone();
            }
        }

        match password {
            Some(password) => {
                if password == self.password {
                    return Ok(sign_in::Ok {
                        user_uuid: user_uuid.unwrap().clone(),
                        jwt: String::new(),
                        user_name: user_name,
                    });
                } else {
                    return Err(sign_in::Error {
                        user_id: None,
                        password: Some(sign_in::PasswordError::WrongPassword),
                    });
                }
            }
            None => Err(sign_in::Error {
                user_id: Some(sign_in::UserIdError::NotExist),
                password: None,
            }),
        }
    }

    fn map_input(self) -> Input {
        Input::SignIn(self)
    }
}

impl OperationResult for sign_in::Result {
    fn is_ok(&self) -> bool {
        self.is_ok()
    }

    fn map_to_resource(&self) -> Vec<ResourceInfo> {
        match self {
            Ok(ok) => {
                let mut resource = Vec::with_capacity(3);

                resource.push(ResourceInfo {
                    row_uuid: ok.user_uuid.clone(),
                    resource: server_methods::Resource::Jwt(ok.jwt.clone()),
                });

                if let Some(name) = &ok.user_name {
                    resource.push(ResourceInfo {
                        row_uuid: ok.user_uuid.clone(),
                        resource: server_methods::Resource::UserName(name.clone()),
                    });
                }

                resource
            }
            Err(_) => Vec::new(),
        }
    }

    fn map_result_to_output(self) -> Output {
        Output::SignIn(self)
    }

    fn map_output_to_result(result: Output) -> Self {
        if let Output::SignIn(result) = result {
            return result;
        }
        unreachable!("{:?}", result)
    }
}

impl OperationsInput for create_company::Input {
    type Result = create_company::Result;

    async fn state_full_check<Ch: CacheIO>(&self, state: &cache::State<Ch>) -> Self::Result {
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
}

impl OperationResult for create_company::Result {
    fn is_ok(&self) -> bool {
        self.is_ok()
    }

    fn map_to_resource(&self) -> Vec<ResourceInfo> {
        Vec::new()
    }

    fn map_result_to_output(self) -> Output {
        Output::CreateCompany(self)
    }

    fn map_output_to_result(result: Output) -> Self {
        if let Output::CreateCompany(result) = result {
            return result;
        }
        unreachable!("{:?}", result)
    }
}

impl OperationsInput for create_company_branch::Input {
    type Result = create_company_branch::Result;

    async fn state_full_check<Ch: CacheIO>(&self, state: &cache::State<Ch>) -> Self::Result {
        todo!()
    }

    fn apply_change(&self, state: &mut cache::StateOfPendingTxn) {
        todo!("add to table company branch and table access control");
    }

    fn map_input(self) -> Input {
        Input::CreateCompanyBranch(self)
    }
}

impl OperationResult for create_company_branch::Result {
    fn is_ok(&self) -> bool {
        self.is_ok()
    }

    fn map_to_resource(&self) -> Vec<ResourceInfo> {
        Vec::new()
    }

    fn map_result_to_output(self) -> Output {
        Output::CreateCompanyBranch(self)
    }

    fn map_output_to_result(result: Output) -> Self {
        if let Output::CreateCompanyBranch(result) = result {
            return result;
        }
        unreachable!("{:?}", result)
    }
}

impl OperationsInput for list_company_and_branch::Input {
    type Result = list_company_and_branch::Result;

    async fn state_full_check<Ch: CacheIO>(&self, state: &cache::State<Ch>) -> Self::Result {
        let mut list_of_companies = state
            .cache
            .read_list_company_and_branch(&self.user_uuid)
            .await;

        for (rowid, row) in &state.state_of_pending_txn.access_control_for_company {
            if row.user_ == self.user_uuid {
                let company_uuid = state.state_of_pending_txn.company.get(&row.data_group);

                list_of_companies.push(db_types::Company {
                    uuid: row.data_group.clone(),
                    name: company_uuid.unwrap().name.clone(),
                    role: row.role.clone(),
                    branches: Vec::new(),
                });

                // TODO make it read also branch
            }
        }

        Ok(list_company_and_branch::Ok {
            list: list_of_companies,
        })
    }

    fn map_input(self) -> Input {
        Input::ListCompanyAndBranch(self)
    }
}

impl OperationResult for list_company_and_branch::Result {
    fn is_ok(&self) -> bool {
        self.is_ok()
    }

    fn map_to_resource(&self) -> Vec<ResourceInfo> {
        match self {
            Ok(ok) => {
                let mut resource = Vec::new();

                for company in ok.list.clone() {
                    resource.push(ResourceInfo {
                        row_uuid: company.uuid.clone(),
                        resource: server_methods::Resource::CompanyName(company.name),
                    });
                    resource.push(ResourceInfo {
                        row_uuid: company.uuid.clone(),
                        resource: server_methods::Resource::RoleAtCompany(company.role),
                    });
                    for branch in company.branches {
                        resource.push(ResourceInfo {
                            row_uuid: branch.uuid.clone(),
                            resource: server_methods::Resource::BranchName(branch.name),
                        });
                        resource.push(ResourceInfo {
                            row_uuid: branch.uuid.clone(),
                            resource:
                                server_methods::Resource::TableCompanyBranchFieldCompanyBelong(
                                    company.uuid.clone(),
                                ),
                        });
                    }
                }

                resource
            }
            Err(_) => Vec::new(),
        }
    }

    fn map_result_to_output(self) -> Output {
        Output::ListCompanyAndBranch(self)
    }

    fn map_output_to_result(result: Output) -> Self {
        if let Output::ListCompanyAndBranch(a) = result {
            return a;
        }
        unreachable!("{:?}", result)
    }
}
