use my_core::accounting_client::client_domain::ui_model::AllSignalTypes;
use my_core::accounting_client::client_domain::ui_model::{self};
use my_core::accounting_domain::utility::types;

use crate::utility::my_signal;

#[derive(Default, Clone)]
pub(crate) struct S;

impl AllSignalTypes for S {
    type Uuid = my_signal::S<types::UuidType>;
    type OptionString = my_signal::S<Option<String>>;
    type OptionUuid = my_signal::S<Option<types::UuidType>>;
    type Dialog = my_signal::S<ui_model::Dialog>;
    type String = my_signal::S<String>;
    type Bool = my_signal::S<bool>;
    type StringVec = my_signal::S<String>;
    type Currency = my_signal::S<types::Currency>;
    type Location = my_signal::S<types::Location>;
    type CompanyAndBranchList = my_signal::S<types::ListOfCompanies>;

    type Navigator = my_signal::S<ui_model::Navigator>;
}
