use crate::accounting_domain::{
    cases::{
        self,
        utility::{resource_utils, types},
    },
    request_response,
};

pub trait Cache: Sized {
    fn new() -> impl Future<Output = Self>;

    fn get_all_txn_input(
        &self,
    ) -> impl Future<
        Output = Vec<
            request_response::push_data::Txn<request_response::push_data::OperationsInput>,
        >,
    >;
    fn write_txn_input(
        &self,
        txn: &request_response::push_data::Txn<request_response::push_data::OperationsInput>,
    ) -> impl Future<Output = ()>;
    fn write_txn_result(
        &self,
        txn: &request_response::push_data::Txn<request_response::push_data::OperationsResult>,
    ) -> impl Future<Output = ()>;
    fn mark_txn_input_as_faild(&self, txn_number: &u64) -> impl Future<Output = ()>;
    fn delete_txn_input(&self, txn_number: &u64) -> impl Future<Output = ()>;

    fn write_resource(
        &self,
        resource: &Vec<resource_utils::ResourceInfo>,
    ) -> impl Future<Output = ()>;
    fn get_jwt(
        &self,
        user_uuid: &types::UuidType,
    ) -> impl Future<Output = Option<types::JsonWebTokenType>>;

    fn read_sign_up(
        &self,
        new_uuid: &types::UuidType,
        user_id: &String,
    ) -> impl Future<
        Output = (
            bool, /* is new_uuid exist */
            bool, /* is user_id exist */
        ),
    >;
    fn read_sign_in(
        &self,
        user_id: &String,
    ) -> impl Future<
        Output = Option<(
            types::UuidType, /* user uuid */
            Option<String>,  /* user name */
            bool,            /* does he have jwt */
        )>,
    >;
    fn read_list_company_and_branch(
        &self,
        user_uuid: &types::UuidType,
    ) -> impl Future<Output = Vec<cases::list_company_and_branch::AllCompaniesThatUserInWithRoles>>;
    fn read_create_company_branch(
        &self,
        user_uuid: &types::UuidType,
        company_belong: &types::UuidType,
        company_branch_name: &String,
    ) -> impl Future<
        Output = (
            Vec<types::Role>, /* roles at company */
            bool,             /* is company exist */
            bool,             /* is branch name used */
        ),
    >;
    fn read_create_account(
        &self,
        read_input: &cases::create_account::ReadInput,
    ) -> impl Future<Output = cases::create_account::ReadOutput>;
}

pub(crate) struct State<Ch: Cache> {
    pub(crate) state_of_pending_txn: resource_utils::StateOfPendingTxn,
    pub(crate) cache: Ch,
}

impl<Ch: Cache> State<Ch> {
    pub(crate) async fn read_sign_up(
        &mut self,
        new_uuid: &types::UuidType,
        user_id: &String,
    ) -> (
        bool, /* is new_uuid exist */
        bool, /* is user_id exist */
    ) {
        let (mut is_new_uuid_exist, mut is_user_id_exist) =
            self.cache.read_sign_up(new_uuid, user_id).await;

        for (uuid, user) in &self.state_of_pending_txn.user {
            if &user.id == user_id {
                is_user_id_exist = true;
            }
            if uuid == new_uuid {
                is_new_uuid_exist = true;
            }
        }

        (is_new_uuid_exist, is_user_id_exist)
    }

