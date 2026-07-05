use crate::{
    accounting_client::{
        cache, cache_actor,
        client_traits::{AllClientTypes, Cache},
    },
    accounting_domain::{cases, request_response, types},
    utility::utils::MyUpSert,
};
use std::collections::{HashMap, HashSet};

pub(crate) trait ViewType1 {
    fn subs() -> &'static [types::Subscribe] {
        unreachable!("we dont need it here")
    }
    fn wrap_input(self) -> request_response::push_data::OperationsInput;
}

pub(crate) trait CacheAndServerType1: Clone {
    fn user_uuid(&self) -> Option<&types::UuidType>;

    type Output: CacheAndServerType2;
    async fn state_full_operation<At: AllClientTypes>(
        &self,
        state: &mut cache::State<At>,
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
    pub(crate) async fn run_operation_check<At: AllClientTypes>(
        &self,
        state: &mut cache::State<At>,
    ) -> request_response::push_data::OperationsResult {
        match self {
            request_response::push_data::OperationsInput::SignUp(i) => {
                operation_check_handler::<_, At>(i, state).await
            }
            request_response::push_data::OperationsInput::SignIn(i) => {
                operation_check_handler::<_, At>(i, state).await
            }
            request_response::push_data::OperationsInput::CreateCompany(i) => {
                operation_check_handler::<_, At>(i, state).await
            }
            request_response::push_data::OperationsInput::CreateCompanyBranch(i) => {
                operation_check_handler::<_, At>(i, state).await
            }
            request_response::push_data::OperationsInput::ListCompanyAndBranch(i) => {
                operation_check_handler::<_, At>(i, state).await
            }
        }
    }

    pub(crate) async fn run_operation_check_apply<At: AllClientTypes>(
        &self,
        state: &mut cache::State<At>,
        subs_to_poke: &mut HashSet<types::Subscribe>,
    ) {
        match self {
            request_response::push_data::OperationsInput::SignUp(i) => {
                operation_check_apply_handler::<_, At>(i, state, subs_to_poke).await
            }
            request_response::push_data::OperationsInput::SignIn(i) => {
                operation_check_apply_handler::<_, At>(i, state, subs_to_poke).await
            }
            request_response::push_data::OperationsInput::CreateCompany(i) => {
                operation_check_apply_handler::<_, At>(i, state, subs_to_poke).await
            }
            request_response::push_data::OperationsInput::CreateCompanyBranch(i) => {
                operation_check_apply_handler::<_, At>(i, state, subs_to_poke).await
            }
            request_response::push_data::OperationsInput::ListCompanyAndBranch(i) => {
                operation_check_apply_handler::<_, At>(i, state, subs_to_poke).await
            }
        }
    }

    pub(crate) async fn run_operation_check_apply_write<At: AllClientTypes>(
        &self,
        txn_number: u64,
        state: &mut cache::State<At>,
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
                operation_check_apply_write_handler::<_, At>(i, state, subs_to_poke).await
            }
            request_response::push_data::OperationsInput::SignIn(i) => {
                operation_check_apply_write_handler::<_, At>(i, state, subs_to_poke).await
            }
            request_response::push_data::OperationsInput::CreateCompany(i) => {
                operation_check_apply_write_handler::<_, At>(i, state, subs_to_poke).await
            }
            request_response::push_data::OperationsInput::CreateCompanyBranch(i) => {
                operation_check_apply_write_handler::<_, At>(i, state, subs_to_poke).await
            }
            request_response::push_data::OperationsInput::ListCompanyAndBranch(i) => {
                operation_check_apply_write_handler::<_, At>(i, state, subs_to_poke).await
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

async fn operation_check_handler<T: CacheAndServerType1, At: AllClientTypes>(
    input: &T,
    state: &mut cache::State<At>,
) -> request_response::push_data::OperationsResult {
    return input.state_full_operation::<At>(state).await.wrap_output();
}

async fn operation_check_apply_handler<T: CacheAndServerType1, At: AllClientTypes>(
    input: &T,
    state: &mut cache::State<At>,
    subs_to_poke: &mut HashSet<types::Subscribe>,
) {
    apply_change(
        input
            .state_full_operation::<At>(state)
            .await
            .extract_resource(),
        &mut state.state_of_pending_txn,
        subs_to_poke,
    )
    .await;
}

async fn operation_check_apply_write_handler<T: CacheAndServerType1, At: AllClientTypes>(
    input: &T,
    state: &mut cache::State<At>,
    subs_to_poke: &mut HashSet<types::Subscribe>,
) -> request_response::push_data::OperationsResult {
    let result = input.state_full_operation::<At>(state).await;

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
    state: &mut cache::StateOfPendingTxn,
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
        async fn state_full_operation<At: AllClientTypes>(
            &self,
            state: &mut cache::State<At>,
        ) -> Self::Output {
            let result = todo!();

            return result;
        }
    }

    impl CacheAndServerType2 for Type3 {
        fn extract_resource(&self) -> Vec<types::ResourceInfo> {
            match self {
                Ok(ok) => todo!("ok.clone().into()"),
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
        pub user_uuid: types::UuidType,
        pub user_name: String,
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

        async fn state_full_operation<At: AllClientTypes>(
            &self,
            state: &mut cache::State<At>,
        ) -> Self::Output {
            let user_uuid_and_is_jwt_exist = state.cache.read_sign_in(&self.user_id).await;

            if let Some((user_uuid, user_name, is_jwt_exist)) = user_uuid_and_is_jwt_exist {
                if is_jwt_exist {
                    return Ok(cases::sign_in::Ok {
                        user_uuid,
                        jwt: types::JsonWebTokenType(String::new()),
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
                        return Ok(cases::sign_in::Ok {
                            user_uuid: user_uuid.unwrap().clone(),
                            jwt: types::JsonWebTokenType(String::new()),
                        });
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
                Ok(ok) => todo!("ok.clone().into()"),
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
                    Ok(ok) => {
                        let mut user_uuid = ok.user_uuid;
                        let mut user_name = todo!();

                        Type4(Ok(SignInOk {
                            user_uuid,
                            user_name,
                        }))
                    }
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

        async fn state_full_operation<At: AllClientTypes>(
            &self,
            state: &mut cache::State<At>,
        ) -> Self::Output {
            let result = todo!();

            return result;
        }
    }

    impl CacheAndServerType2 for Type3 {
        fn extract_resource(&self) -> Vec<types::ResourceInfo> {
            match self {
                Ok(ok) => todo!("ok.clone().into()"),
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

        async fn state_full_operation<At: AllClientTypes>(
            &self,
            state: &mut cache::State<At>,
        ) -> Self::Output {
            let result = todo!();

            return result;
        }
    }

    impl CacheAndServerType2 for Type3 {
        fn extract_resource(&self) -> Vec<types::ResourceInfo> {
            match self {
                Ok(ok) => todo!("ok.clone().into()"),
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

    pub(crate) type Type1 = cases::list_company_and_branch::Input;
    type Type2 = cases::list_company_and_branch::Input;
    type Type3 = cases::list_company_and_branch::MyResult;
    pub(crate) struct Type4(pub(crate) Result<types::ListOfCompanies, ()>);

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

        async fn state_full_operation<At: AllClientTypes>(
            &self,
            state: &mut cache::State<At>,
        ) -> Self::Output {
            let result = todo!();

            return result;
        }
    }

    impl CacheAndServerType2 for Type3 {
        fn extract_resource(&self) -> Vec<types::ResourceInfo> {
            match self {
                Ok(ok) => todo!("ok.clone().into()"),
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
                        #[derive(Default)]
                        struct CompanyData {
                            name: String,
                            currency: types::Currency,
                            role: types::Role,
                        }

                        #[derive(Default)]
                        struct BranchData {
                            name: String,
                            company_belong: types::UuidType,
                        }

                        let resources = ok.resource;
                        let mut company_data: HashMap<types::UuidType, CompanyData> =
                            HashMap::new();
                        let mut branch_data: HashMap<types::UuidType, BranchData> = HashMap::new();

                        for r in resources {
                            let uuid = r.row_uuid.clone();
                            match r.resource {
                                types::Resource::TableCompanyFieldName(name) => {
                                    company_data.upsert(uuid, |data| data.name = name);
                                }
                                types::Resource::TableCompanyFieldCurrency(currency) => {
                                    company_data.upsert(uuid, |data| data.currency = currency);
                                }
                                types::Resource::TableAccessControlForCompanyFieldRole(role) => {
                                    company_data.upsert(uuid, |data| data.role = role);
                                }
                                types::Resource::TableCompanyBranchFieldName(name) => {
                                    branch_data.upsert(uuid, |data| data.name = name);
                                }
                                types::Resource::TableCompanyBranchFieldCompanyBelong(
                                    company_uuid,
                                ) => {
                                    branch_data
                                        .upsert(uuid, |data| data.company_belong = company_uuid);
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
                                        Some(types::Branch {
                                            uuid: branch_uuid.clone(),
                                            name: branch.name.clone(),
                                        })
                                    } else {
                                        None
                                    }
                                })
                                .collect();

                            companies.push(types::Company {
                                uuid,
                                name: data.name,
                                role: data.role,
                                branches,
                            });
                        }

                        Type4(Ok(companies))
                    }
                    Err(_) => Type4(Err(())),
                }
            } else {
                unreachable!("{:?}", result)
            }
        }
    }
}
