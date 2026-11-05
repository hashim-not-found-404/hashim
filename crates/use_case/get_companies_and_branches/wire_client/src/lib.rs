pub use use_case_get_companies_and_branches_domain::Error;
pub use use_case_get_companies_and_branches_domain::Input;
pub use use_case_get_companies_and_branches_domain::Ok;
use use_case_prelude_wire_client::*;

make_client_wrapper_cache!(get_companies_and_branches);
