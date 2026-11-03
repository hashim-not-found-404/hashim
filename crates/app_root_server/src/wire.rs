use anyhow::Result;
use anyhow::bail;
use database::db_client::DbClientAdapter;
use infrastructure::jwt::Jwt;
use kernel::server::CastDTOToServer;
use kernel::server::TraitOperationServerInput;
use std::any::Any;
use utility::dtos::TypeOperationDTOInput;

macro_rules! cast_input_to_server_case {
    ($v:expr, $crate_name:ident) => {
        if let Some(v) = $v.downcast_ref::<$crate_name::Input>() {
            return Ok(Box::new($crate_name::Wrapper(v.clone())));
        }
    };
}

pub(crate) struct MyCaster;

impl CastDTOToServer for MyCaster {
    type Cli = DbClientAdapter;
    type Jwt = Jwt;

    fn cast_input(
        v: TypeOperationDTOInput,
    ) -> Result<Box<dyn TraitOperationServerInput<Cli = Self::Cli, Jwt = Self::Jwt>>> {
        let v: Box<dyn Any> = v;

        cast_input_to_server_case!(v, use_case_create_account_wire_server);
        cast_input_to_server_case!(v, use_case_create_account_for_branch_wire_server);
        cast_input_to_server_case!(v, use_case_create_branch_wire_server);
        cast_input_to_server_case!(v, use_case_create_company_wire_server);
        cast_input_to_server_case!(v, use_case_create_journal_wire_server);
        cast_input_to_server_case!(v, use_case_create_journal_entry_wire_server);
        cast_input_to_server_case!(v, use_case_get_all_accounts_wire_server);
        cast_input_to_server_case!(v, use_case_get_all_journals_wire_server);
        cast_input_to_server_case!(v, use_case_get_companies_and_branches_wire_server);
        cast_input_to_server_case!(v, use_case_sign_in_wire_server);
        cast_input_to_server_case!(v, use_case_sign_up_wire_server);

        bail!("downcast error")
    }
}
