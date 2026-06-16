use crate::prelude::*;

pub(crate) trait ViewType1: Clone {
    fn subs() -> &'static [server_methods::Subscribe] {
        unreachable!("we dont need it here")
    }
    fn wrap_input(self) -> push_data::OperationsInput;
}

pub(crate) trait CacheAndServerType1: Clone {
    fn user_uuid(&self) -> Option<db_types::UuidType>;

    type Output: CacheAndServerType2;
    async fn state_full_operation<Ch: CacheIO>(&self, state: &cache::State<Ch>) -> Self::Output;
    // fn wrap_input(self) -> push_data::OperationsInput;
}

pub(crate) trait CacheAndServerType2 {
    fn is_ok(&self) -> bool;
    fn extract_resource(&self) -> Vec<ResourceInfo>;
    fn wrap_output(self) -> push_data::OperationsResult;
}

pub(crate) trait ViewType2 {
    fn unwrap_output(output: push_data::OperationsResult) -> Self;
}

impl push_data::OperationsInput {
    pub(crate) async fn run_operation_check<Ch: CacheIO>(
        &self,
        state: &mut cache::State<Ch>,
    ) -> push_data::OperationsResult {
        match self {
            push_data::OperationsInput::SignUp(i) => operation_check_handler(i, state).await,
            push_data::OperationsInput::SignIn(i) => operation_check_handler(i, state).await,
            push_data::OperationsInput::CreateCompany(i) => operation_check_handler(i, state).await,
            push_data::OperationsInput::CreateCompanyBranch(i) => {
                operation_check_handler(i, state).await
            }
            push_data::OperationsInput::ListCompanyAndBranch(i) => {
                operation_check_handler(i, state).await
            }
        }
    }

    pub(crate) async fn run_operation_check_apply<Ch: CacheIO>(
        &self,
        state: &mut cache::State<Ch>,
    ) {
        match self {
            push_data::OperationsInput::SignUp(i) => operation_check_apply_handler(i, state).await,
            push_data::OperationsInput::SignIn(i) => operation_check_apply_handler(i, state).await,
            push_data::OperationsInput::CreateCompany(i) => {
                operation_check_apply_handler(i, state).await
            }
            push_data::OperationsInput::CreateCompanyBranch(i) => {
                operation_check_apply_handler(i, state).await
            }
            push_data::OperationsInput::ListCompanyAndBranch(i) => {
                operation_check_apply_handler(i, state).await
            }
        }
    }

    pub(crate) async fn run_operation_check_apply_write<Ch: CacheIO>(
        &self,
        txn_number: u64,
        state: &mut cache::State<Ch>,
    ) -> push_data::OperationsResult {
        match self {
            push_data::OperationsInput::SignUp(i) => {
                operation_check_apply_write_handler(txn_number, i, state).await
            }
            push_data::OperationsInput::SignIn(i) => {
                operation_check_apply_write_handler(txn_number, i, state).await
            }
            push_data::OperationsInput::CreateCompany(i) => {
                operation_check_apply_write_handler(txn_number, i, state).await
            }
            push_data::OperationsInput::CreateCompanyBranch(i) => {
                operation_check_apply_write_handler(txn_number, i, state).await
            }
            push_data::OperationsInput::ListCompanyAndBranch(i) => {
                operation_check_apply_write_handler(txn_number, i, state).await
            }
        }
    }

    pub(crate) fn get_user_uuid(&self) -> Option<&db_types::UuidType> {
        match self {
            push_data::OperationsInput::SignUp(i) => Some(&i.new_uuid),
            push_data::OperationsInput::SignIn(_) => None,
            push_data::OperationsInput::CreateCompany(i) => Some(&i.user_uuid),
            push_data::OperationsInput::CreateCompanyBranch(i) => Some(&i.user_uuid),
            push_data::OperationsInput::ListCompanyAndBranch(i) => Some(&i.user_uuid),
        }
    }

