use crate::my_signals;
use adapters::{
    actors, encode_decode, functions, random_number, row_id, runtime, web_socket_adapter,
};
use dioxus::prelude::*;
use my_core::accounting_client::{
    ui_construct, ui_effect,
    use_cases::client_domain::ui_model::{self, AllSignalTypes, HashimSignal},
};
use std::sync::LazyLock;

type TheModel = ui_model::Model<my_signals::S>;
type TheCommander = ui_effect::Commander<actors::target::S>;

static MODEL: LazyLock<TheModel> = LazyLock::new(TheModel::default);
static COMMANDER: LazyLock<TheCommander> = LazyLock::new(|| {
    ui_construct::new::<
        random_number::target::S,
        runtime::target::S,
        row_id::target::S,
        actors::target::S,
        encode_decode::target::S,
        functions::target::S,
        cache_rusqlite::cache_adapter::S,
        web_socket_adapter::target::S,
        my_signals::S,
    >(&MODEL)
});

fn send(msg: ui_model::Message) {
    COMMANDER.send::<runtime::target::S>(msg);
}

const ICONS_SHOW: Asset = asset!("/assets/icons/show.png");
const ICONS_HIDE: Asset = asset!("/assets/icons/hide.png");
// const MAIN_CSS: Asset = asset!("/assets/main.css");

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
pub(crate) fn App() -> Element {
    rsx! {
        // document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Route> {}
        ErrorStack {}
    }
}

#[component]
fn Dialog(
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
fn AuthenticationPage() -> Element {
    match MODEL.navigator.read() {
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
    let auth_state = &MODEL.page_root.page_auth.auth_feature_state;
    let local_state = &MODEL.page_root.page_auth.page_sign_in;

    let sign_up = move |_| {
        navigator().push(Route::SignUp {});
    };

    let consent_callback = move |consent: ui_model::UserConsent| {
        send(ui_model::Message::SignIn(ui_model::SignIn::Consent(
            consent,
        )));
    };

    let password_callback = move |password: String| {
        send(ui_model::Message::SignIn(ui_model::SignIn::Password(
            password,
        )));
    };

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
                    send(ui_model::Message::SignIn(ui_model::SignIn::UserId(event.value())));
                },
                value: auth_state.user_id.read(),
            }
            label { {local_state.user_id_error.read()} }
            PasswordInput { password_callback }
            label { {local_state.user_password_error.read()} }
            button {
                onclick: move |_| {
                    send(ui_model::Message::SignIn(ui_model::SignIn::Submit));
                },
                "Sign In"
            }
            button { onclick: sign_up, "Sign Up" }
        }
    }
}

#[component]
fn SignUp() -> Element {
    let auth_state = &MODEL.page_root.page_auth.auth_feature_state;
    let local_state = &MODEL.page_root.page_auth.page_sign_up;

    let sign_in = move |_| {
        navigator().push(Route::SignIn {});
    };

    let consent_callback = move |consent: ui_model::UserConsent| {
        send(ui_model::Message::SignUp(ui_model::SignUp::Consent(
            consent,
        )));
    };

    let password_callback = move |password: String| {
        send(ui_model::Message::SignUp(ui_model::SignUp::Password(
            password,
        )));
    };

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
                    send(ui_model::Message::SignUp(ui_model::SignUp::UserName(event.value())));
                },
                value: local_state.user_name.read(),
            }
            label { {local_state.user_name_error.read()} }
            input {
                placeholder: "User Id",
                oninput: move |event| {
                    send(ui_model::Message::SignUp(ui_model::SignUp::UserId(event.value())));
                },
                value: auth_state.user_id.read(),
            }
            label { {local_state.user_id_error.read()} }
            PasswordInput { password_callback }
            button {
                onclick: move |_| {
                    send(ui_model::Message::SignUp(ui_model::SignUp::Submit));
                },
                "Sign Up"
            }
            button { onclick: sign_in, "Back" }
        }
    }
}

