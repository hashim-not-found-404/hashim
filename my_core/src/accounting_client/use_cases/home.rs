use crate::{
    accounting_client::use_cases::client_domain::{
        client_traits, commander,
        ui_model::{self, HashimSignal},
    },
    accounting_domain::cases::utility::types,
    utility::traits,
};
use std::sync::Arc;

impl ui_model::Home {
    pub(crate) async fn update<
        Rn: traits::RandomNumber,
        Rt: traits::Runtime,
        Id: types::RowId,
        Mpsc: traits::MultiProducerSingleConsumer,
        Rg: traits::Regex,
        As: ui_model::AllSignalTypes,
    >(
        self,
        model: &'static ui_model::Model<As>,
        _: client_traits::CacheActorStruct<Mpsc>,
        _: Arc<commander::CommanderLocalState<Mpsc, As>>,
    ) {
        match self {
            ui_model::Home::ShowDashboard => model
                .navigator
                .set(ui_model::Navigator::Home(ui_model::Menu::Dashboard)),
            ui_model::Home::ShowCreateAccount => model
                .navigator
                .set(ui_model::Navigator::Home(ui_model::Menu::CreateAccount)),
        }
    }
}
