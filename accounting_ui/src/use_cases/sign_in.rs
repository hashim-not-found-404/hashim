use crate::utility::components;
use crate::utility::tools;
use dioxus::prelude::*;
use my_core::accounting_client::client_domain::ui_model;
use my_core::accounting_client::client_domain::ui_model::HashimSignal;

#[component]
pub(crate) fn SignIn() -> Element {
    let local_state = &tools::MODEL.page_sign_in;

    let go_to_sign_up = move |_| {
        tools::send(ui_model::Message::SignIn(ui_model::SignIn::GoToSignUp));
    };

    let consent_callback = move |consent: ui_model::UserConsent| {
        tools::send(ui_model::Message::SignIn(ui_model::SignIn::Consent(consent)));
    };

    let password_callback = move |password: String| {
        tools::send(ui_model::Message::SignIn(ui_model::SignIn::Password(password)));
    };

    rsx! {
        div {
            components::Dialog {
                consent_callback,
                operation_name: "sign in",
                show_dialog: local_state.show_dialog.clone(),
            }
            input {
                placeholder: "User ID",
                oninput: move |event| {
                    tools::send(ui_model::Message::SignIn(ui_model::SignIn::UserId(event.value())));
                },
                value: tools::MODEL.user_id.read(),
            }
            label { {local_state.user_id_error.read()} }
            components::PasswordInput { password_callback }
            label { {local_state.user_password_error.read()} }
            button {
                onclick: move |_| {
                    tools::send(ui_model::Message::SignIn(ui_model::SignIn::Submit));
                },
                "Sign In"
            }
            button { onclick: go_to_sign_up, "Sign Up" }
        }
    }
}
