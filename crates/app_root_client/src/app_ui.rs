use crate::navigator::Navigator;
use crate::utils::MODEL;
use crate::utils::init_commander_and_model;
use crate::utils::send;
use dioxus::prelude::*;
use use_case_create_account::client::LocalModel as a;
use use_case_error_handler::client::LocalModel as b;
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
        use_case_error_handler::ui::Component {
            sender: move |msg| {
                send(msg);
            },
            is_expand_all: MODEL.page_error_handler.is_expand_all().read(),
            errors: MODEL.page_error_handler.errors().read(),
        }
    }
}

#[component]
fn Home() -> Element {
    rsx! {
        use_case_create_account::ui::Component {
            sender: move |msg| {
                send(msg);
            },
            show_dialog: MODEL.page_create_account.show_dialog().read(),
            is_loading: MODEL.page_create_account.is_loading().read(),
            is_debit: MODEL.page_create_account.is_debit().read(),
            is_permanent_account: MODEL.page_create_account.is_permanent_account().read(),
            account_name: MODEL.page_create_account.account_name().read(),
            notes: MODEL.page_create_account.notes().read(),
            unit_of_measurement_of_quantity: MODEL.page_create_account.unit_of_measurement_of_quantity().read(),
            account_name_error: MODEL.page_create_account.account_name_error().read(),
        }
    }
}
