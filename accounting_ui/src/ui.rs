use crate::prelude::*;
use dioxus::prelude::*;
use std::str::FromStr;

type TheModel = ui_model::Model<my_types::m::S>;
type TheCommander = ui_effect::Commander<my_types::m::S, actors::m::S>;
type TheAll = (TheModel, TheCommander);

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
    MyHome {},
}

#[component]
pub fn App() -> Element {
    let (model, commander): TheAll = ui_construct::new::<my_types::m::S, actors::m::S>();
    use_context_provider(|| (model, commander));

    rsx! {
        // document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Route> {}
        ErrorStack {}
    }
}

#[component]
fn Dialog(
    consent_callback: EventHandler<process_manager::UserConsent>,
    operation_name: &'static str,
    show_dialog: <my_types::m::S as AllClientTypes>::Dialog,
) -> Element {
    let consent_callback1 = consent_callback.clone();

    match show_dialog.read() {
        ui_model::Dialog::Hide => rsx! {},
        ui_model::Dialog::Show => {
            rsx! {
                div {
                    label { "do you want to proceed operation {operation_name} offline" }
                    button {
                        onclick: move |_| {
                            consent_callback(process_manager::UserConsent::DontWaitForServerResponse)
                        },
                        "Yes"
                    }
                    button { onclick: move |_| { consent_callback1(process_manager::UserConsent::WaitForServerResponse) },
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
fn AuthenticationPage() -> Element {
    let (model, _) = consume_context::<TheAll>();

    match model.navigator.read() {
        ui_model::Navigator::Auth(_) => {
            navigator().push(Route::SignIn {});
        }
        ui_model::Navigator::CompanyBranchSelection(_) => {
            navigator().push(Route::CompanyAndBranchSelection {});
        }
        ui_model::Navigator::Home => todo!(),
    }

    rsx! {
        Outlet::<Route> {}
    }
}

#[component]
fn SignIn() -> Element {
    let (model, commander) = consume_context::<TheAll>();
    let auth_state = model.page_root.page_auth.auth_feature_state;
    let local_state = model.page_root.page_auth.page_sign_in;

    let sign_up = move |_| {
        navigator().push(Route::SignUp {});
    };

    let commander1 = commander.clone();
    let consent_callback = move |consent: process_manager::UserConsent| {
        commander1.send(ui_model::Message::SignIn(
            ui_updaters::sign_in::Msg::Consent(consent),
        ));
    };

    let commander1 = commander.clone();
    let password_callback = move |password: String| {
        commander1.send(ui_model::Message::SignIn(
            ui_updaters::sign_in::Msg::Password(password),
        ));
    };

    let commander1 = commander.clone();
    let commander2 = commander.clone();

    rsx! {
        div {
            Dialog {
                consent_callback,
                operation_name: "sign in",
                show_dialog: local_state.show_dialog.clone(),
            }
            input {
                placeholder: "User ID",
                oninput: move |event| {
                    commander1
                        .send(
                            ui_model::Message::SignIn(
                                ui_updaters::sign_in::Msg::UserId(event.value()),
                            ),
                        );
                },
                value: auth_state.user_id.read(),
            }
            label { {local_state.user_id_error.read()} }
            PasswordInput { password_callback }
            label { {local_state.user_password_error.read()} }
            button {
                onclick: move |_| {
                    commander2
                        .send(
                            ui_model::Message::SignIn(ui_updaters::sign_in::Msg::Submit),
                        );
                },
                "Sign In"
            }
            button { onclick: sign_up, "Sign Up" }
        }
    }
}

#[component]
fn SignUp() -> Element {
    let (model, commander) = consume_context::<TheAll>();
    let auth_state = model.page_root.page_auth.auth_feature_state;
    let local_state = model.page_root.page_auth.page_sign_up;

    let sign_in = move |_| {
        navigator().push(Route::SignIn {});
    };

    let commander1 = commander.clone();
    let consent_callback = move |consent: process_manager::UserConsent| {
        commander1.send(ui_model::Message::SignUp(
            ui_updaters::sign_up::Msg::Consent(consent),
        ));
    };

    let commander1 = commander.clone();
    let password_callback = move |password: String| {
        commander1.send(ui_model::Message::SignUp(
            ui_updaters::sign_up::Msg::Password(password),
        ));
    };

    let commander1 = commander.clone();
    let commander2 = commander.clone();
    let commander3 = commander.clone();

    rsx! {
        div {
            Dialog {
                consent_callback,
                operation_name: "sign up",
                show_dialog: local_state.show_dialog.clone(),
            }
            input {
                placeholder: "Name (Optional)",
                oninput: move |event| {
                    commander1
                        .send(
                            ui_model::Message::SignUp(
                                ui_updaters::sign_up::Msg::UserName(event.value()),
                            ),
                        );
                },
                value: local_state.user_name.read(),
            }
            label { {local_state.user_name_error.read()} }
            input {
                placeholder: "User Id",
                oninput: move |event| {
                    commander2
                        .send(
                            ui_model::Message::SignUp(
                                ui_updaters::sign_up::Msg::UserId(event.value()),
                            ),
                        );
                },
                value: auth_state.user_id.read(),
            }
            label { {local_state.user_id_error.read()} }
            PasswordInput { password_callback }
            button {
                onclick: move |_| {
                    commander3
                        .send(
                            ui_model::Message::SignUp(ui_updaters::sign_up::Msg::Submit),
                        );
                },
                "Sign Up"
            }
            button { onclick: sign_in, "Back" }
        }
    }
}

#[component]
fn PasswordInput(password_callback: EventHandler<String>) -> Element {
    let (model, _) = consume_context::<TheAll>();
    let mut is_password_visible = use_signal(|| false);

    let (input_type, icon_type) = match *is_password_visible.read() {
        true => ("text", ICONS_SHOW),
        false => ("password", ICONS_HIDE),
    };

    let password = model.page_root.page_auth.auth_feature_state.user_password;
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
fn ErrorStack() -> Element {
    let (model, commander) = consume_context::<TheAll>();

    let err = model.external_errors.read();
    if err.is_empty() {
        return rsx!();
    }

    rsx! {
        div {
            button { onclick: move |_| { commander.send(ui_model::Message::CloseError) },
                "X"
            }
            label { {err} }
        }
    }
}

#[component]
fn MyHome() -> Element {
    rsx! {}
}

#[component]
fn CompanyAndBranchSelection() -> Element {
    let (model, commander) = consume_context::<TheAll>();

    let local_state = model
        .page_root
        .page_after_auth
        .page_company_branch_selection
        .list;

    let selected_company = model
        .page_root
        .page_after_auth
        .page_company_branch_selection
        .selected_company;

    let commander1 = commander.clone();

    rsx! {
        div {
            match model.navigator.read() {
                ui_model::Navigator::CompanyBranchSelection(n) => {
                    match n {
                        ui_model::CompanyBranchSelection::None => rsx! {},
                        ui_model::CompanyBranchSelection::CreateCompany => rsx! {
                            CreateCompany {}
                        },
                        ui_model::CompanyBranchSelection::CreateCompanyBranch => rsx! {
                            CreateCompanyBranch {}
                        },
                    }
                }
                _ => rsx! {},
            }

            button {
                onclick: move |_| {
                    commander1
                        .send(
                            ui_model::Message::CompanyAndBranchSelection(
                                ui_updaters::company_and_branch_selection::Msg::ShowCreateCompany,
                            ),
                        )
                },
                "Add New Company"
            }

            div {
                for company in local_state.read() {
                    {
                        let commander2 = commander.clone();
                        let commander3 = commander.clone();
                        rsx! {
                            button {
                                onclick: move |_| {
                                    commander2
                                        .send(
                                            ui_model::Message::CompanyAndBranchSelection(
                                                ui_updaters::company_and_branch_selection::Msg::SelectedCompany(
                                                    company.uuid.clone(),
                                                ),
                                            ),
                                        );
                                },
                                "{company.name}"
                            }

                            if selected_company.read() == Some(company.uuid.clone()) {
                                button {
                                    onclick: move |_| {
                                        commander3
                                            .send(
                                                ui_model::Message::CompanyAndBranchSelection(
                                                    ui_updaters::company_and_branch_selection::Msg::ShowCreateCompanyBranch,
                                                ),
                                            )
                                    },
                                    "Add New Branch"
                                }
                                div {
                                    for branch in company.branches {
                                        {
                                            let commander4 = commander.clone();
                                            rsx! {
                                                button {
                                                    onclick: {
                                                        move |_| {
                                                            commander4
                                                                .send(
                                                                    ui_model::Message::CompanyAndBranchSelection(
                                                                        ui_updaters::company_and_branch_selection::Msg::SelectedCompanyBranch(
                                                                            branch.uuid.clone(),
                                                                        ),
                                                                    ),
                                                                )
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
        }
    }
}

#[component]
fn CreateCompany() -> Element {
    let (model, commander) = consume_context::<TheAll>();
    let local_state = model
        .page_root
        .page_after_auth
        .page_company_branch_selection
        .page_create_company;

    let commander1 = commander.clone();
    let commander2 = commander.clone();
    let commander3 = commander.clone();

    rsx! {
        div {
            input {
                placeholder: "Company Name",
                oninput: move |event| {
                    commander1
                        .send(
                            ui_model::Message::CreateCompany(
                                ui_updaters::create_company::Msg::Name(event.value()),
                            ),
                        );
                },
                value: local_state.company_name.read(),
            }
            select {
                value: local_state.currency.read().as_str(),
                onchange: move |event| {
                    commander2
                        .send(
                            ui_model::Message::CreateCompany(
                                ui_updaters::create_company::Msg::Currency(event.value()),
                            ),
                        );
                },
                option { value: "USD", "USD" }
                option { value: "IQD", "IQD" }
            }
            button {
                onclick: move |_| {
                    commander3
                        .send(
                            ui_model::Message::CreateCompany(
                                ui_updaters::create_company::Msg::Submit,
                            ),
                        );
                },
                "Create"
            }
            button {
                onclick: move |_| {
                    commander
                        .send(
                            ui_model::Message::CreateCompany(ui_updaters::create_company::Msg::Close),
                        );
                },
                "X"
            }
        }
    }
}

#[component]
fn CreateCompanyBranch() -> Element {
    let (model, commander) = consume_context::<TheAll>();
    let local_state = model
        .page_root
        .page_after_auth
        .page_company_branch_selection
        .page_create_company_branch;

    let commander1 = commander.clone();
    let consent_callback = move |consent: process_manager::UserConsent| {
        commander1.send(ui_model::Message::CreateCompanyBranch(
            ui_updaters::create_company_branch::Msg::Consent(consent),
        ));
    };

    let commander2 = commander.clone();
    let commander3 = commander.clone();
    let commander4 = commander.clone();

    rsx! {
        div {
            Dialog {
                consent_callback,
                operation_name: "create company branch",
                show_dialog: local_state.show_dialog.clone(),
            }
            input {
                placeholder: "Branch Name",
                oninput: move |event| {
                    commander2
                        .send(
                            ui_model::Message::CreateCompanyBranch(
                                ui_updaters::create_company_branch::Msg::Name(event.value()),
                            ),
                        );
                },
                value: local_state.branch_name.read(),
            }
            select {
                value: local_state.currency.read().as_str(),
                onchange: move |event| {
                    commander3
                        .send(
                            ui_model::Message::CreateCompanyBranch(
                                ui_updaters::create_company_branch::Msg::Currency(event.value()),
                            ),
                        );
                },
                option { value: "USD", "USD" }
                option { value: "IQD", "IQD" }
            }
            button {
                onclick: move |_| {
                    commander4
                        .send(
                            ui_model::Message::CreateCompanyBranch(
                                ui_updaters::create_company_branch::Msg::Submit,
                            ),
                        );
                },
                "Create"
            }
            button {
                onclick: move |_| {
                    commander
                        .send(
                            ui_model::Message::CreateCompanyBranch(
                                ui_updaters::create_company_branch::Msg::Close,
                            ),
                        );
                },
                "X"
            }
        }
    }
}
