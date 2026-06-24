pub mod m {
    use crate::prelude::*;

    #[derive(Default, Clone)]
    pub struct S;
    impl AllSignalTypes for S {
        type Uuid = my_signal::m::S<db_types::UuidType>;
        type OptionUuid = my_signal::m::S<Option<db_types::UuidType>>;
        type Dialog = my_signal::m::S<ui_model::Dialog>;
        type String = my_signal::m::S<String>;
        type Bool = my_signal::m::S<bool>;
        type StringVec = my_signal::m::S<String>;
        type Currency = my_signal::m::S<db_types::Currency>;
        type Location = my_signal::m::S<db_types::Location>;
        type CompanyAndBranchList = my_signal::m::S<db_types::ListOfCompanies>;

        type Navigator = my_signal::m::S<ui_model::Navigator>;
    }
}
