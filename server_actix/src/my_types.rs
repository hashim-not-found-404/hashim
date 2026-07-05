pub mod target {
    use crate::prelude::*;

    pub struct S;
    impl AllServerTypes for S {
        type Db = db::S;
        type Cli = db_client::S;
        type Jwt = jwt::target::S;
        type Auth = authentication::target::S;
        type Rg = functions::target::S;
        type Id = row_id::target::S;
        type Mpsc = actors::target::S;
        type Rt = runtime::target::S;
        type Ed = encode_decode::target::S;
        type Rn = random_number::target::S;
        type Ws = web_socket_server::target::S;
    }
}
