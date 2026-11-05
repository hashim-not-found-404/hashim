pub use use_case_get_all_journals_domain::Error;
pub use use_case_get_all_journals_domain::Input;
pub use use_case_get_all_journals_domain::Ok;
use use_case_prelude_wire_client::*;

make_client_wrapper_cache!(get_all_journals);