    pub(crate) fn map_to_server_input_type(&self) -> push_data::OperationsInput {
        match self {
            push_data::OperationsInput::SignUp(i) => push_data::OperationsInput::SignUp(i.clone()),
            push_data::OperationsInput::SignIn(i) => push_data::OperationsInput::SignIn(i.clone()),
            push_data::OperationsInput::CreateCompany(i) => {
                push_data::OperationsInput::CreateCompany(i.clone())
            }
            push_data::OperationsInput::CreateCompanyBranch(i) => {
                push_data::OperationsInput::CreateCompanyBranch(i.clone())
            }
            push_data::OperationsInput::ListCompanyAndBranch(i) => {
                push_data::OperationsInput::ListCompanyAndBranch(i.clone())
            }
        }
    }
}

impl push_data::OperationsResult {
    pub(crate) fn extract_resource(&self) -> Vec<ResourceInfo> {
        match self {
            push_data::OperationsResult::SignUp(i) => i.extract_resource(),
            push_data::OperationsResult::SignIn(i) => i.extract_resource(),
            push_data::OperationsResult::CreateCompany(i) => i.extract_resource(),
            push_data::OperationsResult::CreateCompanyBranch(i) => i.extract_resource(),
            push_data::OperationsResult::ListCompanyAndBranch(i) => i.extract_resource(),
        }
    }

    pub(crate) fn is_ok(&self) -> bool {
        match self {
            push_data::OperationsResult::SignUp(i) => i.is_ok(),
            push_data::OperationsResult::SignIn(i) => i.is_ok(),
            push_data::OperationsResult::CreateCompany(i) => i.is_ok(),
            push_data::OperationsResult::CreateCompanyBranch(i) => i.is_ok(),
            push_data::OperationsResult::ListCompanyAndBranch(i) => i.is_ok(),
        }
    }
}

async fn operation_check_handler<T: CacheAndServerType1, Ch: CacheIO>(
    input: &T,
    state: &mut cache::State<Ch>,
) -> push_data::OperationsResult {
    return input.state_full_operation(state).await.wrap_output();
}

async fn operation_check_apply_handler<T: CacheAndServerType1, Ch: CacheIO>(
    input: &T,
    state: &mut cache::State<Ch>,
) {
    apply_change(
        input.state_full_operation(state).await.extract_resource(),
        &mut state.state_of_pending_txn,
    );
}

async fn operation_check_apply_write_handler<T: CacheAndServerType1 + ViewType1, Ch: CacheIO>(
    txn_number: u64,
    input: &T,
    state: &mut cache::State<Ch>,
) -> push_data::OperationsResult {
    let result = input.state_full_operation(state).await;

    if result.is_ok() {
        apply_change(result.extract_resource(), &mut state.state_of_pending_txn);

        state
            .cache
            .write_txn_input(&push_data::Txn {
                txn_number,
                operation: input.clone().wrap_input(),
            })
            .await;
    }

    return result.wrap_output();
}

trait Sdfsdfojzofjoz<V> {
    fn upsert<F>(&mut self, row_uuid: db_types::UuidType, f: F)
    where
        F: FnOnce(&mut V);
}

impl<V: Default> Sdfsdfojzofjoz<V> for HashMap<db_types::UuidType, V> {
    fn upsert<F>(&mut self, row_uuid: db_types::UuidType, f: F)
    where
        F: FnOnce(&mut V),
    {
        self.entry(row_uuid).and_modify(f).or_insert(V::default());
    }
}

fn apply_change(resources: Vec<ResourceInfo>, state: &mut cache::StateOfPendingTxn) {
    for resource in resources {
        let row_uuid = resource.row_uuid;

        match resource.resource {
            server_methods::Resource::Jwt(_) => {}
            server_methods::Resource::TableUserFieldName(r) => {
                state.user.upsert(row_uuid, |table| table.name = Some(r))
            }
            server_methods::Resource::TableUserFieldId(r) => {
                state.user.upsert(row_uuid, |table| table.id = r)
            }
            server_methods::Resource::TableCompanyFieldName(r) => {
                state.company.upsert(row_uuid, |table| table.name = r)
            }
            server_methods::Resource::TableCompanyBranchFieldName(r) => {
                todo!();
                // state.user.upsert(row_uuid, |table| table.name = r)
            }
            server_methods::Resource::TableCompanyBranchFieldCompanyBelong(r) => {
                todo!();
                // state.user.upsert(row_uuid, |table| table.name = r)
            }
            server_methods::Resource::TableCompanyFieldCurrency(r) => {
                state.company.upsert(row_uuid, |table| table.currency = r)
            }
            server_methods::Resource::TableAccessControlForCompanyFieldRole(r) => state
                .access_control_for_company
                .upsert(row_uuid, |table| table.role = r),
            server_methods::Resource::TableAccessControlForCompanyFieldUser(r) => state
                .access_control_for_company
                .upsert(row_uuid, |table| table.user_ = r),
            server_methods::Resource::TableAccessControlForCompanyFieldDataGroup(r) => state
                .access_control_for_company
                .upsert(row_uuid, |table| table.data_group = r),
        }
    }
}

