use crate::domain::Dialog;
use crate::icons::ICONS_HIDE;
use crate::icons::ICONS_SHOW;
use dioxus::prelude::*;

#[component]
pub fn DialogComponent(
    dont_wait_for_server_response: EventHandler,
    wait_for_server_response: EventHandler,
    cancel_operation: EventHandler,
    operation_name: &'static str,
    show_dialog: Dialog,
) -> Element {
    match show_dialog {
        Dialog::Hide => rsx! {},
        Dialog::Show => {
            rsx! {
                div {
                    label { "do you want to proceed operation {operation_name} offline" }
                    button {
                        onclick: move |_| {
                            dont_wait_for_server_response(());
                        },
                        "Yes"
                    }
                    button {
                        onclick: move |_| {
                            wait_for_server_response(());
                        },
                        "No"
                    }
                    button {
                        onclick: move |_| {
                            cancel_operation(());
                        },
                        "Cancel"
                    }
                }

            }
        }
        Dialog::Error => {
            rsx! {
                label { "sorry you can't proceed now" }
            }
        }
    }
}

#[component]
pub fn PasswordInput(password_callback: EventHandler<String>, password: String) -> Element {
    let mut is_password_visible = use_signal(|| false);

    let (input_type, icon_type) = match *is_password_visible.read() {
        true => ("text", ICONS_SHOW),
        false => ("password", ICONS_HIDE),
    };

    rsx! {
        div {
            input {
                placeholder: "Password",
                r#type: input_type,
                oninput: move |event| password_callback(event.value()),
                value: password,
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
pub fn ErrorStack(close_error_callback: EventHandler, error: String) -> Element {
    if error.is_empty() {
        return rsx!();
    }

    rsx! {
        div {
            button { onclick: move |_| { close_error_callback(()) }, "X" }
            label { {error} }
        }
    }
}
