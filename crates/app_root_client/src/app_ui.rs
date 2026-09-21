use crate::cast::MyCaster;
use crate::model::TypeModel;
use crate::navigator::Navigator;
use cache::cache_adapter;
use dioxus::prelude::*;
use kernel::ui_construct;
use std::sync::Arc;
use std::sync::LazyLock;
use utility::ui_effect::Commander;
use utility::ui_effect::MessageTrait;
use utility_ui::components::ErrorStack;
use utility_ui::domain::HashimSignal;

static MODEL: LazyLock<Arc<TypeModel>> =
    LazyLock::new(|| -> Arc<TypeModel> { Arc::new(TypeModel::default()) });

pub(crate) fn send<Msg: MessageTrait>(msg: Msg) {
    static COMMANDER: LazyLock<Commander> = LazyLock::new(|| {
        let model = MODEL.to_owned();
        ui_construct::new::<cache_adapter::S, TypeModel, MyCaster, MyCaster, MyCaster>(model)
    });

    COMMANDER.send(msg);
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
