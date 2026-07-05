use crate::web_socket_server;
use adapters::{
    actors, authentication, encode_decode, functions, jwt, random_number, row_id, runtime,
};
use db_client_cockroach::{db, db_client};
use my_core::server::server_traits::AllServerTypes;

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
    type Ws = web_socket_server::S;
}
