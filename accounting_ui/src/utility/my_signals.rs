use crate::utility::my_signal;
use my_core::accounting_client::client_domain::ui_model;
use my_core::accounting_client::client_domain::ui_model::AllSignalTypes;
use my_core::accounting_domain::utility::types;

#[derive(Default, Clone)]
pub(crate) struct S;

impl AllSignalTypes for S {
    type Bool = my_signal::S<bool>;
    type CompanyAndBranchList = my_signal::S<types::ListOfCompanies>;
    type Currency = my_signal::S<types::Currency>;
    type Dialog = my_signal::S<ui_model::Dialog>;
    type Location = my_signal::S<types::Location>;
    type Navigator = my_signal::S<ui_model::Navigator>;
    type OptionString = my_signal::S<Option<String>>;
    type OptionUuid = my_signal::S<Option<types::UuidType>>;
    type String = my_signal::S<String>;
    type StringVec = my_signal::S<String>;
    type Uuid = my_signal::S<types::UuidType>;
}
