use crate::{
    accounting_client::{cache, cache_actor},
    accounting_domain::{
        cases::{self, MyErrorTrait},
        request_response, types,
    },
    utility::utils::MyUpSert,
};
use std::collections::HashSet;

pub(crate) trait ViewType1 {
    fn subs() -> &'static [types::Subscribe] {
        unreachable!("we dont need it here")
    }
    fn wrap_input(self) -> request_response::push_data::OperationsInput;
}

pub(crate) trait CacheAndServerType1: Clone {
    fn user_uuid(&self) -> Option<&types::UuidType>;

    type Output: CacheAndServerType2;
    async fn state_full_operation<Id: cases::RowId, Ch: cache::Cache>(
        &self,
        state: &mut cache::State<Ch>,
    ) -> Self::Output;
}

pub(crate) trait CacheAndServerType2 {
    fn extract_resource(&self) -> Vec<types::ResourceInfo>;
    fn wrap_output(self) -> request_response::push_data::OperationsResult;
}

pub(crate) trait ViewType2 {
    fn unwrap_output(output: request_response::push_data::OperationsResult) -> Self;
}

impl request_response::push_data::OperationsInput {
    pub(crate) async fn run_operation_check<Id: cases::RowId, Ch: cache::Cache>(
        &self,
        state: &mut cache::State<Ch>,
    ) -> request_response::push_data::OperationsResult {
        match self {
            request_response::push_data::OperationsInput::SignUp(i) => {
                operation_check_handler::<_, Id, Ch>(i, state).await
            }
            request_response::push_data::OperationsInput::SignIn(i) => {
                operation_check_handler::<_, Id, Ch>(i, state).await
            }
            request_response::push_data::OperationsInput::CreateCompany(i) => {
                operation_check_handler::<_, Id, Ch>(i, state).await
            }
            request_response::push_data::OperationsInput::CreateCompanyBranch(i) => {
                operation_check_handler::<_, Id, Ch>(i, state).await
            }
            request_response::push_data::OperationsInput::ListCompanyAndBranch(i) => {
                operation_check_handler::<_, Id, Ch>(i, state).await
            }
        }
    }

    pub(crate) async fn run_operation_check_apply<Id: cases::RowId, Ch: cache::Cache>(
        &self,
        state: &mut cache::State<Ch>,
        subs_to_poke: &mut HashSet<types::Subscribe>,
    ) {
        match self {
            request_response::push_data::OperationsInput::SignUp(i) => {
                operation_check_apply_handler::<_, Id, Ch>(i, state, subs_to_poke).await
            }
            request_response::push_data::OperationsInput::SignIn(i) => {
                operation_check_apply_handler::<_, Id, Ch>(i, state, subs_to_poke).await
            }
            request_response::push_data::OperationsInput::CreateCompany(i) => {
                operation_check_apply_handler::<_, Id, Ch>(i, state, subs_to_poke).await
            }
            request_response::push_data::OperationsInput::CreateCompanyBranch(i) => {
                operation_check_apply_handler::<_, Id, Ch>(i, state, subs_to_poke).await
            }
            request_response::push_data::OperationsInput::ListCompanyAndBranch(i) => {
                operation_check_apply_handler::<_, Id, Ch>(i, state, subs_to_poke).await
            }
        }
    }

    pub(crate) async fn run_operation_check_apply_write<Id: cases::RowId, Ch: cache::Cache>(
        &self,
        txn_number: u64,
        state: &mut cache::State<Ch>,
        subs_to_poke: &mut HashSet<types::Subscribe>,
    ) -> request_response::push_data::OperationsResult {
        state
            .cache
            .write_txn_input(&request_response::push_data::Txn {
                txn_number,
                operation: self.clone(),
            })
            .await;

        match self {
            request_response::push_data::OperationsInput::SignUp(i) => {
                operation_check_apply_write_handler::<_, Id, Ch>(i, state, subs_to_poke).await
            }
            request_response::push_data::OperationsInput::SignIn(i) => {
                operation_check_apply_write_handler::<_, Id, Ch>(i, state, subs_to_poke).await
            }
            request_response::push_data::OperationsInput::CreateCompany(i) => {
                operation_check_apply_write_handler::<_, Id, Ch>(i, state, subs_to_poke).await
            }
            request_response::push_data::OperationsInput::CreateCompanyBranch(i) => {
                operation_check_apply_write_handler::<_, Id, Ch>(i, state, subs_to_poke).await
            }
            request_response::push_data::OperationsInput::ListCompanyAndBranch(i) => {
                operation_check_apply_write_handler::<_, Id, Ch>(i, state, subs_to_poke).await
            }
        }
    }