// all imples down

impl ViewType1 for sign_up::Input {
    fn wrap_input(self) -> push_data::OperationsInput {
        push_data::OperationsInput::SignUp(self)
    }
}

impl CacheAndServerType1 for sign_up::Input {
    fn user_uuid(&self) -> Option<db_types::UuidType> {
        Some(self.new_uuid.clone())
    }

    type Output = sign_up::Result;
    async fn state_full_operation<Ch: CacheIO>(&self, state: &cache::State<Ch>) -> Self::Output {
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
}

impl CacheAndServerType2 for sign_up::Result {
    fn is_ok(&self) -> bool {
        self.is_ok()
    }

    fn extract_resource(&self) -> Vec<ResourceInfo> {
        match self {
            Ok(ok) => {
                let mut resource = Vec::with_capacity(3);

                resource.push(ResourceInfo {
                    row_uuid: ok.user_uuid.clone(),
                    resource: server_methods::Resource::Jwt(ok.jwt.clone()),
                });
                resource.push(ResourceInfo {
                    row_uuid: ok.user_uuid.clone(),
                    resource: server_methods::Resource::TableUserFieldId(ok.user_id.clone()),
                });

                if let Some(name) = &ok.user_name {
                    resource.push(ResourceInfo {
                        row_uuid: ok.user_uuid.clone(),
                        resource: server_methods::Resource::TableUserFieldName(name.clone()),
                    });
                }

                resource
            }
            Err(_) => Vec::new(),
        }
    }

    fn wrap_output(self) -> push_data::OperationsResult {
        push_data::OperationsResult::SignUp(self)
    }
}

impl ViewType2 for sign_up::Result {
    fn unwrap_output(result: push_data::OperationsResult) -> Self {
        if let push_data::OperationsResult::SignUp(result) = result {
            return result;
        }
        unreachable!("{:?}", result)
    }
}

impl ViewType1 for sign_in::Input {
    fn wrap_input(self) -> push_data::OperationsInput {
        push_data::OperationsInput::SignIn(self)
    }
}

impl CacheAndServerType1 for sign_in::Input {
    fn user_uuid(&self) -> Option<db_types::UuidType> {
        None
    }

    type Output = sign_in::Result;

    async fn state_full_operation<Ch: CacheIO>(&self, state: &cache::State<Ch>) -> Self::Output {
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
}

impl CacheAndServerType2 for sign_in::Result {
    fn is_ok(&self) -> bool {
        self.is_ok()
    }

    fn extract_resource(&self) -> Vec<ResourceInfo> {
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
                        resource: server_methods::Resource::TableUserFieldName(name.clone()),
                    });
                }

                resource
            }
            Err(_) => Vec::new(),
        }
    }

    fn wrap_output(self) -> push_data::OperationsResult {
        push_data::OperationsResult::SignIn(self)
    }
}

impl ViewType2 for sign_in::Result {
    fn unwrap_output(result: push_data::OperationsResult) -> Self {
        if let push_data::OperationsResult::SignIn(result) = result {
            return result;
        }
        unreachable!("{:?}", result)
    }
}

impl ViewType1 for create_company::Input {
    fn wrap_input(self) -> push_data::OperationsInput {
        push_data::OperationsInput::CreateCompany(self)
    }
}

impl CacheAndServerType1 for create_company::Input {
    fn user_uuid(&self) -> Option<db_types::UuidType> {
        todo!()
    }

    type Output = create_company::Result;

    async fn state_full_operation<Ch: CacheIO>(&self, state: &cache::State<Ch>) -> Self::Output {
        Ok(create_company::Ok)
    }
}

impl CacheAndServerType2 for create_company::Result {
    fn is_ok(&self) -> bool {
        self.is_ok()
    }

