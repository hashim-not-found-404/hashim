mod backend;
use crate::backend::{MyAllSignals, MySignal};
use adapters::prelude::*;
use cache_rusqlite::prelude::*;
use dioxus::prelude::*;
use dioxus_logger::tracing::Level;
use my_core::prelude::{Signal as HashimSignal, *};
use std::{str::FromStr, sync::Arc};

type StateOfEveryThing = Arc<
    front_end_model_view::State<
        random_number::m::S,
        web_socket_adapter::m::S,
        encode_decode::m::S,
        runtime::m::S,
        cache_adapter::S,
        actors::m::S,
        row_id::m::S,
        MyAllSignals,
    >,
>;

const ICONS_SHOW: Asset = asset!("/assets/icons/show.png");
const ICONS_HIDE: Asset = asset!("/assets/icons/hide.png");
const MAIN_CSS: Asset = asset!("/assets/main.css");

#[derive(Clone, PartialEq, Routable)]
enum Route {
    #[layout(AuthenticationPage)]
    #[route("/")]
    SignIn {},
    #[route("/sign_up")]
    SignUp {},
    #[end_layout]
    #[route("/home")]
    Home {},
}

fn main() {
    dioxus_logger::init(Level::INFO).unwrap();
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let state: StateOfEveryThing = front_end_model_view::State::new();
    use_context_provider(|| state);

    rsx! {
        // document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Route> {}
        ErrorStack {}
    }
}

#[component]
pub fn Dialog(operation_name: &'static str, show_dialog: MySignal<bool>) -> Element {
    let show_dialog1 = show_dialog.clone();

    if show_dialog.read() {
        return rsx! {
            div {
                label { "do you want to proceed operation {operation_name} offline" }
                button {
                    onclick: move |_| show_dialog.set(false),
                    "Yes"
                }
                button {
                    onclick: move |_| show_dialog1.set(false),
                    "No"
                }
            }

        };
    }
    rsx! {}
}

#[component]
pub fn AuthenticationPage() -> Element {
    let state = consume_context::<StateOfEveryThing>();
    let auth_state = Arc::new(front_end_model_view::AuthFeatureState::<MyAllSignals>::default());
    use_context_provider(|| auth_state);

    if state.is_signed_in.read() {
        navigator().push(Route::Home {});
    }

    rsx! {
        Outlet::<Route> {}
    }
}

#[component]
pub fn SignIn() -> Element {
    let state = consume_context::<StateOfEveryThing>();
    let auth_state = consume_context::<Arc<front_end_model_view::AuthFeatureState<MyAllSignals>>>();
    let local_state = Arc::new(front_end_model_view::SignInState::<MyAllSignals>::default());

    let sign_up = move |_| {
        navigator().push(Route::SignUp {});
    };

    let state1 = state.clone();
    let local_state1 = local_state.clone();
    let auth_state1 = auth_state.clone();

    rsx! {
        div {
            input {
                placeholder: "User ID",
                oninput: move |event| {
                    auth_state1.user_id.set(event.value());
                    state1.clone().sign_in(false,local_state1.clone(),auth_state1.clone());
                },
                value: auth_state.user_id.read(),
            }
            label {
                {local_state.user_id_error.read()}
            }
            PasswordInput{ password: auth_state.user_password.clone() }
            label {
                {local_state.user_password_error.read()}
            }
            button {
                onclick: move |_| state.clone().sign_in(true,local_state.clone(),auth_state.clone()),
                "Sign In"
            }
            button {
                onclick: sign_up,
                "Sign Up"
            }
        }
    }
}

