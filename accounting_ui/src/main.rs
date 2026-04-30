use client_tokio_tungstenite as client;
mod backend;
use crate::backend::MySignal;
use dioxus::{core::spawn_forever, prelude::*};
use dioxus_logger::tracing::{self, Level};
use impls_for_wasm::a1::RandomNumberS;
use my_core::prelude::{Signal, *};
use std::sync::Arc;

const ICONS_SHOW: Asset = asset!("/assets/icons/show.png");
const ICONS_HIDE: Asset = asset!("/assets/icons/hide.png");
const MAIN_CSS: Asset = asset!("/assets/main.css");

#[derive(Clone, PartialEq, Routable)]
enum Route {
    #[route("/")]
    SignIn {},

    #[route("/sign_up")]
    SignUp {},
    // #[route("/home")]
    // Home {},
}

fn main() {
    dioxus_logger::init(Level::INFO).unwrap();
    console_error_panic_hook::set_once();
    dioxus::launch(Initializer);
}

#[component]
fn Initializer() -> Element {
    let init_state = use_resource(|| async {
        let fut = client::Dsdff::new().await;
        let state: Dkdkd = Arc::new(front_end_model_view::State::new(fut));
        use_context_provider(|| state.clone());
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

type Dkdkd = Arc<
    front_end_model_view::State<
        client::Dsdff,
        RandomNumberS,
        MySignal<String>,
        MySignal<bool>,
        MySignal<String>,
        MySignal<db_types::Currency>,
        MySignal<Vec<db_types::Company>>,
    >,
>;

#[component]
fn App() -> Element {
    rsx! {
        // document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Route> {}
        ErrorStack {}
    }
}

#[component]
pub fn SignIn() -> Element {
    let state = consume_context::<Dkdkd>();

    let sign_up = move |_| {
        navigator().push(Route::SignUp {});
    };

    let value = state.clone();
    rsx! {
        div {
            input {
                placeholder: "User ID",
                oninput: move |event| value.user_id.set(event.value()),
                value: state.user_id.read(),
            }
            label {
                {state.sign_in_error_for_user_id.read()}
            }
            PasswordInput{}
            label {
                {state.sign_in_error_for_password.read()}
            }
            button {
                onclick: move |_| {
                    let state = state.clone();
                    tracing::info!("we are inside the click");
                    spawn_forever(async move {
                        tracing::info!("we are inside the future");
                        state.sign_in().await;
                        tracing::info!("the future finish");
                    });
                    tracing::info!("there is no block on , but there was DEADDDDD LOOOOCK ;|");
                },
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
    let state = consume_context::<Dkdkd>();

    let sign_in = move |_| {
        navigator().push(Route::SignIn {});
    };

    let value = state.clone();
    let value1 = state.clone();
    rsx! {
        div {
            input {
                placeholder: "Name (Optional)",
                oninput: move |event| value.user_name.set(event.value()),
                value: state.user_name.read(),
            }
            label {
                {state.sign_up_error_for_user_name.read()}
            }
            input {
                placeholder: "User Id",
                oninput: move |event| value1.user_id.set(event.value()),
                value: state.user_id.read(),
            }
            label {
                {state.sign_up_error_for_user_id.read()}
            }
            PasswordInput{}
            button {
                onclick: move |_| {
                    let state = state.clone();
                    spawn_forever(async move {
                        state.sign_up().await;
                    });
                },
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
    let state = consume_context::<Dkdkd>();

    let mut is_password_visible = use_signal(|| false);

    let (input_type, icon_type) = match is_password_visible.read().clone() {
        true => ("text", ICONS_SHOW),
        false => ("password", ICONS_HIDE),
    };

    let value = state.clone();
    rsx! {
        div {
            input {
                placeholder: "Password",
                type: input_type,
                oninput: move |event| value.password.set(event.value()),
                value: state.password.read(),
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
    let state = consume_context::<Dkdkd>();

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
    rsx! {
        div { "create company page" }
    }
}
