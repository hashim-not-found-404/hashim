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
pub fn PasswordInput(
    password_callback: EventHandler<String>,
    disabled: bool,
    password: String,
) -> Element {
    let mut is_password_visible = use_signal(|| false);

    let (input_type, icon_type) = match *is_password_visible.read() {
        true => ("text", ICONS_SHOW),
        false => ("password", ICONS_HIDE),
    };

    rsx! {
        div {
            input {
                disabled: disabled,
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
pub fn ListInput<T: PartialEq + Clone + 'static>(
    disabled: bool,
    placeholder: String,
    selected_item: String,
    on_input: EventHandler<String>,
    on_select: EventHandler<usize>,
    list: Vec<T>,
    row_renderer: Callback<T, Element>,
) -> Element {
    let mut is_open = use_signal(|| false);

    rsx! {
        div {
            input {
                disabled,
                placeholder: "{placeholder}",
                value: selected_item,
                onfocus: move |_| is_open.set(true),
                onblur: move |_| is_open.set(false),
                oninput: move |event| {
                    let v = event.value();
                    is_open.set(true);
                    on_input.call(v);
                },
            }

            if is_open() {
                if list.is_empty() {
                    ul {
                        li { "no items" }
                    }
                } else {
                    ul {
                        for (idx, item) in list.iter().enumerate() {
                            {
                                rsx! {
                                    li {
                                        key: "{idx}",
                                        onmousedown: move |event| {
                                            event.prevent_default();
                                            is_open.set(false);
                                            on_select.call(idx);
                                        },
                                        {row_renderer.call(item.clone())}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
