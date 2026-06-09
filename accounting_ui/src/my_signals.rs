pub mod m {
    use crate::prelude::*;

    #[derive(Default, Clone)]
    pub struct S;
    impl AllSignalTypes for S {
        type OptionRowId = my_signal::m::S<Option<db_types::UuidType>>;
        type Dialog = my_signal::m::S<front_end_model_view::Dialog>;
        type String = my_signal::m::S<String>;
        type Bool = my_signal::m::S<bool>;
        type StringVec = my_signal::m::S<String>;
        type Currency = my_signal::m::S<db_types::Currency>;
        type Location = my_signal::m::S<db_types::Location>;
    }
}
