use crate::utility::components;
use crate::utility::tools;
use dioxus::prelude::*;
use my_core::client::utility::ui_model;
use my_core::client::utility::ui_model::HashimSignal;

#[component]
pub(crate) fn SignUp() -> Element {
    let local_state = &tools::MODEL.page_sign_up;

    let go_to_sign_in = move |_| {
        tools::send(ui_model::Message::SignUp(ui_model::SignUp::GoToSignIn));
    };

    let consent_callback = move |consent: ui_model::UserConsent| {
        tools::send(ui_model::Message::SignUp(ui_model::SignUp::Consent(consent)));
    };

    let password_callback = move |password: String| {
        tools::send(ui_model::Message::SignUp(ui_model::SignUp::Password(password)));
    };

    rsx! {
        div {
            components::Dialog {
                consent_callback,
                operation_name: "sign up",
                show_dialog: local_state.show_dialog.clone(),
            }
            input {
                placeholder: "Name (Optional)",
                oninput: move |event| {
                    tools::send(
                        ui_model::Message::SignUp(ui_model::SignUp::UserName(event.value())),
                    );
                },
                value: tools::MODEL.user_name.read(),
            }
            label { {local_state.user_name_error.read()} }
            input {
                placeholder: "User Id",
                oninput: move |event| {
                    tools::send(ui_model::Message::SignUp(ui_model::SignUp::UserId(event.value())));
                },
                value: tools::MODEL.user_id.read(),
            }
            label { {local_state.user_id_error.read()} }
            components::PasswordInput { password_callback }
            button {
                onclick: move |_| {
                    tools::send(ui_model::Message::SignUp(ui_model::SignUp::Submit));
                },
                "Sign Up"
            }
            button { onclick: go_to_sign_in, "Back" }
        }
    }
}
