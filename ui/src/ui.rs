use crate::use_cases::home::Home;
use crate::use_cases::list_company_and_branch::ListCompanyAndBranch;
use crate::use_cases::sign_in::SignIn;
use crate::use_cases::sign_up::SignUp;
use crate::utility::components::ErrorStack;
use crate::utility::tools::MODEL;
use dioxus::prelude::*;
use my_core::client::utility::ui_model::HashimSignal;
use my_core::client::utility::ui_model::Navigator;

#[derive(Debug, Clone, PartialEq, Routable)]
pub(crate) enum Route {
    #[layout(RootLayout)]
    #[route("/")]
    SignIn {},
    #[route("/sign_up")]
    SignUp {},
    #[route("/list_company_and_branch")]
    ListCompanyAndBranch {},
    #[route("/home")]
    Home {},
}

#[component]
fn RootLayout() -> Element {
    match MODEL.navigator.read() {
        Navigator::SignIn => {
            navigator().push(Route::SignIn {});
        }
        Navigator::SignUp => {
            navigator().push(Route::SignUp {});
        }
        Navigator::ListCompanyAndBranch(_) => {
            navigator().push(Route::ListCompanyAndBranch {});
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
        ErrorStack {}
    }
}