    pub(crate) async fn read_list_company_and_branch(
        &mut self,
        user_uuid: &types::UuidType,
    ) -> Vec<cases::list_company_and_branch::AllCompaniesThatUserInWithRoles> {
        // Start with data from the cache (hierarchical, not flat resources)
        let result = self.cache.read_list_company_and_branch(user_uuid).await;

        // Build a map for O(1) lookup and updates
        use std::collections::HashMap;
        let mut company_map: HashMap<
            types::UuidType,
            cases::list_company_and_branch::AllCompaniesThatUserInWithRoles,
        > = result
            .into_iter()
            .map(|c| (c.company_uuid.clone(), c))
            .collect();

        // Process pending access controls for this user
        for (_, acf) in &self.state_of_pending_txn.access_control_for_company {
            if acf.user_ != *user_uuid {
                continue;
            }

            let company_uuid = acf.data_group.clone();

            // If the company exists in pending transaction, use its data
            if let Some(company) = self.state_of_pending_txn.company.get(&company_uuid) {
                // Get or create the company entry in the map
                let company_entry = company_map.entry(company_uuid.clone()).or_insert_with(|| {
                    cases::list_company_and_branch::AllCompaniesThatUserInWithRoles {
                        company_uuid: company_uuid.clone(),
                        company_name: company.name.clone(),
                        company_currancy: company.currency.clone(),
                        user_roles: Vec::new(),
                        branches: Vec::new(),
                    }
                });

                // Overwrite with pending data (pending is more recent)
                company_entry.company_name = company.name.clone();
                company_entry.company_currancy = company.currency.clone();

                // Add the role from this access control (avoid duplicates)
                if !company_entry.user_roles.contains(&acf.role) {
                    company_entry.user_roles.push(acf.role.clone());
                }

                // Add pending branches that belong to this company
                for (branch_uuid, branch) in &self.state_of_pending_txn.company_branch {
                    if branch.company_belong == company_uuid {
                        // Check if branch already exists (by UUID)
                        let exists = company_entry
                            .branches
                            .iter()
                            .any(|b| b.branch_uuid == *branch_uuid);
                        if !exists {
                            company_entry.branches.push(
                                cases::list_company_and_branch::AllBranchesThatUserInWithRoles {
                                    branch_uuid: branch_uuid.clone(),
                                    branch_name: branch.name.clone(),
                                    branch_currancy: branch.currency.clone(),
                                    user_roles: Vec::new(), // No branch roles in pending transaction
                                },
                            );
                        }
                    }
                }
            }
            // else: no pending company – this shouldn't happen if there's an access control,
            // but we ignore it to avoid incomplete data.
        }

        // Convert the map back into a vector
        let data = company_map.into_values().collect();
        data
    }

    pub(crate) async fn read_create_company_branch(
        &mut self,
        user_uuid: &types::UuidType,
        company_belong: &types::UuidType,
        branch_name: &String,
    ) -> (
        Vec<types::Role>, /* user roles */
        bool,             /* is company_belong exist */
        bool,             /* is branch_name used */
    ) {
        // 1. Read from cache (database)
        let (mut user_roles, mut is_company_exist, mut is_branch_name_used) = self
            .cache
            .read_create_company_branch(user_uuid, company_belong, branch_name)
            .await;

        // 2. Check pending transactions (uncommitted changes)
        // Check pending company access control for roles
        for (_, acf) in &self.state_of_pending_txn.access_control_for_company {
            if acf.data_group == *company_belong && acf.user_ == *user_uuid {
                user_roles.push(acf.role.clone());
            }
        }

        // Check pending company existence
        if self
            .state_of_pending_txn
            .company
            .contains_key(company_belong)
        {
            is_company_exist = true;
        }

        // Check pending branch name usage
        for (_, branch) in &self.state_of_pending_txn.company_branch {
            if branch.company_belong == *company_belong && branch.name == *branch_name {
                is_branch_name_used = true;
                break;
            }
        }

        (user_roles, is_company_exist, is_branch_name_used)
    }

    pub(crate) async fn read_create_account(
        &mut self,
        read_input: &cases::create_account::ReadInput,
    ) -> cases::create_account::ReadOutput {
        let mut read_output = self.cache.read_create_account(read_input).await;
        read_output.is_company_uuid_exist = true;
        read_output.is_new_uuid_used = false;

        for (_, table) in &self.state_of_pending_txn.account {
            if read_input.account_name == table.name
                && read_input.belong_to_company == table.company_belong
            {
                read_output.is_account_name_used = true;
            }
        }

        let mut roles = Vec::new();
        for (_, table) in &self.state_of_pending_txn.access_control_for_company {
            if read_input.user_uuid == table.user_ {
                roles.push(table.role.clone());
            }
        }

        read_output
    }
}
