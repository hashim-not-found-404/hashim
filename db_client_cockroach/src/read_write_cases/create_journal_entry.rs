use crate::utility::db_transaction;
use crate::utility::utils::MyUuidConverter;
use accounting_engine::accounting_stuff;
use my_core::accounting_domain::cases;
use my_core::accounting_domain::utility::types;
use my_core::utility::traits;
use my_core::utility::utils::LogError;
use serde_json::Value;
use std::collections::HashMap;
use std::collections::HashSet;
use std::str::FromStr;
use uuid::Uuid;

const QUERY1: &str = r#"
    WITH
    new_uuid_check AS (
        SELECT EXISTS(SELECT 1 FROM accounting_app.entry WHERE rowid = $1) AS is_new_uuid_used
    ),
    user_roles AS (
        SELECT COALESCE(
            (SELECT jsonb_agg(DISTINCT role) FROM (
                SELECT role FROM accounting_app.access_control_for_company
                WHERE data_group = (SELECT company_belong FROM accounting_app.company_branch WHERE rowid = $2)
                AND user_ = $3
                UNION
                SELECT role FROM accounting_app.access_control_for_company_branch
                WHERE data_group = $2 AND user_ = $3
            )),
            '[]'::jsonb
        ) AS roles_json
    ),
    shared_entry_check AS (
        SELECT EXISTS(SELECT 1 FROM accounting_app.shared_entry WHERE rowid = $4) AS is_shared_entry_exist
    ),
    new_entries_checks AS (
        SELECT COALESCE(
            jsonb_object_agg(rowid::text, exists_flag),
            '{}'::jsonb
        ) AS new_entries_map
        FROM (
            SELECT rowid,
                   EXISTS(SELECT 1 FROM accounting_app.single_entry WHERE rowid = rowid) AS exists_flag
            FROM unnest($5::uuid[]) AS rowid
        ) t
    ),
    account_infos AS (
        SELECT COALESCE(
            jsonb_agg(
                jsonb_build_object(
                    'account_uuid', a.rowid,
                    'is_debit', COALESCE(a.is_debit, true),
                    'in_flow_type', COALESCE(aft.inflow_type, 'Manual'),
                    'out_flow_type', COALESCE(aft.outflow_type, 'Manual'),
                    'inventory', COALESCE(aft.inventory, '[]'::jsonb)
                )
            ),
            '[]'::jsonb
        ) AS account_infos_json
        FROM unnest($6::uuid[]) AS u(account_uuid)
        LEFT JOIN accounting_app.account a ON a.rowid = u.account_uuid
        LEFT JOIN accounting_app.account_flow_type aft ON aft.account = a.rowid AND aft.company_branch = $2
    )
    SELECT
        (SELECT is_new_uuid_used FROM new_uuid_check) AS is_new_uuid_used,
        (SELECT roles_json FROM user_roles) AS user_roles,
        (SELECT is_shared_entry_exist FROM shared_entry_check) AS is_shared_entry_exist,
        (SELECT new_entries_map FROM new_entries_checks) AS new_entries_map,
        (SELECT account_infos_json FROM account_infos) AS account_infos_json
"#;

pub struct S;

impl cases::create_journal_entry::DatabaseRead for S {
    type Db<'a> = db_transaction::S<'a>;

    async fn read(
        db: &mut Self::Db<'_>,
        read_input: &cases::create_journal_entry::ReadInput,
    ) -> Result<cases::create_journal_entry::ReadOutput, traits::DynamicError> {
        // Convert HashSets to Vec<Uuid> for query parameters
        let accounts_uuid_vec: Vec<Uuid> =
            read_input.accounts_uuid.iter().map(|uuid| uuid.to_externel_uuid()).collect();
        let new_entries_uuid_vec: Vec<Uuid> =
            read_input.new_entries_uuid.iter().map(|uuid| uuid.to_externel_uuid()).collect();

        // Prepare parameters
        let params: &[&(dyn tokio_postgres::types::ToSql + Sync)] = &[
            &read_input.new_uuid.to_externel_uuid(),
            &read_input.belong_to_company_branch.to_externel_uuid(),
            &read_input.user_uuid.to_externel_uuid(),
            &read_input.shared_entry_id.as_ref().map(|id| id.to_externel_uuid()),
            &new_entries_uuid_vec,
            &accounts_uuid_vec,
        ];

        // Execute query
        let row = db.txn.query_one(QUERY1, params).await.log()?;

        // Parse results
        let is_new_uuid_used: bool = row.try_get(0).log()?;
        let roles_json: Value = row.try_get(1).log()?;
        let is_shared_entry_exist: bool = row.try_get(2).log()?;
        let new_entries_map_json: Value = row.try_get(3).log()?;
        let account_infos_json: Value = row.try_get(4).log()?;

        // Parse user roles
        let user_roles: Vec<types::Role> = roles_json
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| types::Role::from_str(s).unwrap())
            .collect();

        // Parse new entries UUID map
        let mut used_new_entries_uuid = HashSet::new();
        if let Some(obj) = new_entries_map_json.as_object() {
            for (key, value) in obj {
                let uuid_str = key.clone();
                let used = value.as_bool().unwrap_or(false);
                // Convert string back to UuidType
                let uuid_parsed = Uuid::parse_str(&uuid_str).log()?;
                let uuid_type = types::UuidType(uuid_parsed.into_bytes());
                if used {
                    used_new_entries_uuid.insert(uuid_type);
                }
            }
        }

        // Parse account infos
        #[derive(serde::Deserialize)]
        struct AccountInfoJson {
            account_uuid:  String,
            is_debit:      bool,
            in_flow_type:  String,
            out_flow_type: String,
            inventory:     Vec<accounting_stuff::InventoryRecord>,
        }

        let account_infos: Vec<AccountInfoJson> =
            serde_json::from_value(account_infos_json).unwrap_or_default();

        let mut account_info = cases::create_journal_entry::AccountInfoProviderImpl(HashMap::new());
        for info in account_infos {
            let uuid_parsed = Uuid::parse_str(&info.account_uuid).log()?;
            let uuid_type = types::UuidType(uuid_parsed.into_bytes());
            let in_flow = accounting_stuff::InFlowType::from_str(&info.in_flow_type).log()?;
            let out_flow = accounting_stuff::OutFlowType::from_str(&info.out_flow_type).log()?;
            let inventory = cases::create_journal_entry::InventoryWrapper(info.inventory);
            let account_info_entry = cases::create_journal_entry::AccountInfo {
                is_debit: info.is_debit,
                in_flow_type: in_flow,
                out_flow_type: out_flow,
                inventory,
            };
            account_info.0.insert(uuid_type, account_info_entry);
        }

        Ok(cases::create_journal_entry::ReadOutput {
            is_new_uuid_used,
            user_roles,
            is_shared_entry_exist,
            used_new_entries_uuid,
            account_info,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utility::test_helper::test_query_helper;

    #[tokio::test]
    async fn test_query_string_directly() {
        test_query_helper(QUERY1).await.unwrap();
    }
}
