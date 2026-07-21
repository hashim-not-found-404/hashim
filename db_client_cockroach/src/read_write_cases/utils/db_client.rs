use crate::read_write_cases::utils::{db_transaction, utils::MyUuidConverter};
use my_core::{
    accounting_domain::cases::utility::types,
    server::use_cases::utility::server_traits::{self, DBClient},
    utility::{traits::DynamicError, utils::LogError},
};
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};
use uuid::Uuid;

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
        users_uuid: &HashSet<types::UuidType>,
    ) -> Result<server_traits::AllRoles, DynamicError> {
        let query = r#"
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

        let stmt = self.client.prepare_cached(query).await.log()?;

        let mut result = server_traits::AllRoles {
            companies: HashMap::new(),
            branches: HashMap::new(),
        };

        for user_uuid in users_uuid {
            // Convert RowId to the actual UUID type expected by the database
            let user_id_param = user_uuid.clone().to_externel_uuid();

            let rows = self.client.query(&stmt, &[&user_id_param]).await.log()?;

            for row in rows {
                let entity_type: String = row.try_get("type").log()?;
                let data_group: Uuid = row.try_get("data_group").log()?;
                let role_str: String = row.try_get("role").log()?;
                let user_id: Uuid = row.try_get("user_").log()?;

                // Parse role from string
                let role = types::Role::from_str(&role_str).log()?;

                // Convert Uuid to your RowId type
                let data_group_id = types::UuidType(data_group.into_bytes());
                let user_id_typed = types::UuidType(user_id.into_bytes());

                match entity_type.as_str() {
                    "company" => {
                        result
                            .companies
                            .entry(data_group_id)
                            .or_default()
                            .entry(user_id_typed)
                            .or_default()
                            .push(role);
                    }
                    "branch" => {
                        result
                            .branches
                            .entry(data_group_id)
                            .or_default()
                            .entry(user_id_typed)
                            .or_default()
                            .push(role);
                    }
                    _ => {}
                }
            }
        }

        Ok(result)
    }

    async fn write_nonce_if_not_used(
        &mut self,
        nonce: &types::UuidType,
    ) -> Result<bool /* is nonce used */, DynamicError> {
        let row = self
            .client
            .query_one(
                "INSERT INTO accounting_app.transaction_number (rowid) VALUES ($1)
                 ON CONFLICT (rowid) DO NOTHING
                 RETURNING true",
                &[&nonce.to_externel_uuid()],
            )
            .await
            .log()?;

        let inserted: Option<Uuid> = row.try_get(0).ok();
        Ok(inserted.is_some())
    }
}
