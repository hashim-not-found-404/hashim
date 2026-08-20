use crate::utility::cache_adapter;
use crate::utility::utils::MyUuidConverter;
use crate::utility::utils::MyUuidConverter1;
use accounting_engine::accounting_stuff;
use my_core::accounting_domain::cases;
use my_core::accounting_domain::utility::types;
use my_core::utility::traits;
use rusqlite::params;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::collections::HashSet;
use std::str::FromStr;

const QUERY: &str = r#"
    WITH
    new_uuid_check AS (
        SELECT EXISTS(SELECT 1 FROM entry WHERE rowid = ?1) AS is_new_uuid_used
    ),
    user_roles AS (
        SELECT COALESCE(
            (
                SELECT json_group_array(DISTINCT role)
                FROM (
                    SELECT role FROM access_control_for_company
                    WHERE data_group = (SELECT company_belong FROM company_branch WHERE rowid = ?2)
                    AND user_ = ?3
                    UNION
                    SELECT role FROM access_control_for_company_branch
                    WHERE data_group = ?2 AND user_ = ?3
                )
            ),
            '[]'
        ) AS roles_json
    ),
    shared_entry_check AS (
        SELECT EXISTS(SELECT 1 FROM shared_entry WHERE rowid = ?4) AS is_shared_entry_exist
    ),
    new_entries_checks AS (
        SELECT json_group_object(rowid, exists_flag) AS new_entries_map
        FROM (
            SELECT rowid,
                   EXISTS(SELECT 1 FROM single_entry WHERE rowid = rowid) AS exists_flag
            FROM (
                SELECT value as rowid FROM json_each(?5)
            ) t
        )
    ),
    account_infos AS (
        SELECT json_group_array(
            json_object(
                'account_uuid', COALESCE(a.rowid, u.account_uuid),
                'is_debit', COALESCE(a.is_debit, 1),
                'in_flow_type', COALESCE(aft.inflow_type, 'Manual'),
                'out_flow_type', COALESCE(aft.outflow_type, 'Manual'),
                'inventory', COALESCE(aft.inventory, '[]')  -- fixed: use aft.inventory
            )
        ) AS account_infos_json
        FROM (
            SELECT value as account_uuid FROM json_each(?6)
        ) u
        LEFT JOIN account a ON a.rowid = u.account_uuid
        LEFT JOIN account_flow_type aft ON aft.account = a.rowid AND aft.company_branch = ?2
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
    type Db<'a> = cache_adapter::S;

    async fn read(
        db: &mut Self::Db<'_>,
        read_input: &cases::create_journal_entry::ReadInput,
    ) -> Result<cases::create_journal_entry::ReadOutput, traits::DynamicError> {
        let accounts_uuid_vec: Vec<String> =
            read_input.accounts_uuid.iter().map(|uuid| uuid.to_string()).collect();
        let new_entries_uuid_vec: Vec<String> =
            read_input.new_entries_uuid.iter().map(|uuid| uuid.to_string()).collect();

        let accounts_json = serde_json::to_string(&accounts_uuid_vec).unwrap();
        let new_entries_json = serde_json::to_string(&new_entries_uuid_vec).unwrap();

        let mut stmt = db.tables_db.prepare(QUERY).unwrap();
        let mut rows = stmt
            .query(params![
                &read_input.new_uuid.to_string(),
                &read_input.belong_to_company_branch.to_string(),
                &read_input.user_uuid.to_string(),
                &read_input.shared_entry_id.as_ref().map(|id| id.to_string()),
                &new_entries_json,
                &accounts_json,
            ])
            .unwrap();

        let row = rows.next().unwrap().unwrap();

        let is_new_uuid_used: bool = row.get(0).unwrap();
        let roles_json: String = row.get(1).unwrap();
        let is_shared_entry_exist: bool = row.get(2).unwrap();
        let new_entries_map_json: String = row.get(3).unwrap();
        let account_infos_json: String = row.get(4).unwrap();

        let roles_value: Value = serde_json::from_str(&roles_json).unwrap_or(Value::Array(vec![]));
        let user_roles: Vec<types::Role> = roles_value
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| types::Role::from_str(s).unwrap())
            .collect();

        let new_entries_value: Value = serde_json::from_str(&new_entries_map_json)
            .unwrap_or(Value::Object(serde_json::Map::new()));
        let mut used_new_entries_uuid = HashSet::new();
        if let Some(obj) = new_entries_value.as_object() {
            for (key, value) in obj {
                let uuid_str = key.clone();
                let used = value.as_bool().unwrap_or(false);
                let uuid_type = uuid_str.to_uuid();
                if used {
                    used_new_entries_uuid.insert(uuid_type);
                }
            }
        }

        #[derive(Deserialize)]
        struct AccountInfoJson {
            account_uuid:  String,
            is_debit:      bool,
            in_flow_type:  String,
            out_flow_type: String,
            inventory:     Vec<accounting_stuff::InventoryRecord>,
        }

        let account_infos: Vec<AccountInfoJson> =
            serde_json::from_str(&account_infos_json).unwrap_or_default();

        let mut account_info = cases::create_journal_entry::AccountInfoProviderImpl(HashMap::new());
        for info in account_infos {
            let uuid_type = info.account_uuid.to_uuid();
            let in_flow = accounting_stuff::InFlowType::from_str(&info.in_flow_type).unwrap();
            let out_flow = accounting_stuff::OutFlowType::from_str(&info.out_flow_type).unwrap();
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
    use crate::utility::test_helper::test_query_helper_for_tables_schema;

    #[test]
    fn test_query_string_directly() {
        test_query_helper_for_tables_schema(QUERY).unwrap();
    }
}
