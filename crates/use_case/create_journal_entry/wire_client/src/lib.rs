pub use use_case_create_journal_entry_client::GlobalModel;
pub use use_case_create_journal_entry_domain::Error;
pub use use_case_create_journal_entry_domain::Input;
pub use use_case_create_journal_entry_domain::Ok;
use use_case_prelude_wire_client::*;

make_client_wrapper_cache!(create_journal_entry);
