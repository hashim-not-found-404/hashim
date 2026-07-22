use crate::utility::my_signals;
use crate::utility::tools;
use dioxus::prelude::*;
use my_core::accounting_client::client_domain::ui_model::AllSignalTypes;
use my_core::accounting_client::client_domain::ui_model::HashimSignal;
use my_core::accounting_client::client_domain::ui_model;

#[component]
pub(crate) fn Dialog(
    consent_callback: EventHandler<ui_model::UserConsent>,
    operation_name: &'static str,
    show_dialog: <my_signals::S as AllSignalTypes>::Dialog,
) -> Element {
    let consent_callback1 = consent_callback;

    match show_dialog.read() {
        ui_model::Dialog::Hide => rsx! {},
        ui_model::Dialog::Show => {
            rsx! {
                div {
                    label { "do you want to proceed operation {operation_name} offline" }
                    button { onclick: move |_| { consent_callback(ui_model::UserConsent::DontWaitForServerResponse) },
                        "Yes"
                    }
                    button { onclick: move |_| { consent_callback1(ui_model::UserConsent::WaitForServerResponse) },
                        "No"
                    }
                }

            }
        }
        ui_model::Dialog::Error => {
            rsx! {
                label { "sorry you can't proceed now" }
            }
        }
    }
}

#[component]
pub(crate) fn PasswordInput(password_callback: EventHandler<String>) -> Element {
    let mut is_password_visible = use_signal(|| false);

    let (input_type, icon_type) = match *is_password_visible.read() {
        true => ("text", tools::ICONS_SHOW),
        false => ("password", tools::ICONS_HIDE),
    };

    let password = &tools::MODEL
        .page_root
        .page_auth
        .auth_feature_state
        .user_password;
    rsx! {
        div {
            input {
                placeholder: "Password",
                r#type: input_type,
                oninput: move |event| password_callback(event.value()),
                value: password.read(),
            }
            button {
                onclick: move |_| {
                    *is_password_visible.write() ^= true;
                },
                img { src: icon_type }
            }
        }
    }
}

#[component]
pub(crate) fn ErrorStack() -> Element {
    let err = tools::MODEL.external_errors.read();
    if err.is_empty() {
        return rsx!();
    }

    rsx! {
        div {
            button { onclick: move |_| { tools::send(ui_model::Message::CloseError) },
                "X"
            }
            label { {err} }
        }
    }
}