#[component]
pub fn SignUp() -> Element {
    let state = consume_context::<StateOfEveryThing>();
    let auth_state = consume_context::<Arc<front_end_model_view::AuthFeatureState<MyAllSignals>>>();
    let local_state = Arc::new(front_end_model_view::SignUpState::<MyAllSignals>::default());

    let sign_in = move |_| {
        navigator().push(Route::SignIn {});
    };

    let state1 = state.clone();
    let state2 = state.clone();
    let local_state1 = local_state.clone();
    let local_state2 = local_state.clone();
    let auth_state1 = auth_state.clone();
    let auth_state2 = auth_state.clone();

    rsx! {
        div {
            Dialog {
                operation_name:"sign up",
                show_dialog:local_state.show_dialog.clone()
            }
            input {
                placeholder: "Name (Optional)",
                oninput: move |event| {
                    local_state1.user_name.set(event.value());
                    state1.clone().sign_up(false,local_state1.clone(),auth_state1.clone());
                },
                value: local_state.user_name.read(),
            }
            label {
                {local_state.user_name_error.read()}
            }
            input {
                placeholder: "User Id",
                oninput: move |event| {
                    auth_state2.user_id.set(event.value());
                    state2.clone().sign_up(false,local_state2.clone(),auth_state2.clone());
                },
                value: auth_state.user_id.read(),
            }
            label {
                {local_state.user_id_error.read()}
            }
            PasswordInput{ password: auth_state.user_password.clone() }
            button {
                onclick: move |_| state.clone().sign_up(true,local_state.clone(),auth_state.clone()),
                "Sign Up"
            }
            button {
                onclick: sign_in,
                "Back"
            }
        }
    }
}

#[component]
pub fn PasswordInput(password: MySignal<String>) -> Element {
    let mut is_password_visible = use_signal(|| false);

    let (input_type, icon_type) = match is_password_visible.read().clone() {
        true => ("text", ICONS_SHOW),
        false => ("password", ICONS_HIDE),
    };

    let password1 = password.clone();
    rsx! {
        div {
            input {
                placeholder: "Password",
                type: input_type,
                oninput: move |event| password1.set(event.value()),
                value: password.read(),
            }
            button {
                onclick: move |_| {*is_password_visible.write()^=true;},
                img {src:icon_type},
            }
        }
    }
}

#[component]
pub fn ErrorStack() -> Element {
    let state = consume_context::<StateOfEveryThing>();

    let err = state.external_errors.read();
    if err.is_empty() {
        return rsx!();
    }

    rsx! {
        div {
            button {
                onclick: move |_| {state.external_errors.set(String::new())},
                "X"
            }
            label {
                {err}
            }
        }
    }
}

#[component]
pub fn Home() -> Element {
    rsx! {
        CreateCompany {}
    } // TODO display after sign
}

#[component]
pub fn CreateCompany() -> Element {
    let state = consume_context::<StateOfEveryThing>();
    let local_state = Arc::new(front_end_model_view::CreateCompanyState::<MyAllSignals>::default());

    let state1 = state.clone();
    let local_state1 = local_state.clone();
    let local_state2 = local_state.clone();

    rsx! {
        div {
            input {
                placeholder: "Company Name",
                oninput: move |event| {
                    local_state1.company_name.set(event.value());
                    state1.clone().create_company(false,local_state1.clone());
                },
                value: local_state.company_name.read(),
            }
            select {
                value: local_state.currency.read().as_str(),
                onchange: move |event| local_state2.currency.set(db_types::Currency::from_str(event.value().as_str()).unwrap()),
                option { value: "USD", "USD" }
                option { value: "IQD", "IQD" }
            }
            button {
                onclick: move |_| state.clone().create_company(true,local_state.clone()),
                "Create"
            }
        }
    }
}

#[component]
pub fn CreateCompanyBranch() -> Element {
    let state = consume_context::<StateOfEveryThing>();
    let local_state =
        Arc::new(front_end_model_view::CreateCompanyBranchState::<MyAllSignals>::default());

    let state1 = state.clone();
    let local_state1 = local_state.clone();
    let local_state2 = local_state.clone();

    rsx! {
        div {
            input {
                placeholder: "Branch Name",
                oninput: move |event| {
                    local_state1.branch_name.set(event.value());
                    state1.clone().create_company_branch(false,local_state1.clone());
                },
                value: local_state.branch_name.read(),
            }
            select {
                value: local_state.currency.read().as_str(),
                onchange: move |event| local_state2.currency.set(db_types::Currency::from_str(event.value().as_str()).unwrap()),
                option { value: "USD", "USD" }
                option { value: "IQD", "IQD" }
            }
            button {
                onclick: move |_| state.clone().create_company_branch(true,local_state.clone()),
                "Create"
            }
        }
    }
}
