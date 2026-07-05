use crate::my_signal;
use adapters::{
    actors, encode_decode, functions, random_number, row_id, runtime, web_socket_adapter,
};
use cache_rusqlite::cache_adapter;
use my_core::{
    accounting_client::{client_traits::AllClientTypes, client_types},
    accounting_domain::types,
};

#[derive(Default, Clone)]
pub(crate) struct S;

impl AllClientTypes for S {
    type Rn = random_number::target::S;
    type Ws = web_socket_adapter::target::S;
    type Ed = encode_decode::target::S;
    type Rt = runtime::target::S;
    type Ch = cache_adapter::S;
    type Id = row_id::target::S;
    type Mpsc = actors::target::S;
    type Rg = functions::target::S;

    type Uuid = my_signal::S<types::UuidType>;
    type OptionUuid = my_signal::S<Option<types::UuidType>>;
    type Dialog = my_signal::S<client_types::Dialog>;
    type String = my_signal::S<String>;
    type Bool = my_signal::S<bool>;
    type StringVec = my_signal::S<String>;
    type Currency = my_signal::S<types::Currency>;
    type Location = my_signal::S<types::Location>;
    type CompanyAndBranchList = my_signal::S<types::ListOfCompanies>;

    type Navigator = my_signal::S<client_types::Navigator>;
}
