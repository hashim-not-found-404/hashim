use patterns::make_client_wrapper_cache_check;
use patterns::make_client_wrapper_cache_write;
use patterns::make_client_wrapper_updater;

make_client_wrapper_updater!(create_account);
make_client_wrapper_cache_check!(create_account);
make_client_wrapper_cache_write!(create_account);

make_client_wrapper_cache_check!(get_all_accounts);
make_client_wrapper_cache_write!(get_all_accounts);
