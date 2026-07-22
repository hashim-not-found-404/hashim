use crate::use_cases::home::Home;
use crate::use_cases::list_company_and_branch::ListCompanyAndBranch;
use crate::use_cases::sign_in::SignIn;
use crate::use_cases::sign_up::SignUp;
use crate::utility::components;
use crate::utility::tools;
use dioxus::prelude::*;
use my_core::accounting_client::client_domain::ui_model::HashimSignal;
use my_core::accounting_client::client_domain::ui_model::{self};

#[derive(Clone, PartialEq, Routable)]
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
    match tools::MODEL.navigator.read() {
        ui_model::Navigator::Auth(_) => {
            navigator().push(Route::SignIn {});
        }
        ui_model::Navigator::CompanyBranchSelection(_) => {
            navigator().push(Route::ListCompanyAndBranch {});
        }
        ui_model::Navigator::Home(_) => {
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
        components::ErrorStack {}
    }
}
