use crate::{
    db_types,
    decider::StateOp,
    request_response::ResourceInfo,
    server_methods,
    traits::{AllClientTypes, Cache},
    utils,
};
use std::collections::{HashMap, HashSet};

pub mod tables {
    use crate::db_types;

    #[derive(Default)]
    pub struct User {
        pub name: Option<String>,
        pub id: String,
        pub password: String,
    }
    #[derive(Default)]
    pub struct Company {
        pub name: String,
        pub currency: db_types::Currency,
    }
    #[derive(Default)]
    pub struct AccessControlForCompany {
        pub data_group: db_types::UuidType,
        pub user_: db_types::UuidType,
        pub role: db_types::Role,
    }
    #[derive(Default)]
    pub struct CompanyBranch {
        pub company_belong: db_types::UuidType,
        pub name: String,
        pub location: db_types::Location,
        pub currency: db_types::Currency,
    }
    #[derive(Default)]
    pub struct AccessControlForCompanyBranch {
        pub data_group: db_types::UuidType,
        pub user_: db_types::UuidType,
        pub role: db_types::Role,
    }
}

#[derive(Default)]
pub struct StateOfPendingTxn {
    pub user: HashMap<db_types::UuidType, tables::User>,
    pub company: HashMap<db_types::UuidType, tables::Company>,
    pub access_control_for_company: HashMap<db_types::UuidType, tables::AccessControlForCompany>,
    pub company_branch: HashMap<db_types::UuidType, tables::CompanyBranch>,
    pub access_control_for_company_branch:
        HashMap<db_types::UuidType, tables::AccessControlForCompanyBranch>,
}

pub struct State<At: AllClientTypes> {
    pub state_of_pending_txn: StateOfPendingTxn,
    pub cache: At::Ch,
}

impl<At: AllClientTypes> State<At> {
    pub async fn new() -> Self {
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

impl<At: AllClientTypes> StateOp for State<At> {
    async fn read_sign_up(
        &mut self,
        new_uuid: &db_types::UuidType,
        user_id: &String,
    ) -> Result<
        (
            bool, /* is new_uuid exist */
            bool, /* is user_id exist */
        ),
        utils::DynamicError,
    > {
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

        Ok((is_new_uuid_exist, is_user_id_exist))
    }

    async fn read_sign_in(
        &mut self,
        user_id: &String,
    ) -> Result<Option<(db_types::UuidType, String)>, utils::DynamicError> {
        unreachable!("this is not callable at client side")
    }

    async fn read_create_company(
        &mut self,
        new_uuid: &db_types::UuidType,
    ) -> Result<bool /* is new_uuid exist */, utils::DynamicError> {
        Ok(false)
    }

    async fn read_list_company_and_branch(
        &mut self,
        user_uuid: &db_types::UuidType,
    ) -> Result<Vec<ResourceInfo>, utils::DynamicError> {
        // Start with resources from the cache (already stored in DB)
        let mut resources = self.cache.read_list_company_and_branch(&user_uuid).await;

        // Add pending companies and branches from the current transaction
        for (_, acf) in &self.state_of_pending_txn.access_control_for_company {
            if acf.user_ == user_uuid.clone() {
                let company_uuid = acf.data_group.clone();
                if let Some(company) = self.state_of_pending_txn.company.get(&company_uuid) {
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
                            user_uuid.clone(),
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
                    for (branch_uuid, branch) in &self.state_of_pending_txn.company_branch {
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

        return Ok(resources);
    }

    async fn read_create_company_branch(
        &mut self,
        new_uuid: &db_types::UuidType,
        user_uuid: &db_types::UuidType,
        company_belong: &db_types::UuidType,
        branch_name: &String,
    ) -> Result<
        (
            Vec<db_types::Role>, /* user roles */
            bool,                /* is new_uuid exist */
            bool,                /* is company_belong exist */
            bool,                /* is branch_name used */
        ),
        utils::DynamicError,
    > {
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

        Ok((user_roles, false, is_company_exist, is_branch_name_used))
    }
}
