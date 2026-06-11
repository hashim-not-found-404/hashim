use crate::prelude::*;
use dioxus::prelude::*;
use std::str::FromStr;

type StateOfEveryThing = front_end_model_view::State<
    my_signals::m::S,
    my_types::m::S,
    actors::m::S,
    my_signal::m::S<Option<mpsc_sender::m::S<()>>>,
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
    #[route("/company_and_branch_selection")]
    CompanyAndBranchSelection {},

    #[route("/home")]
    Home {},
}

#[component]
pub fn App() -> Element {
    let state: StateOfEveryThing = front_end_model_view::State::new();
    use_context_provider(|| state);

    rsx! {
        // document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Route> {}
        ErrorStack {}
    }
}

#[component]
fn Dialog(
    sender: my_signal::m::S<Option<mpsc_sender::m::S<()>>>,
    operation_name: &'static str,
    show_dialog: <my_signals::m::S as AllSignalTypes>::Dialog,
) -> Element {
    let show_dialog1 = show_dialog.clone();

    let click = move |_| {
        show_dialog.set(front_end_model_view::Dialog::Hide);
        let mut sender = sender.read().unwrap();
        spawn(async move {
            sender.send(()).await.unwrap();
        });
    };

    match show_dialog1.read() {
        front_end_model_view::Dialog::Hide => rsx! {},
        front_end_model_view::Dialog::Show => {
            rsx! {
                div {
                    label { "do you want to proceed operation {operation_name} offline" }
                    button { onclick: click, "Yes" }
                    button { onclick: move |_| show_dialog1.set(front_end_model_view::Dialog::Hide),
                        "No"
                    }
                }

            }
        }
        front_end_model_view::Dialog::Error => {
            rsx! {
                label { "sorry you can't proceed now" }
            }
        }
    }
}

#[component]
fn AuthenticationPage() -> Element {
    let state = consume_context::<StateOfEveryThing>();
    let auth_state = front_end_model_view::AuthFeatureState::<my_signals::m::S>::default();
    use_context_provider(|| auth_state);

    if state.is_signed_in.read().is_some() {
        navigator().push(Route::CompanyAndBranchSelection {});
    }

    rsx! {
        Outlet::<Route> {}
    }
}

#[component]
fn SignIn() -> Element {
    let state = consume_context::<StateOfEveryThing>();
    let auth_state = consume_context::<front_end_model_view::AuthFeatureState<my_signals::m::S>>();
    let local_state = front_end_model_view::SignInState::<my_signals::m::S>::default();

    let sign_up = move |_| {
        navigator().push(Route::SignUp {});
    };

    let state1 = state.clone();
    let local_state1 = local_state.clone();
    let auth_state1 = auth_state.clone();

    let sender = my_signal::m::S::default();

    rsx! {
        div {
            Dialog {
                sender: sender.clone(),
                operation_name: "sign in",
                show_dialog: local_state.show_dialog.clone(),
            }
            input {
                placeholder: "User ID",
                oninput: move |event| {
                    auth_state1.user_id.set(event.value());
                    state1
                        .clone()
                        .sign_in(
                            my_signal::m::S::default(),
                            false,
                            local_state1.clone(),
                            auth_state1.clone(),
                        );
                },
                value: auth_state.user_id.read(),
            }
            label { {local_state.user_id_error.read()} }
            PasswordInput { password: auth_state.user_password.clone() }
            label { {local_state.user_password_error.read()} }
            button {
                onclick: move |_| {
                    state
                        .clone()
                        .sign_in(sender.clone(), true, local_state.clone(), auth_state.clone())
                },
                "Sign In"
            }
            button { onclick: sign_up, "Sign Up" }
        }
    }
}

#[component]
fn SignUp() -> Element {
    let state = consume_context::<StateOfEveryThing>();
    let auth_state = consume_context::<front_end_model_view::AuthFeatureState<my_signals::m::S>>();
    let local_state = front_end_model_view::SignUpState::<my_signals::m::S>::default();

    let sign_in = move |_| {
        navigator().push(Route::SignIn {});
    };

    let state1 = state.clone();
    let state2 = state.clone();
    let local_state1 = local_state.clone();
    let local_state2 = local_state.clone();
    let auth_state1 = auth_state.clone();
    let auth_state2 = auth_state.clone();

    let sender = my_signal::m::S::default();

    rsx! {
        div {
            Dialog {
                sender: sender.clone(),
                operation_name: "sign up",
                show_dialog: local_state.show_dialog.clone(),
            }
            input {
                placeholder: "Name (Optional)",
                oninput: move |event| {
                    local_state1.user_name.set(event.value());
                    state1
                        .clone()
                        .sign_up(
                            my_signal::m::S::default(),
                            false,
                            local_state1.clone(),
                            auth_state1.clone(),
                        );
                },
                value: local_state.user_name.read(),
            }
            label { {local_state.user_name_error.read()} }
            input {
                placeholder: "User Id",
                oninput: move |event| {
                    auth_state2.user_id.set(event.value());
                    state2
                        .clone()
                        .sign_up(
                            my_signal::m::S::default(),
                            false,
                            local_state2.clone(),
                            auth_state2.clone(),
                        );
                },
                value: auth_state.user_id.read(),
            }
            label { {local_state.user_id_error.read()} }
            PasswordInput { password: auth_state.user_password.clone() }
            button {
                onclick: move |_| {
                    state
                        .clone()
                        .sign_up(sender.clone(), true, local_state.clone(), auth_state.clone())
                },
                "Sign Up"
            }
            button { onclick: sign_in, "Back" }
        }
    }
}

