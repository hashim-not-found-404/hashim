use anyhow::Result;
use anyhow::bail;
use database::db_client;
use infrastructure::jwt::Jwt;
use kernel::server::CastDTOToServer;
use kernel::server::TraitOperationServerInput;
use patterns::cast_input_to_server_case;
use patterns::make_server_wrapper_read;
use patterns::make_server_wrapper_write;
use std::any::Any;
use utility::dtos::TypeOperationDTOInput;

make_server_wrapper_write!(create_account);
make_server_wrapper_read!(get_all_accounts);
make_server_wrapper_write!(sign_up);
make_server_wrapper_read!(sign_in);
make_server_wrapper_write!(create_company);
make_server_wrapper_read!(get_companies_and_branches);
make_server_wrapper_write!(create_branch);
make_server_wrapper_write!(create_account_for_branch);
make_server_wrapper_write!(create_journal_entry);

pub(crate) struct MyCaster;

impl CastDTOToServer for MyCaster {
    type Cli = db_client::S;
    type Jwt = Jwt;

    fn cast_input(
        v: TypeOperationDTOInput,
    ) -> Result<Box<dyn TraitOperationServerInput<Cli = Self::Cli, Jwt = Self::Jwt>>> {
        let v: Box<dyn Any> = v;

        cast_input_to_server_case!(get_all_accounts);
        cast_input_to_server_case!(create_account);
        cast_input_to_server_case!(sign_in);
        cast_input_to_server_case!(sign_up);
        cast_input_to_server_case!(create_company);
        cast_input_to_server_case!(get_companies_and_branches);
        cast_input_to_server_case!(create_branch);
        cast_input_to_server_case!(create_account_for_branch);
        cast_input_to_server_case!(create_journal_entry);

        bail!("downcast error")
    }
}
