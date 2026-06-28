use crate::cache::State;
use crate::decider::EventMaker;
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
    async fn state_full_operation<At: AllClientTypes>(
        &self,
        state: &mut cache::State<At>,
    ) -> Self::Output;
}

pub(crate) trait CacheAndServerType2 {
    fn extract_resource(&self) -> Vec<ResourceInfo>;
    fn wrap_output(self) -> push_data::OperationsResult;
}

pub(crate) trait ViewType2 {
    fn unwrap_output(output: push_data::OperationsResult) -> Self;
}

impl push_data::OperationsInput {
    pub(crate) async fn run_operation_check<At: AllClientTypes>(
        &self,
        state: &mut cache::State<At>,
    ) -> push_data::OperationsResult {
        match self {
            push_data::OperationsInput::SignUp(i) => {
                operation_check_handler::<_, At>(i, state).await
            }
            push_data::OperationsInput::SignIn(i) => {
                operation_check_handler::<_, At>(i, state).await
            }
            push_data::OperationsInput::CreateCompany(i) => {
                operation_check_handler::<_, At>(i, state).await
            }
            push_data::OperationsInput::CreateCompanyBranch(i) => {
                operation_check_handler::<_, At>(i, state).await
            }
            push_data::OperationsInput::ListCompanyAndBranch(i) => {
                operation_check_handler::<_, At>(i, state).await
            }
        }
    }

    pub(crate) async fn run_operation_check_apply<At: AllClientTypes>(
        &self,
        state: &mut cache::State<At>,
        subs_to_poke: &mut HashSet<server_methods::Subscribe>,
    ) {
        match self {
            push_data::OperationsInput::SignUp(i) => {
                operation_check_apply_handler::<_, At>(i, state, subs_to_poke).await
            }
            push_data::OperationsInput::SignIn(i) => {
                operation_check_apply_handler::<_, At>(i, state, subs_to_poke).await
            }
            push_data::OperationsInput::CreateCompany(i) => {
                operation_check_apply_handler::<_, At>(i, state, subs_to_poke).await
            }
            push_data::OperationsInput::CreateCompanyBranch(i) => {
                operation_check_apply_handler::<_, At>(i, state, subs_to_poke).await
            }
            push_data::OperationsInput::ListCompanyAndBranch(i) => {
                operation_check_apply_handler::<_, At>(i, state, subs_to_poke).await
            }
        }
    }

