mod backend;
use crate::backend::MySignal;
use adapters::prelude::*;
use dioxus::prelude::*;
use dioxus_logger::tracing::Level;
use my_core::prelude::{Signal, *};
use std::{str::FromStr, sync::Arc};

type TypeMyWAMP = web_socket::MyWAMP<
    web_socket_adapter::m::S,
    encode_decode::m::S,
    random_number::m::S,
    runtime::m::S,
    actors::m::S,
>;

type StateOfEveryThing = Arc<
    front_end_model_view::State<
        TypeMyWAMP,
        runtime::m::S,
        actors::m::S,
        cache::m::S,
        row_id::m::S,
        random_number::m::S,
        MySignal<String>,
        MySignal<bool>,
        MySignal<String>,
        MySignal<db_types::Currency>,
        MySignal<db_types::Location>,
    >,
>;

const ICONS_SHOW: Asset = asset!("/assets/icons/show.png");
const ICONS_HIDE: Asset = asset!("/assets/icons/hide.png");
const MAIN_CSS: Asset = asset!("/assets/main.css");

#[derive(Clone, PartialEq, Routable)]
enum Route {
    #[route("/")]
    AuthenticationPage {}, // This can be a landing page or redirect    #[route("/sign_in")]

    #[layout(AuthenticationPage)] // ← Layout provides AuthFeatureState
    #[route("/sign_in")]
    SignIn {},
    #[route("/sign_up")]
    SignUp {},
    // #[end_layout]
    // #[route("/home")]
    // Home {},
}

fn main() {
    dioxus_logger::init(Level::INFO).unwrap();
    // console_error_panic_hook::set_once();
    dioxus::launch(Initializer);
}

#[component]
fn Initializer() -> Element {
    let init_state = use_resource(|| async {
        let state: StateOfEveryThing = front_end_model_view::State::new().await;
        use_context_provider(|| state);
    });

    match (init_state.value())() {
        Some(_) => {
            rsx! {
                App {}
            }
        }
        None => rsx! {
            div { "Initializing application..." }
        },
    }
}

#[component]
fn App() -> Element {
    rsx! {
        // document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Route> {}
        ErrorStack {}
        CreateCompany {}
    }
}

#[component]
pub fn AuthenticationPage() -> Element {
    let auth_state = Arc::new(front_end_model_view::AuthFeatureState {
        user_id: MySignal::<String>::default(),
        user_password: MySignal::<String>::default(),
        is_loading: MySignal::<bool>::default(),
    });
    use_context_provider(|| auth_state);

    rsx! {
        Outlet::<Route> {}
    }
    // SignIn()
}

#[component]
pub fn SignIn() -> Element {
    let state = consume_context::<StateOfEveryThing>();
    let auth_state = consume_context::<
        Arc<front_end_model_view::AuthFeatureState<MySignal<String>, MySignal<bool>>>,
    >();

    let local_state = Arc::new(front_end_model_view::SignInState {
        user_id_error: MySignal::<String>::default(),
        user_password_error: MySignal::<String>::default(),
    });

    let sign_up = move |_| {
        navigator().push(Route::SignUp {});
    };

    let auth_state1 = auth_state.clone();

    rsx! {
        div {
            input {
                placeholder: "User ID",
                oninput: move |event| auth_state1.user_id.set(event.value()),
                value: auth_state.user_id.read(),
            }
            label {
                {local_state.user_id_error.read()}
            }
            PasswordInput{ }
            label {
                {local_state.user_password_error.read()}
            }
            button {
                onclick: move |_| state.clone().sign_in(local_state.clone(),auth_state.clone()),
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
    let auth_state = consume_context::<
        Arc<front_end_model_view::AuthFeatureState<MySignal<String>, MySignal<bool>>>,
    >();

    let local_state = Arc::new(front_end_model_view::SignUpState {
        user_name: MySignal::<String>::default(),
        user_id_error: MySignal::<String>::default(),
        user_name_error: MySignal::<String>::default(),
    });

    let sign_in = move |_| {
        navigator().push(Route::SignIn {});
    };

    let local_state1 = local_state.clone();
    let auth_state1 = auth_state.clone();

    rsx! {
        div {
            input {
                placeholder: "Name (Optional)",
                oninput: move |event| local_state1.user_name.set(event.value()),
                value: local_state.user_name.read(),
            }
            label {
                {local_state.user_name_error.read()}
            }
            input {
                placeholder: "User Id",
                oninput: move |event| auth_state1.user_id.set(event.value()),
                value: auth_state.user_id.read(),
            }
            label {
                {local_state.user_id_error.read()}
            }
            PasswordInput{ }
            button {
                onclick: move |_| state.clone().sign_up(local_state.clone(),auth_state.clone()),
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
pub fn PasswordInput() -> Element {
    let auth_state = consume_context::<
        Arc<front_end_model_view::AuthFeatureState<MySignal<String>, MySignal<bool>>>,
    >();

    let mut is_password_visible = use_signal(|| false);

    let (input_type, icon_type) = match is_password_visible.read().clone() {
        true => ("text", ICONS_SHOW),
        false => ("password", ICONS_HIDE),
    };

    let password = auth_state.user_password.clone();
    let password1 = auth_state.user_password.clone();
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
pub fn CreateCompany() -> Element {
    let state = consume_context::<StateOfEveryThing>();

    let local_state = Arc::new(front_end_model_view::CreateCompanyState {
        company_name: MySignal::<String>::default(),
        currency: MySignal::<db_types::Currency>::default(),
    });

    let local_state1 = local_state.clone();
    let local_state2 = local_state.clone();

    rsx! {
        div {
            input {
                placeholder: "Company Name",
                oninput: move |event| local_state1.company_name.set(event.value()),
                value: local_state.company_name.read(),
            }
            select {
                value: local_state.currency.read().as_str(),
                onchange: move |event| local_state2.currency.set(db_types::Currency::from_str(event.value().as_str()).unwrap()),
                option { value: "USD", "USD" }
                option { value: "IQD", "IQD" }
            }
            button {
                onclick: move |_| state.clone().create_company(local_state.clone()),
                "Create"
            }
        }
    }
}
