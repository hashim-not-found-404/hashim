use crate::prelude::*;

pub(crate) trait ViewType1 {
    fn subs() -> &'static [server_methods::Subscribe] {
        unreachable!("we dont need it here")
    }
    fn wrap_input(self) -> push_data::OperationsInput;
}

pub(crate) trait CacheAndServerType1: Clone {
    fn user_uuid(&self) -> Option<&db_types::UuidType>;

    type Output: CacheAndServerType2;
    async fn state_full_operation<Ch: CacheIO>(&self, state: &cache::State<Ch>) -> Self::Output;
    fn wrap_input1(self) -> push_data::OperationsInput;
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
        subs_to_poke: &mut HashSet<server_methods::Subscribe>,
    ) -> push_data::OperationsResult {
        match self {
            push_data::OperationsInput::SignUp(i) => {
                operation_check_apply_write_handler(txn_number, i, state, subs_to_poke).await
            }
            push_data::OperationsInput::SignIn(i) => {
                operation_check_apply_write_handler(txn_number, i, state, subs_to_poke).await
            }
            push_data::OperationsInput::CreateCompany(i) => {
                operation_check_apply_write_handler(txn_number, i, state, subs_to_poke).await
            }
            push_data::OperationsInput::CreateCompanyBranch(i) => {
                operation_check_apply_write_handler(txn_number, i, state, subs_to_poke).await
            }
            push_data::OperationsInput::ListCompanyAndBranch(i) => {
                operation_check_apply_write_handler(txn_number, i, state, subs_to_poke).await
            }
        }
    }

    pub(crate) fn get_user_uuid(&self) -> Option<&db_types::UuidType> {
        match self {
            push_data::OperationsInput::SignUp(i) => i.user_uuid(),
            push_data::OperationsInput::SignIn(i) => i.user_uuid(),
            push_data::OperationsInput::CreateCompany(i) => i.user_uuid(),
            push_data::OperationsInput::CreateCompanyBranch(i) => i.user_uuid(),
            push_data::OperationsInput::ListCompanyAndBranch(i) => i.user_uuid(),
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
    )
    .await;
}

async fn operation_check_apply_write_handler<T: CacheAndServerType1, Ch: CacheIO>(
    txn_number: u64,
    input: &T,
    state: &mut cache::State<Ch>,
    subs_to_poke: &mut HashSet<server_methods::Subscribe>,
) -> push_data::OperationsResult {
    let result = input.state_full_operation(state).await;

    if result.is_ok() {
        let resources = result.extract_resource();
        web_socket::collect_subs_to_poke(subs_to_poke, &resources);

        apply_change(resources, &mut state.state_of_pending_txn).await;

        state
            .cache
            .write_txn_input(&push_data::Txn {
                txn_number,
                operation: input.clone().wrap_input1(),
            })
            .await;
    }

    return result.wrap_output();
}

async fn apply_change(resources: Vec<ResourceInfo>, state: &mut cache::StateOfPendingTxn) {
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
            server_methods::Resource::TableCompanyBranchFieldName(r) => state
                .company_branch
                .upsert(row_uuid, |table| table.name = r),
            server_methods::Resource::TableCompanyBranchFieldCompanyBelong(r) => state
                .company_branch
                .upsert(row_uuid, |table| table.company_belong = r),
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
    fn user_uuid(&self) -> Option<&db_types::UuidType> {
        Some(&self.new_uuid)
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

        let mut resource = Vec::new();

        resource.push(ResourceInfo {
            row_uuid: self.new_uuid.clone(),
            resource: server_methods::Resource::TableUserFieldId(self.user_id.clone()),
        });
        if let Some(name) = self.name.clone() {
            resource.push(ResourceInfo {
                row_uuid: self.new_uuid.clone(),
                resource: server_methods::Resource::TableUserFieldName(name),
            });
        }

        return Ok(sign_up::Ok { resource });
    }

    fn wrap_input1(self) -> push_data::OperationsInput {
        push_data::OperationsInput::SignUp(self)
    }
}

impl CacheAndServerType2 for sign_up::Result {
    fn is_ok(&self) -> bool {
        self.is_ok()
    }