#[component]
fn PasswordInput(password: my_signal::m::S<String>) -> Element {
    let mut is_password_visible = use_signal(|| false);

    let (input_type, icon_type) = match *is_password_visible.read() {
        true => ("text", ICONS_SHOW),
        false => ("password", ICONS_HIDE),
    };

    let password1 = password.clone();
    rsx! {
        div {
            input {
                placeholder: "Password",
                r#type: input_type,
                oninput: move |event| password1.set(event.value()),
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
fn ErrorStack() -> Element {
    let state = consume_context::<StateOfEveryThing>();

    let err = state.external_errors.read();
    if err.is_empty() {
        return rsx!();
    }

    rsx! {
        div {
            button { onclick: move |_| { state.external_errors.set(String::new()) }, "X" }
            label { {err} }
        }
    }
}

#[component]
fn Home() -> Element {
    rsx! {}
}

#[component]
fn CompanyAndBranchSelection() -> Element {
    let state = consume_context::<StateOfEveryThing>();

    let mut show_create_company = use_signal(|| false);
    let mut selected_company_id = use_signal(|| None);

    let local_state = <my_signals::m::S as AllSignalTypes>::CompanyAndBranchList::default();
    state.clone().list_company_and_branch(local_state.clone());

    rsx! {
        div {
            if *show_create_company.read() {
                CreateCompany { show_form: show_create_company }
            }

            button { onclick: move |_| show_create_company.set(true), "Add New Company" }

            div {
                for company in local_state.read() {
                    div {
                        button {
                            onclick: move |_| {
                                if *selected_company_id.read() == Some(company.id.clone()) {
                                    selected_company_id.set(None);
                                } else {
                                    selected_company_id.set(Some(company.id.clone()));
                                }
                            },
                            "{company.name}"
                        }

                        if *selected_company_id.read() == Some(company.id.clone()) {
                            button { "Add Branch" }
                            div {
                                for branch in company.branches {
                                    button {
                                        onclick: {
                                            let selected_company_branch = state.selected_company_branch.clone();
                                            move |_| {
                                                selected_company_branch.set(Some(branch.id.clone()));
                                            }
                                        },
                                        "{branch.name}"
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

#[component]
fn CreateCompany(show_form: Signal<bool>) -> Element {
    let state = consume_context::<StateOfEveryThing>();
    let local_state = front_end_model_view::CreateCompanyState::<my_signals::m::S>::default();

    let local_state1 = local_state.clone();
    let local_state2 = local_state.clone();

    rsx! {
        div {
            input {
                placeholder: "Company Name",
                oninput: move |event| {
                    local_state1.company_name.set(event.value());
                },
                value: local_state.company_name.read(),
            }
            select {
                value: local_state.currency.read().as_str(),
                onchange: move |event| {
                    local_state2
                        .currency
                        .set(db_types::Currency::from_str(event.value().as_str()).unwrap())
                },
                option { value: "USD", "USD" }
                option { value: "IQD", "IQD" }
            }
            button {
                onclick: move |_| {
                    state.clone().create_company(local_state.clone());
                    show_form.set(false);
                },
                "Create"
            }
            button {
                onclick: move |_| {
                    show_form.set(false);
                },
                "X"
            }
        }
    }
}

#[component]
fn CreateCompanyBranch() -> Element {
    let state = consume_context::<StateOfEveryThing>();
    let local_state = front_end_model_view::CreateCompanyBranchState::<my_signals::m::S>::default();

    let state1 = state.clone();
    let local_state1 = local_state.clone();
    let local_state2 = local_state.clone();

    rsx! {
        div {
            input {
                placeholder: "Branch Name",
                oninput: move |event| {
                    local_state1.branch_name.set(event.value());
                    state1.clone().create_company_branch(false, local_state1.clone());
                },
                value: local_state.branch_name.read(),
            }
            select {
                value: local_state.currency.read().as_str(),
                onchange: move |event| {
                    local_state2
                        .currency
                        .set(db_types::Currency::from_str(event.value().as_str()).unwrap())
                },
                option { value: "USD", "USD" }
                option { value: "IQD", "IQD" }
            }
            button { onclick: move |_| state.clone().create_company_branch(true, local_state.clone()),
                "Create"
            }
        }
    }
}