    pub(crate) fn get_user_uuid(&self) -> Option<&types::UuidType> {
        match self {
            request_response::push_data::OperationsInput::SignUp(i) => i.user_uuid(),
            request_response::push_data::OperationsInput::SignIn(i) => i.user_uuid(),
            request_response::push_data::OperationsInput::CreateCompany(i) => i.user_uuid(),
            request_response::push_data::OperationsInput::CreateCompanyBranch(i) => i.user_uuid(),
            request_response::push_data::OperationsInput::ListCompanyAndBranch(i) => i.user_uuid(),
        }
    }
}

impl request_response::push_data::OperationsResult {
    pub(crate) fn extract_resource(&self) -> Vec<types::ResourceInfo> {
        match self {
            request_response::push_data::OperationsResult::SignUp(i) => i.extract_resource(),
            request_response::push_data::OperationsResult::SignIn(i) => i.extract_resource(),
            request_response::push_data::OperationsResult::CreateCompany(i) => i.extract_resource(),
            request_response::push_data::OperationsResult::CreateCompanyBranch(i) => {
                i.extract_resource()
            }
            request_response::push_data::OperationsResult::ListCompanyAndBranch(i) => {
                i.extract_resource()
            }
        }
    }

    pub(crate) fn is_ok(&self) -> bool {
        match self {
            request_response::push_data::OperationsResult::SignUp(i) => i.is_ok(),
            request_response::push_data::OperationsResult::SignIn(i) => i.is_ok(),
            request_response::push_data::OperationsResult::CreateCompany(i) => i.is_ok(),
            request_response::push_data::OperationsResult::CreateCompanyBranch(i) => i.is_ok(),
            request_response::push_data::OperationsResult::ListCompanyAndBranch(i) => i.is_ok(),
        }
    }
}

async fn operation_check_handler<T: CacheAndServerType1, Id: cases::RowId, Ch: cache::Cache>(
    input: &T,
    state: &mut cache::State<Ch>,
) -> request_response::push_data::OperationsResult {
    return input
        .state_full_operation::<Id, Ch>(state)
        .await
        .wrap_output();
}

async fn operation_check_apply_handler<
    T: CacheAndServerType1,
    Id: cases::RowId,
    Ch: cache::Cache,
>(
    input: &T,
    state: &mut cache::State<Ch>,
    subs_to_poke: &mut HashSet<types::Subscribe>,
) {
    apply_change(
        input
            .state_full_operation::<Id, Ch>(state)
            .await
            .extract_resource(),
        &mut state.state_of_pending_txn,
        subs_to_poke,
    )
    .await;
}

async fn operation_check_apply_write_handler<
    T: CacheAndServerType1,
    Id: cases::RowId,
    Ch: cache::Cache,
>(
    input: &T,
    state: &mut cache::State<Ch>,
    subs_to_poke: &mut HashSet<types::Subscribe>,
) -> request_response::push_data::OperationsResult {
    let result = input.state_full_operation::<Id, Ch>(state).await;

    apply_change(
        result.extract_resource(),
        &mut state.state_of_pending_txn,
        subs_to_poke,
    )
    .await;

    return result.wrap_output();
}