#[component]
fn PasswordInput(password_callback: EventHandler<String>) -> Element {
    let mut is_password_visible = use_signal(|| false);

    let (input_type, icon_type) = match *is_password_visible.read() {
        true => ("text", ICONS_SHOW),
        false => ("password", ICONS_HIDE),
    };

    let password = &MODEL.page_root.page_auth.auth_feature_state.user_password;
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
    let err = MODEL.external_errors.read();
    if err.is_empty() {
        return rsx!();
    }

    rsx! {
        div {
            button { onclick: move |_| { send(ui_model::Message::CloseError) }, "X" }
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
    let local_state = &MODEL
        .page_root
        .page_after_auth
        .page_company_branch_selection
        .list;

    let selected_company = &MODEL
        .page_root
        .page_after_auth
        .page_company_branch_selection
        .selected_company;

    rsx! {
        div {
            match MODEL.navigator.read() {
                ui_model::Navigator::CompanyBranchSelection(n) => {
                    match n {
                        ui_model::CompanyBranchSelection::None => rsx! {},
                        ui_model::CompanyBranchSelection::CreateCompany => rsx! {
                            CreateCompany {}
                        },
                        ui_model::CompanyBranchSelection::CreateCompanyBranch => {
                            rsx! {
                                CreateCompanyBranch {}
                            }
                        }
                    }
                }
                _ => rsx! {},
            }

            button {
                onclick: move |_| {
                    send(
                        ui_model::Message::CompanyAndBranchSelection(
                            ui_model::CompanyAndBranchSelection::ShowCreateCompany,
                        ),
                    )
                },
                "Add New Company"
            }

            div {
                for company in local_state.read() {
                    {
                        rsx! {
                            button {
                                onclick: move |_| {
                                    send(
                                        ui_model::Message::CompanyAndBranchSelection(
                                            ui_model::CompanyAndBranchSelection::SelectedCompany(
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
                                        send(
                                            ui_model::Message::CompanyAndBranchSelection(
                                                ui_model::CompanyAndBranchSelection::ShowCreateCompanyBranch,
                                            ),
                                        )
                                    },
                                    "Add New Branch"
                                }
                                div {
                                    for branch in company.branches {
                                        {
                                            rsx! {
                                                button {
                                                    onclick: {
                                                        move |_| {
                                                            send(
                                                                ui_model::Message::CompanyAndBranchSelection(
                                                                    ui_model::CompanyAndBranchSelection::SelectedCompanyBranch(
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
    let local_state = &MODEL
        .page_root
        .page_after_auth
        .page_company_branch_selection
        .page_create_company;

    rsx! {
        div {
            input {
                placeholder: "Company Name",
                oninput: move |event| {
                    send(
                        ui_model::Message::CreateCompany(
                            ui_model::CreateCompany::Name(event.value()),
                        ),
                    );
                },
                value: local_state.company_name.read(),
            }
            select {
                value: local_state.currency.read().as_str(),
                onchange: move |event| {
                    send(
                        ui_model::Message::CreateCompany(
                            ui_model::CreateCompany::Currency(event.value()),
                        ),
                    );
                },
                option { value: "USD", "USD" }
                option { value: "IQD", "IQD" }
            }
            button {
                onclick: move |_| {
                    send(ui_model::Message::CreateCompany(ui_model::CreateCompany::Submit));
                },
                "Create"
            }
            button {
                onclick: move |_| {
                    send(ui_model::Message::CreateCompany(ui_model::CreateCompany::Close));
                },
                "X"
            }
        }
    }
}

#[component]
fn CreateCompanyBranch() -> Element {
    let local_state = &MODEL
        .page_root
        .page_after_auth
        .page_company_branch_selection
        .page_create_company_branch;

    let consent_callback = move |consent: ui_model::UserConsent| {
        send(ui_model::Message::CreateCompanyBranch(
            ui_model::CreateCompanyBranch::Consent(consent),
        ));
    };

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
                    send(
                        ui_model::Message::CreateCompanyBranch(
                            ui_model::CreateCompanyBranch::Name(event.value()),
                        ),
                    );
                },
                value: local_state.branch_name.read(),
            }
            select {
                value: local_state.currency.read().as_str(),
                onchange: move |event| {
                    send(
                        ui_model::Message::CreateCompanyBranch(
                            ui_model::CreateCompanyBranch::Currency(event.value()),
                        ),
                    );
                },
                option { value: "USD", "USD" }
                option { value: "IQD", "IQD" }
            }
            button {
                onclick: move |_| {
                    send(
                        ui_model::Message::CreateCompanyBranch(ui_model::CreateCompanyBranch::Submit),
                    );
                },
                "Create"
            }
            button {
                onclick: move |_| {
                    send(
                        ui_model::Message::CreateCompanyBranch(ui_model::CreateCompanyBranch::Close),
                    );
                },
                "X"
            }
        }
    }
}
