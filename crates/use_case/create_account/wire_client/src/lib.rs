pub use use_case_create_account_domain::Error;
pub use use_case_create_account_domain::Input;
pub use use_case_create_account_domain::Ok;
use use_case_prelude_wire_client::*;

make_client_wrapper_cache!(create_account);
