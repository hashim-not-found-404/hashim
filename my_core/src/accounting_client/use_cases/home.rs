use crate::accounting_client::client_domain::client_traits;
use crate::accounting_client::client_domain::commander;
use crate::accounting_client::client_domain::ui_model;
use crate::accounting_client::client_domain::ui_model::HashimSignal;
use crate::utility::traits;
use std::sync::Arc;

impl ui_model::Home {
    pub(crate) async fn update<
        Mpsc: traits::MultiProducerSingleConsumer,
        As: ui_model::AllSignalTypes,
    >(
        self,
        model: &'static ui_model::Model<As>,
        _: client_traits::CacheActorStruct<Mpsc>,
        _: Arc<commander::CommanderLocalState<Mpsc, As>>,
    ) {
        match self {
            ui_model::Home::ShowDashboard => {
                model.navigator.set(ui_model::Navigator::Home(ui_model::HomeNav {
                    show_menu:       false,
                    page_to_present: ui_model::Menu::Dashboard,
                }))
            }
            ui_model::Home::ShowCreateAccount => {
                model.navigator.set(ui_model::Navigator::Home(ui_model::HomeNav {
                    show_menu:       false,
                    page_to_present: ui_model::Menu::CreateAccount,
                }))
            }
            ui_model::Home::ShowCreateAccountForBranch => {
                model.navigator.set(ui_model::Navigator::Home(ui_model::HomeNav {
                    show_menu:       false,
                    page_to_present: ui_model::Menu::CreateAccountForBranch,
                }));
            }
            ui_model::Home::ShowCreateJournalEntry => {
                model.navigator.set(ui_model::Navigator::Home(ui_model::HomeNav {
                    show_menu:       false,
                    page_to_present: ui_model::Menu::CreateJournalEntry,
                }));
            }
        }
    }
}
