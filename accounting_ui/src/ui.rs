use crate::use_cases::home::MyHome;
use crate::use_cases::list_company_and_branch::CompanyAndBranchSelection;
use crate::use_cases::sign_in::SignIn;
use crate::use_cases::sign_up::SignUp;
use crate::utils::components;
use crate::utils::tools;
use dioxus::prelude::*;
use my_core::accounting_client::use_cases::client_domain::ui_model::{self, HashimSignal};

#[derive(Clone, PartialEq, Routable)]
pub(crate) enum Route {
    #[layout(RootLayout)]
    #[route("/")]
    SignIn {},
    #[route("/sign_up")]
    SignUp {},
    #[route("/company_and_branch_selection")]
    CompanyAndBranchSelection {},
    #[route("/home")]
    MyHome {},
}

#[component]
fn RootLayout() -> Element {
    match tools::MODEL.navigator.read() {
        ui_model::Navigator::Auth(_) => {
            navigator().push(Route::SignIn {});
        }
        ui_model::Navigator::CompanyBranchSelection(_) => {
            navigator().push(Route::CompanyAndBranchSelection {});
        }
        ui_model::Navigator::Home(_) => {
            navigator().push(Route::MyHome {});
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
