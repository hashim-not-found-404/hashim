use crate::utility::my_signal;
use accounting_engine::accounting_stuff;
use my_core::client::utility::ui_model;
use my_core::client::utility::ui_model::AllSignalTypes;
use my_core::domain::utility::types::Currency;
use my_core::domain::utility::types::ListOfCompanies;
use my_core::domain::utility::types::Location;
use my_core::domain::utility::uuid::UuidType;

#[derive(Debug, Clone, Default)]
pub(crate) struct S;

impl AllSignalTypes for S {
    type AccountsSuggestionList = my_signal::S<Vec<ui_model::Account>>;
    type Bool = my_signal::S<bool>;
    type CompanyAndBranchList = my_signal::S<ListOfCompanies>;
    type Currency = my_signal::S<Currency>;
    type Dialog = my_signal::S<ui_model::Dialog>;
    type InFlowType = my_signal::S<accounting_stuff::InFlowType>;
    type JournalEntry = my_signal::S<Vec<ui_model::DoubleEntry>>;
    type Location = my_signal::S<Location>;
    type Navigator = my_signal::S<ui_model::Navigator>;
    type OptionString = my_signal::S<Option<String>>;
    type OptionUuid = my_signal::S<Option<UuidType>>;
    type OutFlowType = my_signal::S<accounting_stuff::OutFlowType>;
    type String = my_signal::S<String>;
    type StringVec = my_signal::S<String>;
    type Uuid = my_signal::S<UuidType>;
}
