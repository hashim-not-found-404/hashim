use crate::domain::Dialog;
use crate::domain::HashimSignal;
use crate::icons::ICONS_HIDE;
use crate::icons::ICONS_SHOW;
use dioxus::prelude::*;

#[component]
pub(crate) fn DialogComponent<Signal: HashimSignal<Dialog> + PartialEq + 'static + Clone>(
    dont_wait_for_server_response: EventHandler,
    wait_for_server_response: EventHandler,
    cancel_operation: EventHandler,
    operation_name: &'static str,
    show_dialog: Signal,
) -> Element {
    match show_dialog.read() {
        Dialog::Hide => rsx! {},
        Dialog::Show => {
            rsx! {
                div {
                    label { "do you want to proceed operation {operation_name} offline" }
                    button { onclick: move |_| { dont_wait_for_server_response(()); },
                        "Yes"
                    }
                    button { onclick: move |_| { wait_for_server_response(()); },
                        "No"
                    }
                    button { onclick: move |_| { cancel_operation(()); },
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
pub(crate) fn PasswordInput<Password: HashimSignal<String> + 'static + Clone + PartialEq>(
    password_callback: EventHandler<String>,
    password: Password,
) -> Element {
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
pub(crate) fn ErrorStack<ExternalErrors: HashimSignal<String> + 'static + PartialEq + Clone>(
    close_error_callback: EventHandler,
    error: ExternalErrors,
) -> Element {
    let err = error.read();
    if err.is_empty() {
        return rsx!();
    }

    rsx! {
        div {
            button { onclick: move |_| { close_error_callback(()) },
                "X"
            }
            label { {err} }
        }
    }
}