    fn extract_resource(&self) -> Vec<ResourceInfo> {
        Vec::new()
    }

    fn wrap_output(self) -> push_data::OperationsResult {
        push_data::OperationsResult::CreateCompany(self)
    }
}

impl ViewType2 for create_company::Result {
    fn unwrap_output(result: push_data::OperationsResult) -> Self {
        if let push_data::OperationsResult::CreateCompany(result) = result {
            return result;
        }
        unreachable!("{:?}", result)
    }
}

impl ViewType1 for create_company_branch::Input {
    fn wrap_input(self) -> push_data::OperationsInput {
        push_data::OperationsInput::CreateCompanyBranch(self)
    }
}

impl CacheAndServerType1 for create_company_branch::Input {
    fn user_uuid(&self) -> Option<db_types::UuidType> {
        todo!()
    }

    type Output = create_company_branch::Result;

    async fn state_full_operation<Ch: CacheIO>(&self, state: &cache::State<Ch>) -> Self::Output {
        todo!()
    }
}

impl CacheAndServerType2 for create_company_branch::Result {
    fn is_ok(&self) -> bool {
        self.is_ok()
    }

    fn extract_resource(&self) -> Vec<ResourceInfo> {
        Vec::new()
    }

    fn wrap_output(self) -> push_data::OperationsResult {
        push_data::OperationsResult::CreateCompanyBranch(self)
    }
}

impl ViewType2 for create_company_branch::Result {
    fn unwrap_output(result: push_data::OperationsResult) -> Self {
        if let push_data::OperationsResult::CreateCompanyBranch(result) = result {
            return result;
        }
        unreachable!("{:?}", result)
    }
}

impl ViewType1 for list_company_and_branch::Input {
    fn subs() -> &'static [server_methods::Subscribe] {
        &[
            server_methods::Subscribe::TableCompanyBranchFieldName,
            server_methods::Subscribe::TableCompanyFieldName,
            server_methods::Subscribe::TableAccessControlForCompanyFieldRole,
        ]
    }

    fn wrap_input(self) -> push_data::OperationsInput {
        push_data::OperationsInput::ListCompanyAndBranch(self)
    }
}

impl CacheAndServerType1 for list_company_and_branch::Input {
    fn user_uuid(&self) -> Option<db_types::UuidType> {
        Some(self.user_uuid.clone())
    }

    type Output = list_company_and_branch::Result;

    async fn state_full_operation<Ch: CacheIO>(&self, state: &cache::State<Ch>) -> Self::Output {
        let mut list_of_companies = state
            .cache
            .read_list_company_and_branch(&self.user_uuid)
            .await;

        mbg!(&list_of_companies); // %cINFO%c my_core/src/operations.rs:515%c
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
        mbg!(&list_of_companies); // %cINFO%c my_core/src/operations.rs:530%c

        Ok(list_company_and_branch::Ok {
            list: list_of_companies,
        })
    }
}

impl CacheAndServerType2 for list_company_and_branch::Result {
    fn is_ok(&self) -> bool {
        self.is_ok()
    }

    fn extract_resource(&self) -> Vec<ResourceInfo> {
        match self {
            Ok(ok) => {
                let mut resource = Vec::new();

                for company in ok.list.clone() {
                    resource.push(ResourceInfo {
                        row_uuid: company.uuid.clone(),
                        resource: server_methods::Resource::TableCompanyFieldName(company.name),
                    });
                    resource.push(ResourceInfo {
                        row_uuid: company.uuid.clone(),
                        resource: server_methods::Resource::TableAccessControlForCompanyFieldRole(
                            company.role,
                        ),
                    });
                    for branch in company.branches {
                        resource.push(ResourceInfo {
                            row_uuid: branch.uuid.clone(),
                            resource: server_methods::Resource::TableCompanyBranchFieldName(
                                branch.name,
                            ),
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

    fn wrap_output(self) -> push_data::OperationsResult {
        push_data::OperationsResult::ListCompanyAndBranch(self)
    }
}

impl ViewType2 for list_company_and_branch::Result {
    // TODO i need to change the type to be usable for ui
    fn unwrap_output(result: push_data::OperationsResult) -> Self {
        if let push_data::OperationsResult::ListCompanyAndBranch(a) = result {
            return a;
        }
        unreachable!("{:?}", result)
    }
}
