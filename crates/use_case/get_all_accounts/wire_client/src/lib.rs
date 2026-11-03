pub use use_case_get_all_accounts_domain::Error;
pub use use_case_get_all_accounts_domain::Input;
pub use use_case_get_all_accounts_domain::Ok;
use use_case_prelude_wire_client::*;

make_client_wrapper_cache!(get_all_accounts);
