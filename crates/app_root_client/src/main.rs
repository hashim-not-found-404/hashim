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
use kernel::ui_construct::Casting;
use std::sync::Arc;
use std::sync::LazyLock;
use utility::cache::CacheCaster;
use utility::cache::OpErrorTrait;
use utility::cache::OpInputTrait;
use utility::cache::OpInputTrait1;
use utility::cache::OpOkTrait;
use utility::cache::OpOkTrait1;
use utility::dtos::TypeOperationsError;
use utility::dtos::TypeOperationsInput;
use utility::dtos::TypeOperationsOk;
use utility::ui_effect::Caster;
use utility::ui_effect::Commander;
use utility::ui_effect::MessageTrait;
use utility::ui_effect::UpdaterTrait;
use utility_ui::components::ErrorStack;
use utility_ui::domain::HashimSignal;

struct MyCaster;

impl Caster for MyCaster {
    type Mdl = TypeModel;

    fn cast_message_to_updater(v: Box<dyn MessageTrait>) -> Box<dyn UpdaterTrait<Mdl = Self::Mdl>> {
        todo!()
    }
}

impl CacheCaster for MyCaster {
    type Cache = cache_adapter::S;

    fn cast_input(v: &dyn OpInputTrait) -> &dyn OpInputTrait1<Cache = Self::Cache> {
        todo!()
    }

    fn cast_ok(v: &dyn OpOkTrait) -> &dyn OpOkTrait1<Cache = Self::Cache> {
        todo!()
    }
}

impl Casting for MyCaster {
    fn cast_input(v: TypeOperationsInput) -> Box<dyn OpInputTrait> {
        todo!()
    }

    fn cast_ok(v: TypeOperationsOk) -> Box<dyn OpOkTrait> {
        todo!()
    }

    fn cast_error(v: TypeOperationsError) -> Box<dyn OpErrorTrait> {
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