    fn extract_resource(&self) -> Vec<ResourceInfo> {
        match self {
            Ok(ok) => ok.resource.clone(),
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
    fn user_uuid(&self) -> Option<&db_types::UuidType> {
        None
    }

    type Output = sign_in::Result;

    async fn state_full_operation<Ch: CacheIO>(&self, state: &cache::State<Ch>) -> Self::Output {
        let user_uuid_and_is_jwt_exist = state.cache.read_sign_in(&self.user_id).await;

        if let Some((user_uuid, user_name, is_jwt_exist)) = user_uuid_and_is_jwt_exist {
            if is_jwt_exist {
                let mut resource = Vec::new();

                resource.push(ResourceInfo {
                    row_uuid: user_uuid,
                    resource: server_methods::Resource::TableUserFieldName(
                        user_name.unwrap_or_default(),
                    ),
                });

                return Ok(sign_in::Ok { resource });
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
                    let mut resource = Vec::new();

                    resource.push(ResourceInfo {
                        row_uuid: user_uuid.unwrap().clone(),
                        resource: server_methods::Resource::TableUserFieldName(
                            user_name.unwrap_or_default(),
                        ),
                    });

                    return Ok(sign_in::Ok { resource });
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

    fn wrap_input1(self) -> push_data::OperationsInput {
        push_data::OperationsInput::SignIn(self)
    }
}

impl CacheAndServerType2 for sign_in::Result {
    fn is_ok(&self) -> bool {
        self.is_ok()
    }

    fn extract_resource(&self) -> Vec<ResourceInfo> {
        match self {
            Ok(ok) => ok.resource.clone(),
            Err(_) => Vec::new(),
        }
    }

    fn wrap_output(self) -> push_data::OperationsResult {
        push_data::OperationsResult::SignIn(self)
    }
}

pub(crate) struct SignInResultForView(pub Result<db_types::UuidType, sign_in::Error>);

impl ViewType2 for SignInResultForView {
    fn unwrap_output(result: push_data::OperationsResult) -> Self {
        if let push_data::OperationsResult::SignIn(result) = result {
            return match result {
                Ok(ok) => SignInResultForView(Ok(ok.resource[0].row_uuid.clone())),
                Err(err) => SignInResultForView(Err(err)),
            };
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
    fn user_uuid(&self) -> Option<&db_types::UuidType> {
        Some(&self.user_uuid)
    }

    type Output = create_company::Result;

    async fn state_full_operation<Ch: CacheIO>(&self, state: &cache::State<Ch>) -> Self::Output {
        let v = ResourceInfo {
            row_uuid: self.new_uuid.clone(),
            resource: server_methods::Resource::TableCompanyFieldName(self.company_name.clone()),
        };
        let v1 = ResourceInfo {
            row_uuid: self.new_uuid.clone(),
            resource: server_methods::Resource::TableCompanyFieldCurrency(self.currency.clone()),
        };
        let v2 = ResourceInfo {
            row_uuid: self.new_uuid.clone(),
            resource: server_methods::Resource::TableAccessControlForCompanyFieldRole(
                db_types::Role::Manager,
            ),
        };
        let v3 = ResourceInfo {
            row_uuid: self.new_uuid.clone(),
            resource: server_methods::Resource::TableAccessControlForCompanyFieldUser(
                self.user_uuid.clone(),
            ),
        };
        let v4 = ResourceInfo {
            row_uuid: self.new_uuid.clone(),
            resource: server_methods::Resource::TableAccessControlForCompanyFieldDataGroup(
                self.new_uuid.clone(),
            ),
        };

        Ok(create_company::Ok {
            resource: vec![v, v1, v2, v3, v4],
        })
    }

    fn wrap_input1(self) -> push_data::OperationsInput {
        push_data::OperationsInput::CreateCompany(self)
    }
}

impl CacheAndServerType2 for create_company::Result {
    fn is_ok(&self) -> bool {
        self.is_ok()
    }

    fn extract_resource(&self) -> Vec<ResourceInfo> {
        match self {
            Ok(ok) => ok.resource.clone(),
            Err(_) => Vec::new(),
        }
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
    fn user_uuid(&self) -> Option<&db_types::UuidType> {
        Some(&self.user_uuid)
    }

    type Output = create_company_branch::Result;

    async fn state_full_operation<Ch: CacheIO>(&self, state: &cache::State<Ch>) -> Self::Output {
        todo!()
    }

    fn wrap_input1(self) -> push_data::OperationsInput {
        push_data::OperationsInput::CreateCompanyBranch(self)
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
    fn user_uuid(&self) -> Option<&db_types::UuidType> {
        Some(&self.user_uuid)
    }

    type Output = list_company_and_branch::Result;

    async fn state_full_operation<Ch: CacheIO>(&self, state: &cache::State<Ch>) -> Self::Output {
        // Start with resources from the cache (already stored in DB)
        let mut resources = state
            .cache
            .read_list_company_and_branch(&self.user_uuid)
            .await;

        // Add pending companies and branches from the current transaction
        for (_, acf) in &state.state_of_pending_txn.access_control_for_company {
            if acf.user_ == self.user_uuid {
                let company_uuid = acf.data_group.clone();
                if let Some(company) = state.state_of_pending_txn.company.get(&company_uuid) {
                    // Company name
                    resources.push(ResourceInfo {
                        row_uuid: company_uuid.clone(),
                        resource: server_methods::Resource::TableCompanyFieldName(
                            company.name.clone(),
                        ),
                    });

                    // Access control: role
                    resources.push(ResourceInfo {
                        row_uuid: company_uuid.clone(),
                        resource: server_methods::Resource::TableAccessControlForCompanyFieldRole(
                            acf.role.clone(),
                        ),
                    });

                    // Access control: user_
                    resources.push(ResourceInfo {
                        row_uuid: company_uuid.clone(),
                        resource: server_methods::Resource::TableAccessControlForCompanyFieldUser(
                            self.user_uuid.clone(),
                        ),
                    });

                    // Access control: data_group
                    resources.push(ResourceInfo {
                        row_uuid: company_uuid.clone(),
                        resource:
                            server_methods::Resource::TableAccessControlForCompanyFieldDataGroup(
                                company_uuid.clone(),
                            ),
                    });

                    // Pending branches for this company
                    for (branch_uuid, branch) in &state.state_of_pending_txn.company_branch {
                        if branch.company_belong == company_uuid {
                            resources.push(ResourceInfo {
                                row_uuid: branch_uuid.clone(),
                                resource: server_methods::Resource::TableCompanyBranchFieldName(
                                    branch.name.clone(),
                                ),
                            });
                            resources.push(ResourceInfo {
                                row_uuid: branch_uuid.clone(),
                                resource:
                                    server_methods::Resource::TableCompanyBranchFieldCompanyBelong(
                                        company_uuid.clone(),
                                    ),
                            });
                        }
                    }
                }
            }
        }

        Ok(list_company_and_branch::Ok {
            resource: resources,
        })
    }

    fn wrap_input1(self) -> push_data::OperationsInput {
        push_data::OperationsInput::ListCompanyAndBranch(self)
    }
}

impl CacheAndServerType2 for list_company_and_branch::Result {
    fn is_ok(&self) -> bool {
        self.is_ok()
    }

    fn extract_resource(&self) -> Vec<ResourceInfo> {
        match self {
            Ok(ok) => ok.resource.clone(),
            Err(_) => Vec::new(),
        }
    }

    fn wrap_output(self) -> push_data::OperationsResult {
        push_data::OperationsResult::ListCompanyAndBranch(self)
    }
}

pub(crate) struct ListCompanyAndBranchForView(pub Result<db_types::ListOfCompanies, ()>);

impl ViewType2 for ListCompanyAndBranchForView {
    fn unwrap_output(result: push_data::OperationsResult) -> Self {
        if let push_data::OperationsResult::ListCompanyAndBranch(res) = result {
            match res {
                Ok(ok) => {
                    #[derive(Default)]
                    struct CompanyData {
                        name: String,
                        currency: db_types::Currency,
                        role: db_types::Role,
                    }

                    #[derive(Default)]
                    struct BranchData {
                        name: String,
                        company_belong: db_types::UuidType,
                    }

                    let resources = ok.resource;
                    let mut company_data: HashMap<db_types::UuidType, CompanyData> = HashMap::new();
                    let mut branch_data: HashMap<db_types::UuidType, BranchData> = HashMap::new();

                    for r in resources {
                        let uuid = r.row_uuid.clone();
                        match r.resource {
                            server_methods::Resource::TableCompanyFieldName(name) => {
                                company_data.upsert(uuid, |data| data.name = name);
                            }
                            server_methods::Resource::TableCompanyFieldCurrency(currency) => {
                                company_data.upsert(uuid, |data| data.currency = currency);
                            }
                            server_methods::Resource::TableAccessControlForCompanyFieldRole(
                                role,
                            ) => {
                                company_data.upsert(uuid, |data| data.role = role);
                            }
                            server_methods::Resource::TableCompanyBranchFieldName(name) => {
                                branch_data.upsert(uuid, |data| data.name = name);
                            }
                            server_methods::Resource::TableCompanyBranchFieldCompanyBelong(
                                company_uuid,
                            ) => {
                                branch_data.upsert(uuid, |data| data.company_belong = company_uuid);
                            }
                            _ => {} // ignore other resources (Jwt, etc.)
                        }
                    }

                    // Build companies from the aggregated data
                    let mut companies = Vec::with_capacity(company_data.len());
                    for (uuid, data) in company_data {
                        let branches = branch_data
                            .iter()
                            .filter_map(|(branch_uuid, branch)| {
                                if branch.company_belong == uuid {
                                    Some(db_types::Branch {
                                        uuid: branch_uuid.clone(),
                                        name: branch.name.clone(),
                                    })
                                } else {
                                    None
                                }
                            })
                            .collect();

                        companies.push(db_types::Company {
                            uuid,
                            name: data.name,
                            role: data.role,
                            branches,
                        });
                    }

                    ListCompanyAndBranchForView(Ok(companies))
                }
                Err(_) => ListCompanyAndBranchForView(Err(())),
            }
        } else {
            unreachable!("{:?}", result)
        }
    }
}
