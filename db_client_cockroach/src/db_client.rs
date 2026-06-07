use crate::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};
use uuid::Uuid;

pub struct S {
    pub(crate) client: deadpool_postgres::Object,
}

impl DBClient for S {
    type RowId = row_id::m::S;
    type HashedPassword = authentication::m::S;
    type Txn<'a> = db_transaction::S<'a>;

    async fn begin_transaction(&mut self) -> Result<Self::Txn<'_>, DynamicError> {
        Ok(db_transaction::S {
            txn: self.client.transaction().await.log()?,
        })
    }

    async fn read_sign_in(
        &mut self,
        user_id: &String,
    ) -> Result<Option<(Self::RowId, Self::HashedPassword)>, DynamicError> {
        let query = "SELECT rowid,pass FROM accounting_app.user WHERE id = $1 LIMIT 1;";
        let stmt = self.client.prepare_cached(query).await.log()?;
        let row = self.client.query_opt(&stmt, &[user_id]).await.log()?;

        match row {
            Some(row) => {
                let row_id = row.try_get::<_, Uuid>(0).log()?;
                let hashed_password = row.try_get::<_, String>(1).log()?;
                return Ok(Some((row_id.into(), hashed_password.into())));
            }
            None => {
                return Ok(None);
            }
        }
    }

    async fn read_roles_for_user(
        &mut self,
        users_uuid: &HashSet<Self::RowId>,
    ) -> Result<server_methods::AllRoles<Self::RowId>, DynamicError> {
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

        let mut result = server_methods::AllRoles::<Self::RowId> {
            companies: HashMap::new(),
            branches: HashMap::new(),
        };

        for user_uuid in users_uuid {
            // Convert RowId to the actual UUID type expected by the database
            let user_id_param = user_uuid.clone().into_inner();

            let rows = self.client.query(&stmt, &[&user_id_param]).await.log()?;

            for row in rows {
                let entity_type: String = row.try_get("type").log()?;
                let data_group: Uuid = row.try_get("data_group").log()?;
                let role_str: String = row.try_get("role").log()?;
                let user_id: Uuid = row.try_get("user_").log()?;

                // Parse role from string
                let role = db_types::Role::from_str(&role_str).log()?;

                // Convert Uuid to your RowId type
                let data_group_id = Self::RowId::from(data_group);
                let user_id_typed = Self::RowId::from(user_id);

                match entity_type.as_str() {
                    "company" => {
                        result
                            .companies
                            .entry(data_group_id)
                            .or_insert_with(HashMap::new)
                            .entry(user_id_typed)
                            .or_insert_with(Vec::new)
                            .push(role);
                    }
                    "branch" => {
                        result
                            .branches
                            .entry(data_group_id)
                            .or_insert_with(HashMap::new)
                            .entry(user_id_typed)
                            .or_insert_with(Vec::new)
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
        nonce: &Self::RowId,
    ) -> Result<bool /* is nonce used */, DynamicError> {
        let row = self
            .client
            .query_one(
                "INSERT INTO accounting_app.transaction_number (rowid) VALUES ($1)
                 ON CONFLICT (rowid) DO NOTHING
                 RETURNING true",
                &[&nonce.into_inner()],
            )
            .await
            .log()?;

        let inserted: Option<Uuid> = row.try_get(0).ok();
        Ok(inserted.is_some())
    }
}
