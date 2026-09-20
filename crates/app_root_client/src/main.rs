mod model;
mod navigator;

use crate::model::TypeModel;
use crate::navigator::Navigator;
use cache::cache_adapter;
use dioxus::launch;
use dioxus::prelude::*;
use dioxus_logger::init;
use dioxus_logger::tracing::Level;
use kernel::ui_construct;
use kernel::ui_construct::CastDTOToClient;
use std::sync::Arc;
use std::sync::LazyLock;
use utility::cache::CastClientToCache;
use utility::cache::TraitOperationCacheInput;
use utility::cache::TraitOperationCacheOk;
use utility::cache::TraitOperationClientError;
use utility::cache::TraitOperationClientInput;
use utility::cache::TraitOperationClientOk;
use utility::dtos::TypeOperationDTOError;
use utility::dtos::TypeOperationDTOInput;
use utility::dtos::TypeOperationDTOOk;
use utility::ui_effect::CastMessageToUpdater;
use utility::ui_effect::Commander;
use utility::ui_effect::MessageTrait;
use utility::ui_effect::UpdaterTrait;
use utility_ui::components::ErrorStack;
use utility_ui::domain::HashimSignal;

struct MyCaster;

impl CastMessageToUpdater for MyCaster {
    type Mdl = TypeModel;

    fn cast_message_to_updater(v: Box<dyn MessageTrait>) -> Box<dyn UpdaterTrait<Mdl = Self::Mdl>> {
        todo!()
    }
}

impl CastClientToCache for MyCaster {
    type Cache = cache_adapter::S;

    fn cast_input(
        v: &dyn TraitOperationClientInput,
    ) -> &dyn TraitOperationCacheInput<Cache = Self::Cache> {
        todo!()
    }

    fn cast_ok(v: &dyn TraitOperationClientOk) -> &dyn TraitOperationCacheOk<Cache = Self::Cache> {
        todo!()
    }
}

impl CastDTOToClient for MyCaster {
    fn cast_input(v: TypeOperationDTOInput) -> Box<dyn TraitOperationClientInput> {
        todo!()
    }

    fn cast_ok(v: TypeOperationDTOOk) -> Box<dyn TraitOperationClientOk> {
        todo!()
    }

    fn cast_error(v: TypeOperationDTOError) -> Box<dyn TraitOperationClientError> {
        todo!()
    }
}

static MODEL: LazyLock<Arc<TypeModel>> =
    LazyLock::new(|| -> Arc<TypeModel> { Arc::new(TypeModel::default()) });

pub(crate) fn send<Msg: MessageTrait>(msg: Msg) {
    static COMMANDER: LazyLock<Commander> = LazyLock::new(|| {
        let model = MODEL.to_owned();
        ui_construct::new::<cache_adapter::S, TypeModel, MyCaster, MyCaster, MyCaster>(model)
    });

    COMMANDER.send(msg);
}

fn main() {
    init(Level::INFO).unwrap();
    launch(App);
}

#[derive(Debug, Clone, PartialEq, Routable)]
pub(crate) enum Route {
    #[layout(RootLayout)]
    // #[route("/")]
    // SignIn {},
    // #[route("/sign_up")]
    // SignUp {},
    // #[route("/get_companies_and_branches")]
    // GetCompaniesAndBranches {},
    #[route("/home")]
    Home {},
}

#[component]
fn Home() -> Element {
    rsx! {}
}

#[component]
fn RootLayout() -> Element {
    match MODEL.navigator.read() {
        Navigator::SignIn => {
            // navigator().push(Route::SignIn {});
        }
        Navigator::SignUp => {
            // navigator().push(Route::SignUp {});
        }
        Navigator::GetCompaniesAndBranches(_) => {
            // navigator().push(Route::GetCompaniesAndBranches {});
        }
        Navigator::Home(_) => {
            navigator().push(Route::Home {});
        }
    }

    rsx! {
        Outlet::<Route> {}
    }
}

#[component]
pub(crate) fn App() -> Element {
    rsx! {
        // document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Route> {}
        ErrorStack {
           close_error_callback: move |_| {
               // send();
           },
           error: "hash",
        }
    }
}
