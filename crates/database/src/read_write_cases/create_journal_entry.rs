use crate::utility::db_transaction;
use crate::utility::utils::MyUuidConverter;
use accounting_engine::accounting_stuff;
use my_core::domain::use_cases;
use my_core::domain::utility::types::DatabaseRead;
use my_core::domain::utility::types::DatabaseWrite;
use my_core::domain::utility::types::Role;
use my_core::domain::utility::uuid::UuidType;
use my_core::server::utility::server_traits;
use my_core::utility::traits::DynamicError;
use my_core::utility::utils::LogError;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::collections::HashSet;
use std::str::FromStr;
use tokio_postgres::types::ToSql;
use uuid::Uuid;

const READ_QUERY: &str = r#"
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

impl use_cases::create_journal_entry::DatabaseRead for S {}

impl DatabaseRead for S {
    type Db<'a> = db_transaction::S<'a>;
    type Input = use_cases::create_journal_entry::ReadInput;
    type Output = use_cases::create_journal_entry::ReadOutput;

    async fn read(
        db: &mut Self::Db<'_>,
        input: &Self::Input,
    ) -> Result<Self::Output, DynamicError> {
        let accounts_uuid_vec: Vec<Uuid> =
            input.accounts_uuid.iter().map(|uuid| uuid.to_externel_uuid()).collect();
        let new_entries_uuid_vec: Vec<Uuid> =
            input.new_entries_uuid.iter().map(|uuid| uuid.to_externel_uuid()).collect();

        let params: &[&(dyn ToSql + Sync)] = &[
            &input.new_uuid.to_externel_uuid(),
            &input.belong_to_company_branch.to_externel_uuid(),
            &input.user_uuid.to_externel_uuid(),
            &input.shared_entry_id.as_ref().map(|id| id.to_externel_uuid()),
            &new_entries_uuid_vec,
            &accounts_uuid_vec,
        ];

        let row = db.txn.query_one(READ_QUERY, params).await.log()?;

        let is_new_uuid_used: bool = row.try_get(0).log()?;
        let roles_json: Value = row.try_get(1).log()?;
        let is_shared_entry_exist: bool = row.try_get(2).log()?;
        let new_entries_map_json: Value = row.try_get(3).log()?;
        let account_infos_json: Value = row.try_get(4).log()?;

        let user_roles: Vec<Role> = roles_json
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| Role::from_str(s).unwrap())
            .collect();

        let mut used_new_entries_uuid = HashSet::new();
        if let Some(obj) = new_entries_map_json.as_object() {
            for (key, value) in obj {
                let uuid_str = key.clone();
                let used = value.as_bool().unwrap_or(false);
                let uuid_parsed = Uuid::parse_str(&uuid_str).log()?;
                let uuid_type = UuidType(uuid_parsed.into_bytes());
                if used {
                    used_new_entries_uuid.insert(uuid_type);
                }
            }
        }

        #[derive(Debug, Deserialize)]
        struct AccountInfoJson {
            account_uuid:  String,
            is_debit:      bool,
            in_flow_type:  String,
            out_flow_type: String,
            inventory:     Vec<accounting_stuff::InventoryRecord>,
        }

        let account_infos: Vec<AccountInfoJson> =
            serde_json::from_value(account_infos_json).unwrap_or_default();

        let mut account_info =
            use_cases::create_journal_entry::AccountInfoProviderImpl(HashMap::new());
        for info in account_infos {
            let uuid_parsed = Uuid::parse_str(&info.account_uuid).log()?;
            let uuid_type = UuidType(uuid_parsed.into_bytes()).into();
            let in_flow = accounting_stuff::InFlowType::from_str(&info.in_flow_type).log()?;
            let out_flow = accounting_stuff::OutFlowType::from_str(&info.out_flow_type).log()?;
            let inventory = use_cases::create_journal_entry::InventoryWrapper(info.inventory);
            let account_info_entry = use_cases::create_journal_entry::AccountInfo {
                is_debit: info.is_debit,
                in_flow_type: in_flow,
                out_flow_type: out_flow,
                inventory,
            };
            account_info.0.insert(uuid_type, account_info_entry);
        }

        Ok(use_cases::create_journal_entry::ReadOutput {
            is_new_uuid_used,
            user_roles,
            is_shared_entry_exist,
            used_new_entries_uuid,
            account_info,
        })
    }
}

const WRITE_QUERY: &str = r#"
    WITH entry_insert AS (
        INSERT INTO accounting_app.entry (rowid, writer, time, shared_entry_id)
        VALUES ($3, $4, $5, $6)
        RETURNING 1
    ),
    single_insert AS (
        INSERT INTO accounting_app.single_entry (
            rowid, double_entry, entry, account, is_debit,
            cost_out_flow_type, quantity, amount
        )
        SELECT
            (j->>'rowid')::uuid,
            (j->>'double_entry')::smallint,
            (j->>'entry')::uuid,
            (j->>'account')::uuid,
            (j->>'is_debit')::bool,
            j->>'cost_out_flow_type',
            (j->>'quantity')::decimal,
            (j->>'amount')::decimal
        FROM jsonb_array_elements($1::jsonb) AS j
        RETURNING 1
    ),
    inventory_update AS (
        UPDATE accounting_app.account_flow_type
        SET inventory = (u.value->>'inventory')::jsonb
        FROM (
            SELECT jsonb_array_elements($2::jsonb) AS value
        ) AS u
        WHERE account_flow_type.rowid = (u.value->>'account_uuid')::uuid
        RETURNING 1
    )
    SELECT 1
"#;

impl DatabaseWrite for S {
    type Db<'a> = db_transaction::S<'a>;
    type Input = use_cases::create_journal_entry::Ok;

    async fn write(txn: &mut Self::Db<'_>, input: &Self::Input) -> Result<(), DynamicError> {
        let single_entries: Vec<serde_json::Value> = input
            .double_entry
            .iter()
            .map(|single| {
                serde_json::json!({
                    "rowid": single.new_uuid.to_externel_uuid(),
                    "double_entry": single.double_entry_number as i16,
                    "entry": input.new_uuid.to_externel_uuid(),
                    "account": single.account.to_externel_uuid(),
                    "is_debit": single.is_debit,
                    "cost_out_flow_type": single.out_flow_type.as_str(),
                    "quantity": single.quantity,
                    "amount": single.amount,
                })
            })
            .collect();
        let single_json = serde_json::to_value(&single_entries)?;

        let inventory_updates: Vec<serde_json::Value> = input
            .inventory
            .iter()
            .map(|(account_uuid, inventory_wrapper)| {
                serde_json::json!({
                    "account_uuid": account_uuid.to_externel_uuid(),
                    "inventory": inventory_wrapper.0,
                })
            })
            .collect();
        let inventory_json = serde_json::to_value(&inventory_updates)?;

        let shared_entry_id_param = input.shared_entry_id.as_ref().map(|id| id.to_externel_uuid());

        txn.txn
            .execute(WRITE_QUERY, &[
                &single_json,
                &inventory_json,
                &input.new_uuid.to_externel_uuid(),
                &input.user_uuid.to_externel_uuid(),
                &(input.time as i64),
                &shared_entry_id_param,
            ])
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utility::test_helper::test_query_helper;

    #[tokio::test]
    async fn test_query_string_directly() {
        test_query_helper(READ_QUERY).await.unwrap();
        test_query_helper(WRITE_QUERY).await.unwrap();
    }
}