async fn apply_change(
    resources: Vec<types::ResourceInfo>,
    state: &mut cache::tables::StateOfPendingTxn,
    subs_to_poke: &mut HashSet<types::Subscribe>,
) {
    cache_actor::collect_subs_to_poke(subs_to_poke, &resources);

    for resource in resources {
        let row_uuid = resource.row_uuid;

        match resource.resource {
            types::Resource::Jwt(_) => {}
            types::Resource::TableUserFieldName(r) => {
                state.user.upsert(row_uuid, |table| table.name = Some(r))
            }
            types::Resource::TableUserFieldId(r) => {
                state.user.upsert(row_uuid, |table| table.id = r)
            }
            types::Resource::TableCompanyFieldName(r) => {
                state.company.upsert(row_uuid, |table| table.name = r)
            }
            types::Resource::TableCompanyBranchFieldName(r) => state
                .company_branch
                .upsert(row_uuid, |table| table.name = r),
            types::Resource::TableCompanyBranchFieldCompanyBelong(r) => state
                .company_branch
                .upsert(row_uuid, |table| table.company_belong = r),
            types::Resource::TableCompanyBranchFieldCurrency(r) => state
                .company_branch
                .upsert(row_uuid, |table| table.currency = r),
            types::Resource::TableCompanyBranchFieldLocation(r) => state
                .company_branch
                .upsert(row_uuid, |table| table.location = r),
            types::Resource::TableCompanyFieldCurrency(r) => {
                state.company.upsert(row_uuid, |table| table.currency = r)
            }
            types::Resource::TableAccessControlForCompanyFieldRole(r) => state
                .access_control_for_company
                .upsert(row_uuid, |table| table.role = r),
            types::Resource::TableAccessControlForCompanyFieldUser(r) => state
                .access_control_for_company
                .upsert(row_uuid, |table| table.user_ = r),
            types::Resource::TableAccessControlForCompanyFieldDataGroup(r) => state
                .access_control_for_company
                .upsert(row_uuid, |table| table.data_group = r),
            types::Resource::TableAccessControlForCompanyBranchFieldRole(r) => state
                .access_control_for_company_branch
                .upsert(row_uuid, |table| table.role = r),
            types::Resource::TableAccessControlForCompanyBranchFieldUser(r) => state
                .access_control_for_company_branch
                .upsert(row_uuid, |table| table.user_ = r),
            types::Resource::TableAccessControlForCompanyBranchFieldDataGroup(r) => state
                .access_control_for_company_branch
                .upsert(row_uuid, |table| table.data_group = r),
        }
    }
}

// all imples down

pub(crate) mod sign_up {
    use super::*;

    pub(crate) type Type1 = cases::sign_up::Input;
    type Type2 = cases::sign_up::Input;
    type Type3 = cases::sign_up::MyResult;
    pub(crate) type Type4 = cases::sign_up::MyResult;

    impl Into<Vec<types::ResourceInfo>> for &cases::sign_up::Ok {
        fn into(self) -> Vec<types::ResourceInfo> {
            let mut resource = Vec::with_capacity(3);

            resource.push(types::ResourceInfo {
                row_uuid: self.new_uuid.clone(),
                resource: types::Resource::Jwt(self.jwt.clone()),
            });

            resource.push(types::ResourceInfo {
                row_uuid: self.new_uuid.clone(),
                resource: types::Resource::TableUserFieldId(self.user_id.clone()),
            });

            if let Some(user_name) = &self.user_name {
                resource.push(types::ResourceInfo {
                    row_uuid: self.new_uuid.clone(),
                    resource: types::Resource::TableUserFieldName(user_name.clone()),
                });
            }

            resource
        }
    }

    impl ViewType1 for Type1 {
        fn wrap_input(self) -> request_response::push_data::OperationsInput {
            request_response::push_data::OperationsInput::SignUp(self)
        }
    }

    impl CacheAndServerType1 for Type2 {
        fn user_uuid(&self) -> Option<&types::UuidType> {
            Some(&self.new_uuid)
        }

        type Output = Type3;
        async fn state_full_operation<Id: cases::RowId, Ch: cache::Cache>(
            &self,
            state: &mut cache::State<Ch>,
        ) -> Self::Output {
            let (is_new_uuid_exist, is_user_id_exist) =
                state.read_sign_up(&self.new_uuid, &self.user_id).await;
            let errr = self.state_full_check::<Id>(is_new_uuid_exist, is_user_id_exist);
            if errr.is_there_error() {
                return Err(errr);
            }

            let result = cases::sign_up::Ok {
                new_uuid: self.new_uuid.clone(),
                user_id: self.user_id.clone(),
                user_name: self.name.clone(),
                hashed_password: String::new(),
                jwt: types::JsonWebTokenType(String::new()),
            };

            return Ok(result);
        }
    }

    impl CacheAndServerType2 for Type3 {
        fn extract_resource(&self) -> Vec<types::ResourceInfo> {
            match self {
                Ok(ok) => ok.into(),
                Err(_) => Vec::new(),
            }
        }

        fn wrap_output(self) -> request_response::push_data::OperationsResult {
            request_response::push_data::OperationsResult::SignUp(self)
        }
    }

    impl ViewType2 for Type4 {
        fn unwrap_output(result: request_response::push_data::OperationsResult) -> Self {
            if let request_response::push_data::OperationsResult::SignUp(result) = result {
                return result;
            }
            unreachable!("{:?}", result)
        }
    }
}