    pub(crate) async fn run_operation_check_apply_write<At: AllClientTypes>(
        &self,
        txn_number: u64,
        state: &mut cache::State<At>,
        subs_to_poke: &mut HashSet<server_methods::Subscribe>,
    ) -> push_data::OperationsResult {
        state
            .cache
            .write_txn_input(&push_data::Txn {
                txn_number,
                operation: self.clone(),
            })
            .await;

        match self {
            push_data::OperationsInput::SignUp(i) => {
                operation_check_apply_write_handler::<_, At>(i, state, subs_to_poke).await
            }
            push_data::OperationsInput::SignIn(i) => {
                operation_check_apply_write_handler::<_, At>(i, state, subs_to_poke).await
            }
            push_data::OperationsInput::CreateCompany(i) => {
                operation_check_apply_write_handler::<_, At>(i, state, subs_to_poke).await
            }
            push_data::OperationsInput::CreateCompanyBranch(i) => {
                operation_check_apply_write_handler::<_, At>(i, state, subs_to_poke).await
            }
            push_data::OperationsInput::ListCompanyAndBranch(i) => {
                operation_check_apply_write_handler::<_, At>(i, state, subs_to_poke).await
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

async fn operation_check_handler<T: CacheAndServerType1, At: AllClientTypes>(
    input: &T,
    state: &mut cache::State<At>,
) -> push_data::OperationsResult {
    return input.state_full_operation::<At>(state).await.wrap_output();
}

async fn operation_check_apply_handler<T: CacheAndServerType1, At: AllClientTypes>(
    input: &T,
    state: &mut cache::State<At>,
    subs_to_poke: &mut HashSet<server_methods::Subscribe>,
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
    subs_to_poke: &mut HashSet<server_methods::Subscribe>,
) -> push_data::OperationsResult {
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
    resources: Vec<ResourceInfo>,
    state: &mut cache::StateOfPendingTxn,
    subs_to_poke: &mut HashSet<server_methods::Subscribe>,
) {
    cache_actor::collect_subs_to_poke(subs_to_poke, &resources);

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
            server_methods::Resource::TableCompanyBranchFieldCurrency(r) => state
                .company_branch
                .upsert(row_uuid, |table| table.currency = r),
            server_methods::Resource::TableCompanyBranchFieldLocation(r) => state
                .company_branch
                .upsert(row_uuid, |table| table.location = r),
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
            server_methods::Resource::TableAccessControlForCompanyBranchFieldRole(r) => state
                .access_control_for_company_branch
                .upsert(row_uuid, |table| table.role = r),
            server_methods::Resource::TableAccessControlForCompanyBranchFieldUser(r) => state
                .access_control_for_company_branch
                .upsert(row_uuid, |table| table.user_ = r),
            server_methods::Resource::TableAccessControlForCompanyBranchFieldDataGroup(r) => state
                .access_control_for_company_branch
                .upsert(row_uuid, |table| table.data_group = r),
        }
    }
}

// all imples down

struct SodoJwt;
impl JWT for SodoJwt {
    fn new() -> Self {
        Self
    }

    fn sign(&self, user_uuid: &db_types::UuidType) -> db_types::JsonWebTokenType {
        db_types::JsonWebTokenType(String::new())
    }

    fn validate(&self, token: db_types::JsonWebTokenType) -> Option<db_types::UuidType> {
        unreachable!("this is not callable at client side")
    }
}

struct SodoAuth;
impl HashedPassword for SodoAuth {
    fn sign_up(password: &String) -> String {
        password.clone()
    }

    fn sign_in(password: &String, password_hash: &String) -> bool {
        unreachable!("this is not callable at client side")
    }
}

pub(crate) mod sign_up {
    use super::*;

    pub(crate) type Type1 = decider::sign_up::Input;
    type Type2 = decider::sign_up::Input;
    type Type3 = decider::sign_up::Result;
    pub(crate) type Type4 = decider::sign_up::Result;

    impl ViewType1 for Type1 {
        fn wrap_input(self) -> push_data::OperationsInput {
            push_data::OperationsInput::SignUp(self)
        }
    }

    impl CacheAndServerType1 for Type2 {
        fn user_uuid(&self) -> Option<&db_types::UuidType> {
            Some(&self.new_uuid)
        }

        type Output = Type3;
        async fn state_full_operation<At: AllClientTypes>(
            &self,
            state: &mut cache::State<At>,
        ) -> Self::Output {
            let result = self
                .handle::<State<_>, At::Rn, At::Rt, At::Id, At::Mpsc, At::Ed, At::Rg, SodoAuth, SodoJwt>(&mut server_methods::SideEffects::default(), state, &SodoJwt)
                .await
                .unwrap();

            return result;
        }
    }

    impl CacheAndServerType2 for Type3 {
        fn extract_resource(&self) -> Vec<ResourceInfo> {
            match self {
                Ok(ok) => ok.clone().into(),
                Err(_) => Vec::new(),
            }
        }

        fn wrap_output(self) -> push_data::OperationsResult {
            push_data::OperationsResult::SignUp(self)
        }
    }

    impl ViewType2 for Type4 {
        fn unwrap_output(result: push_data::OperationsResult) -> Self {
            if let push_data::OperationsResult::SignUp(result) = result {
                return result;
            }
            unreachable!("{:?}", result)
        }
    }
}

pub(crate) mod sign_in {
    use super::*;

    pub(crate) type Type1 = decider::sign_in::Input;
    type Type2 = decider::sign_in::Input;
    type Type3 = decider::sign_in::Result;
    pub(crate) struct Type4(pub(crate) Result<SignInOk, decider::sign_in::Error>);

    pub(crate) struct SignInOk {
        pub user_uuid: db_types::UuidType,
        pub user_name: String,
    }

    impl ViewType1 for Type1 {
        fn wrap_input(self) -> push_data::OperationsInput {
            push_data::OperationsInput::SignIn(self)
        }
    }

    impl CacheAndServerType1 for Type2 {
        fn user_uuid(&self) -> Option<&db_types::UuidType> {
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
                    return Ok(decider::sign_in::Ok {
                        user_uuid,
                        jwt: db_types::JsonWebTokenType(String::new()),
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
                        return Ok(decider::sign_in::Ok {
                            user_uuid: user_uuid.unwrap().clone(),
                            jwt: db_types::JsonWebTokenType(String::new()),
                        });
                    } else {
                        return Err(decider::sign_in::Error {
                            user_id: None,
                            password: Some(decider::sign_in::PasswordError::WrongPassword),
                        });
                    }
                }
                None => Err(decider::sign_in::Error {
                    user_id: Some(decider::sign_in::UserIdError::NotExist),
                    password: None,
                }),
            }
        }
    }

    impl CacheAndServerType2 for Type3 {
        fn extract_resource(&self) -> Vec<ResourceInfo> {
            match self {
                Ok(ok) => ok.clone().into(),
                Err(_) => Vec::new(),
            }
        }

        fn wrap_output(self) -> push_data::OperationsResult {
            push_data::OperationsResult::SignIn(self)
        }
    }

    impl ViewType2 for Type4 {
        fn unwrap_output(result: push_data::OperationsResult) -> Self {
            if let push_data::OperationsResult::SignIn(result) = result {
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

    pub(crate) type Type1 = decider::create_company::Input;
    type Type2 = decider::create_company::Input;
    type Type3 = decider::create_company::Result;
    pub(crate) type Type4 = decider::create_company::Result;

    impl ViewType1 for Type1 {
        fn wrap_input(self) -> push_data::OperationsInput {
            push_data::OperationsInput::CreateCompany(self)
        }
    }

    impl CacheAndServerType1 for Type2 {
        fn user_uuid(&self) -> Option<&db_types::UuidType> {
            Some(&self.user_uuid)
        }

        type Output = Type3;

        async fn state_full_operation<At: AllClientTypes>(
            &self,
            state: &mut cache::State<At>,
        ) -> Self::Output {
            let result = self
                .handle::<State<_>, At::Rn, At::Rt, At::Id, At::Mpsc, At::Ed, At::Rg, SodoAuth, SodoJwt>(&mut server_methods::SideEffects::default(), state, &SodoJwt)
                .await
                .unwrap();

            return result;
        }
    }

    impl CacheAndServerType2 for Type3 {
        fn extract_resource(&self) -> Vec<ResourceInfo> {
            match self {
                Ok(ok) => ok.clone().into(),
                Err(_) => Vec::new(),
            }
        }

        fn wrap_output(self) -> push_data::OperationsResult {
            push_data::OperationsResult::CreateCompany(self)
        }
    }

    impl ViewType2 for Type4 {
        fn unwrap_output(result: push_data::OperationsResult) -> Self {
            if let push_data::OperationsResult::CreateCompany(result) = result {
                return result;
            }
            unreachable!("{:?}", result)
        }
    }
}

pub(crate) mod create_company_branch {
    use super::*;

    pub(crate) type Type1 = decider::create_company_branch::Input;
    type Type2 = decider::create_company_branch::Input;
    type Type3 = decider::create_company_branch::Result;
    pub(crate) type Type4 = decider::create_company_branch::Result;

    impl ViewType1 for Type1 {
        fn wrap_input(self) -> push_data::OperationsInput {
            push_data::OperationsInput::CreateCompanyBranch(self)
        }
    }

    impl CacheAndServerType1 for Type2 {
        fn user_uuid(&self) -> Option<&db_types::UuidType> {
            Some(&self.user_uuid)
        }

        type Output = Type3;

        async fn state_full_operation<At: AllClientTypes>(
            &self,
            state: &mut cache::State<At>,
        ) -> Self::Output {
            let result = self
                .handle::<State<_>, At::Rn, At::Rt, At::Id, At::Mpsc, At::Ed, At::Rg, SodoAuth, SodoJwt>(&mut server_methods::SideEffects::default(), state, &SodoJwt)
                .await
                .unwrap();

            return result;
        }
    }

    impl CacheAndServerType2 for Type3 {
        fn extract_resource(&self) -> Vec<ResourceInfo> {
            match self {
                Ok(ok) => ok.clone().into(),
                Err(_) => Vec::new(),
            }
        }

        fn wrap_output(self) -> push_data::OperationsResult {
            push_data::OperationsResult::CreateCompanyBranch(self)
        }
    }

    impl ViewType2 for Type4 {
        fn unwrap_output(result: push_data::OperationsResult) -> Self {
            if let push_data::OperationsResult::CreateCompanyBranch(result) = result {
                return result;
            }
            unreachable!("{:?}", result)
        }
    }
}

pub(crate) mod list_company_and_branch {
    use super::*;

    pub(crate) type Type1 = decider::list_company_and_branch::Input;
    type Type2 = decider::list_company_and_branch::Input;
    type Type3 = decider::list_company_and_branch::Result;
    pub(crate) struct Type4(pub(crate) Result<db_types::ListOfCompanies, ()>);

    impl ViewType1 for Type1 {
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

    impl CacheAndServerType1 for Type2 {
        fn user_uuid(&self) -> Option<&db_types::UuidType> {
            Some(&self.user_uuid)
        }

        type Output = Type3;

        async fn state_full_operation<At: AllClientTypes>(
            &self,
            state: &mut cache::State<At>,
        ) -> Self::Output {
            let result = self
                .handle::<State<_>, At::Rn, At::Rt, At::Id, At::Mpsc, At::Ed, At::Rg, SodoAuth, SodoJwt>(&mut server_methods::SideEffects::default(), state, &SodoJwt)
                .await
                .unwrap();

            return result;
        }
    }

    impl CacheAndServerType2 for Type3 {
        fn extract_resource(&self) -> Vec<ResourceInfo> {
            match self {
                Ok(ok) => ok.clone().into(),
                Err(_) => Vec::new(),
            }
        }

        fn wrap_output(self) -> push_data::OperationsResult {
            push_data::OperationsResult::ListCompanyAndBranch(self)
        }
    }

    impl ViewType2 for Type4 {
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
                        let mut company_data: HashMap<db_types::UuidType, CompanyData> =
                            HashMap::new();
                        let mut branch_data: HashMap<db_types::UuidType, BranchData> =
                            HashMap::new();

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
