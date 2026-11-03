use use_case_prelude_wire_client::*;
pub use use_case_sign_in_domain::Error;
pub use use_case_sign_in_domain::Input;
pub use use_case_sign_in_domain::Ok;

make_client_wrapper_cache!(sign_in);
