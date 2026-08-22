use crate::utility::tools;
use dioxus::prelude::*;
use my_core::accounting_client::client_domain::ui_model;

#[component]
pub(crate) fn CreateJournalEntry() -> Element {
    use_effect(move || {
        tools::send(ui_model::Message::CreateJournalEntry(ui_model::CreateJournalEntry::Subscribe));
    });

    use_drop(move || {
        tools::send(ui_model::Message::CreateJournalEntry(
            ui_model::CreateJournalEntry::UnSubscribe,
        ));
    });

    rsx! {}
}