pub(crate) mod sign_in {
    use super::*;

    pub(crate) type Type1 = cases::sign_in::Input;
    type Type2 = cases::sign_in::Input;
    type Type3 = cases::sign_in::MyResult;
    pub(crate) struct Type4(pub(crate) Result<SignInOk, cases::sign_in::Error>);

    pub(crate) struct SignInOk {
        pub(crate) user_uuid: types::UuidType,
        pub(crate) user_name: String,
    }

    impl Into<Vec<types::ResourceInfo>> for &cases::sign_in::Ok {
        fn into(self) -> Vec<types::ResourceInfo> {
            use types::{Resource, ResourceInfo};

            let mut resources = Vec::with_capacity(3);
            let user_uuid = &self.user_uuid;

            // JWT
            resources.push(ResourceInfo {
                row_uuid: user_uuid.clone(),
                resource: Resource::Jwt(self.jwt.clone()),
            });

            // User ID
            resources.push(ResourceInfo {
                row_uuid: user_uuid.clone(),
                resource: Resource::TableUserFieldId(self.user_id.clone()),
            });

            // User name (optional)
            if let Some(name) = &self.user_name {
                resources.push(ResourceInfo {
                    row_uuid: user_uuid.clone(),
                    resource: Resource::TableUserFieldName(name.clone()),
                });
            }

            resources
        }
    }

    impl ViewType1 for Type1 {
        fn wrap_input(self) -> request_response::push_data::OperationsInput {
            request_response::push_data::OperationsInput::SignIn(self)
        }
    }

    impl CacheAndServerType1 for Type2 {
        fn user_uuid(&self) -> Option<&types::UuidType> {
            None
        }

        type Output = Type3;

        async fn state_full_operation<Id: cases::RowId, Ch: cache::Cache>(
            &self,
            state: &mut cache::State<Ch>,
        ) -> Self::Output {
            let user_uuid_and_is_jwt_exist = state.cache.read_sign_in(&self.user_id).await;

            if let Some((user_uuid, user_name, is_jwt_exist)) = user_uuid_and_is_jwt_exist {
                if is_jwt_exist {
                    return Ok(cases::sign_in::Ok {
                        user_uuid,
                        jwt: types::JsonWebTokenType(String::new()),
                        user_id: self.user_id.clone(),
                        user_name,
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
                        return Ok(self.state_full_operation(
                            &types::JsonWebTokenType(String::new()),
                            &user_uuid.unwrap(),
                            &user_name,
                        ));
                    } else {
                        return Err(cases::sign_in::Error {
                            user_id: None,
                            password: Some(cases::sign_in::PasswordError::WrongPassword),
                        });
                    }
                }
                None => Err(cases::sign_in::Error {
                    user_id: Some(cases::sign_in::UserIdError::NotExist),
                    password: None,
                }),
            }
        }
    }

    impl CacheAndServerType2 for Type3 {
        fn extract_resource(&self) -> Vec<types::ResourceInfo> {
            match self {
                Ok(ok) => ok.into(),
                Err(_) => Vec::new(),
            }
        }

        fn wrap_output(self) -> request_response::push_data::OperationsResult {
            request_response::push_data::OperationsResult::SignIn(self)
        }
    }

    impl ViewType2 for Type4 {
        fn unwrap_output(result: request_response::push_data::OperationsResult) -> Self {
            if let request_response::push_data::OperationsResult::SignIn(result) = result {
                match result {
                    Ok(ok) => Type4(Ok(SignInOk {
                        user_uuid: ok.user_uuid,
                        user_name: ok.user_name.unwrap_or_default(),
                    })),
                    Err(err) => Type4(Err(err)),
                }
            } else {
                unreachable!("{:?}", result)
            }
        }
    }
}

pub(crate) mod create_company {
    use super::*;

    pub(crate) type Type1 = cases::create_company::Input;
    type Type2 = cases::create_company::Input;
    type Type3 = cases::create_company::MyResult;
    pub(crate) type Type4 = cases::create_company::MyResult;

