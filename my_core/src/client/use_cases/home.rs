use crate::client::utility::ui_model;
use crate::client::utility::ui_model::HashimSignal;

impl ui_model::Home {
    pub(crate) fn update<As: ui_model::AllSignalTypes>(self, model: &'static ui_model::Model<As>) {
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
