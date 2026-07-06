use crate::{
    accounting_client::client_traits::{AllClientTypes, Cache},
    accounting_domain::types,
    utility::utils,
};
use std::collections::{HashMap, HashSet};

pub(crate) mod tables {
    use crate::accounting_domain::types;

    #[derive(Default)]
    pub(crate) struct User {
        pub(crate) name: Option<String>,
        pub(crate) id: String,
        pub(crate) password: String,
    }
    #[derive(Default)]
    pub(crate) struct Company {
        pub(crate) name: String,
        pub(crate) currency: types::Currency,
    }
    #[derive(Default)]
    pub(crate) struct AccessControlForCompany {
        pub(crate) data_group: types::UuidType,
        pub(crate) user_: types::UuidType,
        pub(crate) role: types::Role,
    }
    #[derive(Default)]
    pub(crate) struct CompanyBranch {
        pub(crate) company_belong: types::UuidType,
        pub(crate) name: String,
        pub(crate) location: types::Location,
        pub(crate) currency: types::Currency,
    }
    #[derive(Default)]
    pub(crate) struct AccessControlForCompanyBranch {
        pub(crate) data_group: types::UuidType,
        pub(crate) user_: types::UuidType,
        pub(crate) role: types::Role,
    }
}

#[derive(Default)]
pub(crate) struct StateOfPendingTxn {
    pub(crate) user: HashMap<types::UuidType, tables::User>,
    pub(crate) company: HashMap<types::UuidType, tables::Company>,
    pub(crate) access_control_for_company:
        HashMap<types::UuidType, tables::AccessControlForCompany>,
    pub(crate) company_branch: HashMap<types::UuidType, tables::CompanyBranch>,
    pub(crate) access_control_for_company_branch:
        HashMap<types::UuidType, tables::AccessControlForCompanyBranch>,
}

pub(crate) struct State<At: AllClientTypes> {
    pub(crate) state_of_pending_txn: StateOfPendingTxn,
    pub(crate) cache: At::Ch,
}

impl<At: AllClientTypes> State<At> {
    pub(crate) async fn new() -> Self {
        let cache = At::Ch::new().await;
        let txns = cache.get_all_txn_input().await;

        let mut state = Self {
            state_of_pending_txn: StateOfPendingTxn::default(),
            cache,
        };

        for op in txns {
            op.operation
                .run_operation_check_apply::<At>(&mut state, &mut HashSet::new())
                .await;
        }

        state
    }
}

impl<At: AllClientTypes> State<At> {
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

    pub(crate) async fn read_sign_in(
        &mut self,
        user_id: &String,
    ) -> Option<(types::UuidType, String)> {
        unreachable!("this is not callable at client side")
    }

    pub(crate) async fn read_list_company_and_branch(
        &mut self,
        user_uuid: &types::UuidType,
    ) -> Vec<types::ResourceInfo> {
        // Start with resources from the cache (already stored in DB)
        let mut resources = self.cache.read_list_company_and_branch(&user_uuid).await;

        // Add pending companies and branches from the current transaction
        for (_, acf) in &self.state_of_pending_txn.access_control_for_company {
            if acf.user_ == user_uuid.clone() {
                let company_uuid = acf.data_group.clone();
                if let Some(company) = self.state_of_pending_txn.company.get(&company_uuid) {
                    // Company name
                    resources.push(types::ResourceInfo {
                        row_uuid: company_uuid.clone(),
                        resource: types::Resource::TableCompanyFieldName(company.name.clone()),
                    });

                    // Access control: role
                    resources.push(types::ResourceInfo {
                        row_uuid: company_uuid.clone(),
                        resource: types::Resource::TableAccessControlForCompanyFieldRole(
                            acf.role.clone(),
                        ),
                    });

                    // Access control: user_
                    resources.push(types::ResourceInfo {
                        row_uuid: company_uuid.clone(),
                        resource: types::Resource::TableAccessControlForCompanyFieldUser(
                            user_uuid.clone(),
                        ),
                    });

                    // Access control: data_group
                    resources.push(types::ResourceInfo {
                        row_uuid: company_uuid.clone(),
                        resource: types::Resource::TableAccessControlForCompanyFieldDataGroup(
                            company_uuid.clone(),
                        ),
                    });

                    // Pending branches for this company
                    for (branch_uuid, branch) in &self.state_of_pending_txn.company_branch {
                        if branch.company_belong == company_uuid {
                            resources.push(types::ResourceInfo {
                                row_uuid: branch_uuid.clone(),
                                resource: types::Resource::TableCompanyBranchFieldName(
                                    branch.name.clone(),
                                ),
                            });
                            resources.push(types::ResourceInfo {
                                row_uuid: branch_uuid.clone(),
                                resource: types::Resource::TableCompanyBranchFieldCompanyBelong(
                                    company_uuid.clone(),
                                ),
                            });
                        }
                    }
                }
            }
        }

        return resources;
    }

    pub(crate) async fn read_create_company_branch(
        &mut self,
        new_uuid: &types::UuidType,
        user_uuid: &types::UuidType,
        company_belong: &types::UuidType,
        branch_name: &String,
    ) -> (
        Vec<types::Role>, /* user roles */
        bool,             /* is new_uuid exist */
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

        (user_roles, false, is_company_exist, is_branch_name_used)
    }
}