    impl Into<Vec<types::ResourceInfo>> for &cases::create_company::Ok {
        fn into(self) -> Vec<types::ResourceInfo> {
            let company_uuid = self.new_uuid.clone();

            vec![
                // Company fields
                types::ResourceInfo {
                    row_uuid: company_uuid.clone(),
                    resource: types::Resource::TableCompanyFieldName(self.company_name.clone()),
                },
                types::ResourceInfo {
                    row_uuid: company_uuid.clone(),
                    resource: types::Resource::TableCompanyFieldCurrency(self.currency.clone()),
                },
                // Access control fields (using the same UUID as the row identifier)
                types::ResourceInfo {
                    row_uuid: company_uuid.clone(),
                    resource: types::Resource::TableAccessControlForCompanyFieldRole(
                        self.role.clone(),
                    ),
                },
                types::ResourceInfo {
                    row_uuid: company_uuid.clone(),
                    resource: types::Resource::TableAccessControlForCompanyFieldUser(
                        self.user_uuid.clone(),
                    ),
                },
                types::ResourceInfo {
                    row_uuid: company_uuid.clone(),
                    resource: types::Resource::TableAccessControlForCompanyFieldDataGroup(
                        company_uuid,
                    ),
                },
            ]
        }
    }

    impl ViewType1 for Type1 {
        fn wrap_input(self) -> request_response::push_data::OperationsInput {
            request_response::push_data::OperationsInput::CreateCompany(self)
        }
    }

    impl CacheAndServerType1 for Type2 {
        fn user_uuid(&self) -> Option<&types::UuidType> {
            Some(&self.user_uuid)
        }

        type Output = Type3;

        async fn state_full_operation<Id: cases::RowId, Ch: cache::Cache>(
            &self,
            _: &mut cache::State<Ch>,
        ) -> Self::Output {
            let result = self.state_less_operation();
            return Ok(result);
        }
    }

    impl CacheAndServerType2 for Type3 {
        fn extract_resource(&self) -> Vec<types::ResourceInfo> {
            match self {
                Ok(ok) => ok.into(),
                Err(_) => Vec::new(),
            }
        }

        fn wrap_output(self) -> request_response::push_data::OperationsResult {
            request_response::push_data::OperationsResult::CreateCompany(self)
        }
    }

    impl ViewType2 for Type4 {
        fn unwrap_output(result: request_response::push_data::OperationsResult) -> Self {
            if let request_response::push_data::OperationsResult::CreateCompany(result) = result {
                return result;
            }
            unreachable!("{:?}", result)
        }
    }
}

pub(crate) mod create_company_branch {
    use super::*;

    pub(crate) type Type1 = cases::create_company_branch::Input;
    type Type2 = cases::create_company_branch::Input;
    type Type3 = cases::create_company_branch::MyResult;
    pub(crate) type Type4 = cases::create_company_branch::MyResult;

    impl Into<Vec<types::ResourceInfo>> for &cases::create_company_branch::Ok {
        fn into(self) -> Vec<types::ResourceInfo> {
            let branch_uuid = self.new_uuid.clone();

            vec![
                // Branch fields
                types::ResourceInfo {
                    row_uuid: branch_uuid.clone(),
                    resource: types::Resource::TableCompanyBranchFieldName(
                        self.branch_name.clone(),
                    ),
                },
                types::ResourceInfo {
                    row_uuid: branch_uuid.clone(),
                    resource: types::Resource::TableCompanyBranchFieldCompanyBelong(
                        self.company_belong.clone(),
                    ),
                },
                types::ResourceInfo {
                    row_uuid: branch_uuid.clone(),
                    resource: types::Resource::TableCompanyBranchFieldLocation(
                        self.location.clone(),
                    ),
                },
                types::ResourceInfo {
                    row_uuid: branch_uuid.clone(),
                    resource: types::Resource::TableCompanyBranchFieldCurrency(
                        self.currency.clone(),
                    ),
                },
                // Access control for this branch (row_uuid is the branch UUID)
                types::ResourceInfo {
                    row_uuid: branch_uuid.clone(),
                    resource: types::Resource::TableAccessControlForCompanyBranchFieldRole(
                        self.role.clone(),
                    ),
                },
                types::ResourceInfo {
                    row_uuid: branch_uuid.clone(),
                    resource: types::Resource::TableAccessControlForCompanyBranchFieldUser(
                        self.user_uuid.clone(),
                    ),
                },
                types::ResourceInfo {
                    row_uuid: branch_uuid,
                    resource: types::Resource::TableAccessControlForCompanyBranchFieldDataGroup(
                        self.new_uuid.clone(),
                    ),
                },
            ]
        }
    }

