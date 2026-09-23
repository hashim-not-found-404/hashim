use crate::navigator::Navigator;
use crate::utils::MODEL;
use crate::utils::init_commander_and_model;
use crate::utils::send;
use dioxus::prelude::*;
use use_case_error_handler::client::LocalModel;
use use_case_error_handler::ui::ErrorStack;
use utility_ui::domain::HashimSignal;

#[derive(Debug, Clone, PartialEq, Routable)]
pub(crate) enum Route {
    #[layout(RootLayout)]
    // #[route("/")]
    // SignIn {},
    // #[route("/sign_up")]
    // SignUp {},
    // #[route("/get_companies_and_branches")]
    // GetCompaniesAndBranches {},
    #[route("/")]
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
    init_commander_and_model();

    rsx! {
        // document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Route> {}
        ErrorStack {
            sender: move |msg| {
                send(msg);
            },
            is_expand_all: MODEL.page_error_handler.is_expand_all().read(),
            errors: MODEL.page_error_handler.errors().read(),
        }
    }
}
