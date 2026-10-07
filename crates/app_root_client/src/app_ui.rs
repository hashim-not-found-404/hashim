use crate::navigator::Intent as NavIntent;
use crate::navigator::Message;
use crate::navigator::Navigator;
use crate::utils::MODEL;
use crate::utils::init_commander_and_model;
use crate::utils::send;
use cache::utils::MyUuidConverter;
use dioxus::prelude::*;
use std::ops::Deref;
use use_case_create_account::client::LocalModel as _;
use use_case_create_branch::client::LocalModel as _;
use use_case_create_company::client::LocalModel as _;
use use_case_error_handler::client::LocalModel as _;
use use_case_sign_in::client::LocalModel as _;
use use_case_sign_up::client::LocalModel as _;
use utility::process_manager::ProcessId;
use utility_ui::domain::HashimSignal;

#[derive(Debug, Clone, PartialEq, Routable)]
pub(crate) enum Route {
    #[layout(RootLayout)]
    #[route("/")]
    SignIn {},
    #[route("/sign_up")]
    SignUp {},
    // #[route("/get_companies_and_branches")]
    // GetCompaniesAndBranches {},
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
        Navigator::GetCompaniesAndBranches(_) => {
            // navigator().push(Route::GetCompaniesAndBranches {});
        }
    }

    if MODEL.user_uuid().is_some() {
        navigator().push(Route::Home {});
    }

    rsx! {
        Outlet::<Route> {}
    }
}

#[component]
pub(crate) fn App() -> Element {
    init_commander_and_model();

    let process_id = use_hook(ProcessId::default);

    rsx! {
        Router::<Route> {}
        use_case_error_handler::ui::Component {
            sender: move |i| send(process_id, use_case_error_handler::client::Message::Intent(i)),
            is_expand_all: MODEL.page_error_handler.is_expand_all().read(),
            errors: MODEL.page_error_handler.errors().read(),
        }
    }
}

#[component]
fn Home() -> Element {
    let process_id = use_hook(ProcessId::default);
    let process_id1 = use_hook(ProcessId::default);
    let process_id2 = use_hook(ProcessId::default);
    let process_id3 = use_hook(ProcessId::default);

    rsx! {
        use_case_select_default_company::ui::Component {
            sender: move |i| send(
                process_id,
                use_case_select_default_company::client::Message::Intent(i),
            ),
            user_name: MODEL
                .user_name
                .read()
                .or_else(|| MODEL.user_id.read().into())
                .unwrap_or_default(),
            companies: use_case_select_default_company::client::list_of_companies(
                MODEL.page_select_default_company.as_ref(),
            ),
            branches: use_case_select_default_company::client::list_of_branches(
                MODEL.page_select_default_company.as_ref(),
                MODEL.as_ref(),
            ),
            selected_company: MODEL.selected_company_uuid.read(),
            selected_company_name: use_case_select_default_company::client::selected_company_name(
                MODEL.page_select_default_company.as_ref(),
                MODEL.as_ref(),
            ),
            selected_branch: MODEL.selected_company_branch_uuid.read(),
            selected_branch_name: use_case_select_default_company::client::selected_company_branch_name(
                MODEL.page_select_default_company.as_ref(),
                MODEL.as_ref(),
            ),
        }
        use_case_create_company::ui::Component {
            sender: move |i| send(process_id1, use_case_create_company::client::Message::Intent(i)),
            show_dialog: MODEL.page_create_company.show_dialog().read(),
            is_loading: use_case_create_company::client::is_loading(
                MODEL.page_create_company.as_ref(),
            ),
            company_name: MODEL.page_create_company.company_name().read(),
            currency: MODEL.page_create_company.currency().read(),
            company_name_error: use_case_create_company::client::error_company_name(
                MODEL.page_create_company.as_ref(),
            )
            .map(|e| format!("{e:?}")),
        }
        use_case_create_branch::ui::Component {
            sender: move |i| send(process_id2, use_case_create_branch::client::Message::Intent(i)),
            show_dialog: MODEL.page_create_branch.show_dialog().read(),
            is_loading: MODEL.page_create_branch.is_loading().read(),
            selected_company_name: MODEL.page_create_branch.selected_company_name().read(),
            company_name_error: MODEL.page_create_branch.company_name_error().read(),
            selected_company_uuid: MODEL
                .page_create_branch
                .selected_company_uuid()
                .read()
                .map(|a| { a.into_inner().to_string() })
                .unwrap_or_default(),
            branch_name: MODEL.page_create_branch.branch_name().read(),
            currency: MODEL.page_create_branch.currency().read(),
            location: MODEL.page_create_branch.location().read(),
            branch_name_error: MODEL
                .page_create_branch
                .branch_name_error()
                .read()
                .map(|e| format!("{e:?}")),
            location_error: MODEL
                .page_create_branch
                .location_error()
                .read()
                .map(|e| format!("{e:?}")),
            list_of_company_name_and_uuid: MODEL
                .page_create_branch
                .list_of_companies_to_display()
                .read(),
        }
        use_case_create_account::ui::Component {
            sender: move |i| send(process_id3, use_case_create_account::client::Message::Intent(i)),
            show_dialog: MODEL.page_create_account.show_dialog().read(),
            is_loading: MODEL.page_create_account.is_loading().read(),
            is_debit: MODEL.page_create_account.is_debit().read(),
            is_permanent_account: MODEL.page_create_account.is_permanent_account().read(),
            account_name: MODEL.page_create_account.account_name().read(),
            unit_of_measurement_of_quantity: MODEL
                .page_create_account
                .unit_of_measurement_of_quantity()
                .read(),
            account_name_error: MODEL
                .page_create_account
                .account_name_error()
                .read()
                .map(|e| format!("{e:?}")),
            selected_company_name: MODEL.page_create_account.selected_company_name().read(),
            selected_company_uuid: MODEL
                .page_create_account
                .selected_company_uuid()
                .read()
                .map(|a| a.into_inner().to_string())
                .unwrap_or_default(),
            company_name_error: MODEL.page_create_account.company_name_error().read(),
            list_of_company_name_and_uuid: MODEL
                .page_create_account
                .list_of_companies_to_display()
                .read(),
        }
    }
}

