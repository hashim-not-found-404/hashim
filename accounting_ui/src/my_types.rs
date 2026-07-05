pub mod target {
    use crate::prelude::*;

    #[derive(Default, Clone)]
    pub struct S;

    impl AllClientTypes for S {
        type Rn = random_number::target::S;
        type Ws = web_socket_adapter::target::S;
        type Ed = encode_decode::target::S;
        type Rt = runtime::target::S;
        type Ch = cache_adapter::S;
        type Id = row_id::target::S;
        type Mpsc = actors::target::S;
        type Rg = functions::target::S;

        type Uuid = my_signal::target::S<db_types::UuidType>;
        type OptionUuid = my_signal::target::S<Option<db_types::UuidType>>;
        type Dialog = my_signal::target::S<ui_model::Dialog>;
        type String = my_signal::target::S<String>;
        type Bool = my_signal::target::S<bool>;
        type StringVec = my_signal::target::S<String>;
        type Currency = my_signal::target::S<db_types::Currency>;
        type Location = my_signal::target::S<db_types::Location>;
        type CompanyAndBranchList = my_signal::target::S<db_types::ListOfCompanies>;

        type Navigator = my_signal::target::S<ui_model::Navigator>;
    }
}
