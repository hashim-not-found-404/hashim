mod backend;

use dioxus::{core::spawn_forever, prelude::*};
use dioxus_logger::tracing::Level;
use my_core::front_end_model_view::Signal;

const ICONS_SHOW: Asset = asset!("/assets/icons/show.png");
const ICONS_HIDE: Asset = asset!("/assets/icons/hide.png");
const MAIN_CSS: Asset = asset!("/assets/main.css");

#[derive(Clone, PartialEq, Routable)]
enum Route {
    #[route("/")]
    SignIn {},

    #[route("/sign_up")]
    SignUp {},
}

fn main() {
    dioxus_logger::init(Level::INFO).unwrap();
    dioxus::launch(App);
}

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
    let sign_up = move |_| {
        navigator().push(Route::SignUp {});
    };

    rsx! {
        div {
            input {
                placeholder: "User ID",
                oninput: move |event| backend::STATE.user_id.set(event.value()),
                value: backend::STATE.user_id.read(),
            }
            label {
                {backend::STATE.sign_in_error_for_user_id.read()}
            }
            PasswordInput{}
            label {
                {backend::STATE.sign_in_error_for_password.read()}
            }
            button {
                onclick: |_| {spawn_forever(backend::STATE.sign_in());},
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
    let sign_in = move |_| {
        navigator().push(Route::SignIn {});
    };

    rsx! {
        div {
            input {
                placeholder: "Name (Optional)",
                oninput: move |event| backend::STATE.user_name.set(event.value()),
                value: backend::STATE.user_name.read(),
            }
            label {
                {backend::STATE.sign_up_error_for_user_name.read()}
            }
            input {
                placeholder: "User Id",
                oninput: move |event| backend::STATE.user_id.set(event.value()),
                value: backend::STATE.user_id.read(),
            }
            label {
                {backend::STATE.sign_up_error_for_user_id.read()}
            }
            PasswordInput{}
            button {
                onclick: |_| {spawn_forever(backend::STATE.sign_up());},
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
    let mut is_password_visible = use_signal(|| false);

    let (input_type, icon_type) = match is_password_visible.read().clone() {
        true => ("text", ICONS_SHOW),
        false => ("password", ICONS_HIDE),
    };

    rsx! {
        div {
            input {
                placeholder: "Password",
                type: input_type,
                oninput: move |event| backend::STATE.password.set(event.value()),
                value: backend::STATE.password.read(),
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
    let err = backend::STATE.external_errors.read();
    if err.is_empty() {
        return rsx!();
    }

    rsx! {
        div {
            button {
                onclick: |_| {backend::STATE.external_errors.set(String::new())},
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
    rsx! {}
}