#[component]
fn SignIn() -> Element {
    let process_id = use_hook(ProcessId::default);
    let process_id1 = use_hook(ProcessId::default);

    rsx! {
        use_case_sign_in::ui::Component {
            sender: move |i| send(process_id, use_case_sign_in::client::Message::Intent(i)),
            on_go_to_sign_up: move || send(process_id1, Message::Intent(NavIntent::GoToSignUp)),
            show_dialog: MODEL.page_sign_in.show_dialog().read(),
            user_id: MODEL.user_id.read(),
            password: MODEL.feature_state_auth.user_password.read(),
            error_user_id: use_case_sign_in::client::error_user_id(
                    MODEL.page_sign_in.deref(),
                )
                .map(|e| format!("{e:?}")),
            error_password: use_case_sign_in::client::error_password(
                    MODEL.page_sign_in.deref(),
                )
                .map(|e| format!("{e:?}")),
            is_loading: use_case_sign_in::client::is_auth_loading(
                MODEL.page_sign_in.deref(),
            ),
        }
    }
}

#[component]
fn SignUp() -> Element {
    let process_id = use_hook(ProcessId::default);
    let process_id1 = use_hook(ProcessId::default);

    rsx! {
        use_case_sign_up::ui::Component {
            sender: move |i| send(process_id, use_case_sign_up::client::Message::Intent(i)),
            on_back: move || send(process_id1, Message::Intent(NavIntent::GoToSignIn)),
            show_dialog: MODEL.page_sign_up.show_dialog().read(),
            user_id: MODEL.user_id.read(),
            user_name: MODEL.user_name.read(),
            password: MODEL.feature_state_auth.user_password.read(),
            error_user_id: use_case_sign_up::client::error_user_id(
                    MODEL.page_sign_up.deref(),
                )
                .map(|e| format!("{e:?}")),
            error_user_name: use_case_sign_up::client::error_user_name(
                    MODEL.page_sign_up.deref(),
                )
                .map(|e| format!("{e:?}")),
            is_loading: use_case_sign_up::client::is_auth_loading(
                    MODEL.page_sign_up.deref(),
                ),
        }
    }
}