    impl ViewType1 for Type1 {
        fn wrap_input(self) -> request_response::push_data::OperationsInput {
            request_response::push_data::OperationsInput::CreateCompanyBranch(self)
        }
    }

    impl CacheAndServerType1 for Type2 {
        fn user_uuid(&self) -> Option<&types::UuidType> {
            Some(&self.user_uuid)
        }

        type Output = Type3;

        async fn state_full_operation<Id: cases::RowId, Ch: cache::Cache>(
            &self,
            state: &mut cache::State<Ch>,
        ) -> Self::Output {
            let (user_roles, is_company_belong_exist, is_branch_name_used) = state
                .read_create_company_branch(
                    &self.user_uuid,
                    &self.company_belong,
                    &self.branch_name,
                )
                .await;

            let errr = self.state_full_check::<Id>(
                &user_roles,
                false,
                is_company_belong_exist,
                is_branch_name_used,
            );
            if errr.is_there_error() {
                return Err(errr);
            }

            let result = self.state_less_operation();

            return Ok(result);
        }
    }

    impl CacheAndServerType2 for Type3 {
        fn extract_resource(&self) -> Vec<types::ResourceInfo> {
            match self {
                Ok(ok) => ok.into(),
                Err(_) => Vec::new(),
            }
        }

        fn wrap_output(self) -> request_response::push_data::OperationsResult {
            request_response::push_data::OperationsResult::CreateCompanyBranch(self)
        }
    }

    impl ViewType2 for Type4 {
        fn unwrap_output(result: request_response::push_data::OperationsResult) -> Self {
            if let request_response::push_data::OperationsResult::CreateCompanyBranch(result) =
                result
            {
                return result;
            }
            unreachable!("{:?}", result)
        }
    }
}

pub(crate) mod list_company_and_branch {
    use super::*;
    use std::cmp::Ordering;

    pub(crate) type Type1 = cases::list_company_and_branch::Input;
    type Type2 = cases::list_company_and_branch::Input;
    type Type3 = cases::list_company_and_branch::MyResult;
    pub(crate) struct Type4(pub(crate) Result<types::ListOfCompanies, ()>);

    /// Sort a list of companies by name then by UUID, and sort branches inside each company similarly.
    pub fn sort_companies(companies: &mut types::ListOfCompanies) {
        companies.sort_by(|a, b| compare_by_name_then_uuid(&a.name, &a.uuid, &b.name, &b.uuid));
        for company in companies {
            company
                .branches
                .sort_by(|a, b| compare_by_name_then_uuid(&a.name, &a.uuid, &b.name, &b.uuid));
        }
    }

    /// Helper that compares two items by name (lexicographically) and, if equal, by UUID.
    fn compare_by_name_then_uuid(
        name_a: &str,
        uuid_a: &types::UuidType,
        name_b: &str,
        uuid_b: &types::UuidType,
    ) -> Ordering {
        match name_a.cmp(name_b) {
            Ordering::Equal => uuid_a.cmp(uuid_b),
            other => other,
        }
    }

