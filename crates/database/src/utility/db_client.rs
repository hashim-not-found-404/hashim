use crate::utility::db_transaction;
use crate::utility::utils::MyUuidConverter;
use my_core::domain::utility::new_types::BranchUuid;
use my_core::domain::utility::new_types::CompanyUuid;
use my_core::domain::utility::new_types::NonceUuid;
use my_core::domain::utility::new_types::UserUuid;
use my_core::domain::utility::new_types::UuidType;
use my_core::domain::utility::types::Role;
use my_core::server::utility::server_traits;
use my_core::server::utility::server_traits::DBClient;
use my_core::utility::traits::DynamicError;
use my_core::utility::utils::LogError;
use std::collections::HashSet;
use std::str::FromStr;
use uuid::Uuid;

const READ_ROLES_FOR_USER_QUERY: &str = r#"
    SELECT
        'company' as type,
        data_group,
        role,
        user_
    FROM accounting_app.access_control_for_company
    WHERE user_ = $1

    UNION ALL

    SELECT
        'branch' as type,
        data_group,
        role,
        user_
    FROM accounting_app.access_control_for_company_branch
    WHERE user_ = $1
"#;

const WRITE_NONCE_IF_NOT_USED_QUERY: &str = "
    INSERT INTO accounting_app.transaction_number (rowid) VALUES ($1)
    ON CONFLICT (rowid) DO NOTHING
    RETURNING true";

pub struct S {
    pub(crate) client: deadpool_postgres::Object,
}

impl DBClient for S {
    type Txn<'a> = db_transaction::S<'a>;

    async fn begin_transaction(&mut self) -> Result<Self::Txn<'_>, DynamicError> {
        Ok(db_transaction::S {
            txn: self.client.transaction().await.log()?,
        })
    }

    async fn read_roles_for_user(
        &mut self,
        users_uuid: &HashSet<UserUuid>,
    ) -> Result<server_traits::TheCompaniesAndBranchesHeIn, DynamicError> {
        let stmt = self.client.prepare_cached(READ_ROLES_FOR_USER_QUERY).await.log()?;

        let mut result = server_traits::TheCompaniesAndBranchesHeIn {
            companies:                Default::default(),
            branches:                 Default::default(),
            branches_of_each_company: Default::default(),
        };

        for user_uuid in users_uuid {
            let user_id_param = user_uuid.clone().to_externel_uuid();

            let rows = self.client.query(&stmt, &[&user_id_param]).await.log()?;

            for row in rows {
                let entity_type: String = row.try_get("type").log()?;
                let data_group: Uuid = row.try_get("data_group").log()?;
                let role_str: String = row.try_get("role").log()?;
                let user_id: Uuid = row.try_get("user_").log()?;

                let role = Role::from_str(&role_str).log()?;

                let data_group_id = UuidType(data_group.into_bytes()).into();
                let user_id_typed = UuidType(user_id.into_bytes()).into();

                match entity_type.as_str() {
                    "company" => {
                        result
                            .companies
                            .entry(user_id_typed)
                            .or_default()
                            .insert(CompanyUuid(data_group_id));
                    }
                    "branch" => {
                        result
                            .branches
                            .entry(user_id_typed)
                            .or_default()
                            .insert(BranchUuid(data_group_id));
                    }
                    _ => {}
                }
            }
        }

        Ok(result)
    }

    async fn write_nonce_if_not_used_and_return_is_nonce_used(
        &mut self,
        nonce: &NonceUuid,
    ) -> Result<bool, DynamicError> {
        let row = self
            .client
            .query_one(WRITE_NONCE_IF_NOT_USED_QUERY, &[&nonce.to_externel_uuid()])
            .await
            .log()?;

        let inserted: Option<Uuid> = row.try_get(0).ok();
        Ok(inserted.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utility::test_helper::test_query_helper;

    #[tokio::test]
    async fn test_query_string_directly() {
        test_query_helper(READ_ROLES_FOR_USER_QUERY).await.unwrap();
        test_query_helper(WRITE_NONCE_IF_NOT_USED_QUERY).await.unwrap();
    }
}
