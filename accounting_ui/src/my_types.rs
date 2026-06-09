pub mod m {
    use crate::prelude::*;

    pub struct S;
    impl AllClientTypes for S {
        type Rn = random_number::m::S;
        type Ws = web_socket_adapter::m::S;
        type Ed = encode_decode::m::S;
        type Rt = runtime::m::S;
        type Ch = cache_adapter::S;
        type Id = row_id::m::S;
    }
}