    impl Into<Vec<types::ResourceInfo>> for &cases::list_company_and_branch::Ok {
        fn into(self) -> Vec<types::ResourceInfo> {
            use types::{Resource, ResourceInfo};

            let mut resources = Vec::new();
            let user_uuid = &self.user_uuid;

            for company in &self.data {
                let company_uuid = &company.company_uuid;

                // ---- Company fields ----
                resources.push(ResourceInfo {
                    row_uuid: company_uuid.clone(),
                    resource: Resource::TableCompanyFieldName(company.company_name.clone()),
                });
                resources.push(ResourceInfo {
                    row_uuid: company_uuid.clone(),
                    resource: Resource::TableCompanyFieldCurrency(company.company_currancy.clone()),
                });

                // ---- Company access control ----
                // One resource per role (multiple roles possible)
                for role in &company.user_roles {
                    resources.push(ResourceInfo {
                        row_uuid: company_uuid.clone(),
                        resource: Resource::TableAccessControlForCompanyFieldRole(role.clone()),
                    });
                }
                // Always add the user and data_group (self) once per company
                resources.push(ResourceInfo {
                    row_uuid: company_uuid.clone(),
                    resource: Resource::TableAccessControlForCompanyFieldUser(user_uuid.clone()),
                });
                resources.push(ResourceInfo {
                    row_uuid: company_uuid.clone(),
                    resource: Resource::TableAccessControlForCompanyFieldDataGroup(
                        company_uuid.clone(),
                    ),
                });

                // ---- Branches ----
                for branch in &company.branches {
                    let branch_uuid = &branch.branch_uuid;

                    resources.push(ResourceInfo {
                        row_uuid: branch_uuid.clone(),
                        resource: Resource::TableCompanyBranchFieldName(branch.branch_name.clone()),
                    });
                    resources.push(ResourceInfo {
                        row_uuid: branch_uuid.clone(),
                        resource: Resource::TableCompanyBranchFieldCurrency(
                            branch.branch_currancy.clone(),
                        ),
                    });
                    resources.push(ResourceInfo {
                        row_uuid: branch_uuid.clone(),
                        resource: Resource::TableCompanyBranchFieldCompanyBelong(
                            company_uuid.clone(),
                        ),
                    });

                    // Branch access control (roles)
                    for role in &branch.user_roles {
                        resources.push(ResourceInfo {
                            row_uuid: branch_uuid.clone(),
                            resource: Resource::TableAccessControlForCompanyBranchFieldRole(
                                role.clone(),
                            ),
                        });
                    }
                    // Add user and data_group for each branch
                    resources.push(ResourceInfo {
                        row_uuid: branch_uuid.clone(),
                        resource: Resource::TableAccessControlForCompanyBranchFieldUser(
                            user_uuid.clone(),
                        ),
                    });
                    resources.push(ResourceInfo {
                        row_uuid: branch_uuid.clone(),
                        resource: Resource::TableAccessControlForCompanyBranchFieldDataGroup(
                            branch_uuid.clone(),
                        ),
                    });
                }
            }

            resources
        }
    }

    impl ViewType1 for Type1 {
        fn subs() -> &'static [types::Subscribe] {
            &[
                types::Subscribe::TableCompanyBranchFieldName,
                types::Subscribe::TableCompanyFieldName,
                types::Subscribe::TableAccessControlForCompanyFieldRole,
            ]
        }

        fn wrap_input(self) -> request_response::push_data::OperationsInput {
            request_response::push_data::OperationsInput::ListCompanyAndBranch(self)
        }
    }

    impl CacheAndServerType1 for Type2 {
        fn user_uuid(&self) -> Option<&types::UuidType> {
            Some(&self.user_uuid)
        }

        type Output = Type3;

        async fn state_full_operation<Id: cases::RowId, Ch: cache::Cache>(
            &self,
            state: &mut cache::State<Ch>,
        ) -> Self::Output {
            let result = state.read_list_company_and_branch(&self.user_uuid).await;
            return Ok(cases::list_company_and_branch::Ok {
                user_uuid: self.user_uuid.clone(),
                data: result,
            });
        }
    }

    impl CacheAndServerType2 for Type3 {
        fn extract_resource(&self) -> Vec<types::ResourceInfo> {
            match self {
                Ok(ok) => ok.into(),
                Err(_) => Vec::new(),
            }
        }

        fn wrap_output(self) -> request_response::push_data::OperationsResult {
            request_response::push_data::OperationsResult::ListCompanyAndBranch(self)
        }
    }

    impl ViewType2 for Type4 {
        fn unwrap_output(result: request_response::push_data::OperationsResult) -> Self {
            if let request_response::push_data::OperationsResult::ListCompanyAndBranch(res) = result
            {
                match res {
                    Ok(ok) => {
                        let mut companies = Vec::with_capacity(ok.data.len());

                        for company_entry in ok.data {
                            // Convert branches for this company
                            let branches = company_entry
                                .branches
                                .into_iter()
                                .map(|branch_entry| types::Branch {
                                    uuid: branch_entry.branch_uuid,
                                    name: branch_entry.branch_name,
                                })
                                .collect();

                            // Pick a single role (e.g., the first one, or highest privilege)
                            // If no role, provide a sensible default (adjust as needed)
                            let role = company_entry
                                .user_roles
                                .first()
                                .cloned()
                                .unwrap_or_default();

                            companies.push(types::Company {
                                uuid: company_entry.company_uuid,
                                name: company_entry.company_name,
                                role,
                                branches,
                            });
                        }

                        sort_companies(&mut companies);

                        Type4(Ok(companies))
                    }
                    Err(_) => Type4(Err(())),
                }
            } else {
                unreachable!("Expected ListCompanyAndBranch, got {:?}", result)
            }
        }
    }
}
