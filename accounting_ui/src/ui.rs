use crate::prelude::*;
use dioxus::prelude::*;
use std::str::FromStr;

type StateOfEveryThing = front_end_model_view::State<
    my_signals::m::S,
    my_types::m::S,
    actors::m::S,
    my_signal::m::S<Option<mpsc_sender::m::S<front_end_model_view::IsProceed>>>,
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
    sender: my_signal::m::S<Option<mpsc_sender::m::S<front_end_model_view::IsProceed>>>,
    operation_name: &'static str,
    show_dialog: <my_signals::m::S as AllSignalTypes>::Dialog,
) -> Element {
    let show_dialog1 = show_dialog.clone();

    let click = move |s: front_end_model_view::IsProceed| {
        show_dialog.set(front_end_model_view::Dialog::Hide);
        let mut sender = sender.read().unwrap();
        spawn(async move {
            sender.send(s).await.unwrap();
        });
    };

    let click1 = click.clone();

    match show_dialog1.read() {
        front_end_model_view::Dialog::Hide => rsx! {},
        front_end_model_view::Dialog::Show => {
            rsx! {
                div {
                    label { "do you want to proceed operation {operation_name} offline" }
                    button { onclick: move |_| click(front_end_model_view::IsProceed::Yes),
                        "Yes"
                    }
                    button { onclick: move |_| click1(front_end_model_view::IsProceed::No),
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

    let local_state = front_end_model_view::SignInState::<my_signals::m::S>::default();
    use_context_provider(|| local_state);

    let local_state = front_end_model_view::SignUpState::<my_signals::m::S>::default();
    use_context_provider(|| local_state);

    let sender: my_signal::m::S<Option<mpsc_sender::m::S<front_end_model_view::IsProceed>>> =
        my_signal::m::S::default();
    use_context_provider(|| sender);

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
    let local_state = consume_context::<front_end_model_view::SignInState<my_signals::m::S>>();
    let sender = consume_context::<
        my_signal::m::S<Option<mpsc_sender::m::S<front_end_model_view::IsProceed>>>,
    >();

    let sign_up = move |_| {
        navigator().push(Route::SignUp {});
    };

    let state1 = state.clone();
    let local_state1 = local_state.clone();
    let auth_state1 = auth_state.clone();

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
    let local_state = consume_context::<front_end_model_view::SignUpState<my_signals::m::S>>();
    let sender = consume_context::<
        my_signal::m::S<Option<mpsc_sender::m::S<front_end_model_view::IsProceed>>>,
    >();

    let sign_in = move |_| {
        navigator().push(Route::SignIn {});
    };

    let state1 = state.clone();
    let state2 = state.clone();
    let local_state1 = local_state.clone();
    let local_state2 = local_state.clone();
    let auth_state1 = auth_state.clone();
    let auth_state2 = auth_state.clone();

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

enum ActiveForm {
    None,
    CreateCompany,
    CreateCompanyBranch,
}

#[component]
fn CompanyAndBranchSelection() -> Element {
    let state = consume_context::<StateOfEveryThing>();

    let mut show_active_form = use_signal(|| ActiveForm::None);
    let mut selected_company_id = use_signal(|| None);

    let local_state = <my_signals::m::S as AllSignalTypes>::CompanyAndBranchList::default();

    let state1 = state.clone();
    let local_state1 = local_state.clone();

    let mut cleanup = use_signal(|| None);

    use_effect(move || {
        let c = state1
            .clone()
            .list_company_and_branch_listener(local_state1.clone());
        state1.clone().list_company_and_branch(local_state1.clone());

        cleanup.set(Some(c));
    });

    use_drop(move || {
        if let Some(c) = cleanup.take() {
            c();
        }
    });

    rsx! {
        div {
            match *show_active_form.read() {
                ActiveForm::None => rsx! {},
                ActiveForm::CreateCompany => rsx! {
                    CreateCompany { show_form: show_active_form }
                },
                ActiveForm::CreateCompanyBranch => rsx! {
                    CreateCompanyBranch { show_form: show_active_form , company_uuid: selected_company_id.read().clone().unwrap()}
                },
            }
            button { onclick: move |_| show_active_form.set(ActiveForm::CreateCompany),
                "Add New Company"
            }

            div {
                for company in local_state.read() {
                    div {
                        button {
                            onclick: move |_| {
                                if *selected_company_id.read() == Some(company.uuid.clone()) {
                                    selected_company_id.set(None);
                                } else {
                                    selected_company_id.set(Some(company.uuid.clone()));
                                }
                            },
                            "{company.name}"
                        }

                        if *selected_company_id.read() == Some(company.uuid.clone()) {
                            button { onclick: move |_| show_active_form.set(ActiveForm::CreateCompanyBranch),
                                "Add New Branch"
                            }
                            div {
                                for branch in company.branches {
                                    button {
                                        onclick: {
                                            let selected_company_branch = state.selected_company_branch.clone();
                                            move |_| {
                                                selected_company_branch.set(Some(branch.uuid.clone()));
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
fn CreateCompany(show_form: Signal<ActiveForm>) -> Element {
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
                    show_form.set(ActiveForm::None);
                },
                "Create"
            }
            button {
                onclick: move |_| {
                    show_form.set(ActiveForm::None);
                },
                "X"
            }
        }
    }
}

#[component]
fn CreateCompanyBranch(show_form: Signal<ActiveForm>, company_uuid: db_types::UuidType) -> Element {
    let state = consume_context::<StateOfEveryThing>();
    let local_state = front_end_model_view::CreateCompanyBranchState::<my_signals::m::S>::default();

    let sender = my_signal::m::S::default();

    let state1 = state.clone();
    let local_state1 = local_state.clone();
    let local_state2 = local_state.clone();

    local_state2.company_belong.set(company_uuid.clone());

    rsx! {
        div {
            Dialog {
                sender: sender.clone(),
                operation_name: "create company branch",
                show_dialog: local_state.show_dialog.clone(),
            }
            input {
                placeholder: "Branch Name",
                oninput: move |event| {
                    local_state1.branch_name.set(event.value());
                    state1
                        .clone()
                        .create_company_branch(
                            my_signal::m::S::default(),
                            false,
                            local_state1.clone(),
                        );
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
            button {
                onclick: move |_| {
                    state.clone().create_company_branch(sender.clone(), true, local_state.clone());
                    show_form.set(ActiveForm::None);
                },
                "Create"
            }
            button {
                onclick: move |_| {
                    show_form.set(ActiveForm::None);
                },
                "X"
            }
        }
    }
}
