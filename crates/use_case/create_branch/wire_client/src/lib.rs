pub use use_case_create_branch_domain::Error;
pub use use_case_create_branch_domain::Input;
pub use use_case_create_branch_domain::Ok;
use use_case_prelude_wire_client::*;

make_client_wrapper_cache!(create_branch);
