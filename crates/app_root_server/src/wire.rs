use patterns::make_server_wrapper_read;
use patterns::make_server_wrapper_write;

make_server_wrapper_write!(create_account);
make_server_wrapper_read!(get_all_accounts);
