pub mod m {
    use crate::prelude::*;

    pub struct S;
    impl AllServerTypes for S {
        type Db = db::S;
        type Cli = db_client::S;
        type Jwt = jwt::m::S;
        type Auth = authentication::m::S;
        type Rg = functions::m::S;
        type Id = row_id::m::S;
        type Mpsc = actors::m::S;
        type Rt = runtime::m::S;
        type De = encode_decode::m::S;
        type Rn = random_number::m::S;
        type Ws = web_socket_server::m::S;
    }
}
