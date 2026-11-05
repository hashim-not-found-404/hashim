use crate::model::TypeGlobalModel;
use crate::use_case_navigator::client::Intent as NavIntent;
use crate::use_case_navigator::client::Message;
use crate::use_case_navigator::client::Navigator;
use crate::utils::COMMANDER;
use crate::utils::init_commander_and_model;
use crate::utils::send;
use crate::utils::with_model;
use crate::wire::AccountForBranchForm;
use crate::wire::AccountForm;
use crate::wire::BranchForm;
use crate::wire::CompanyForm;
use crate::wire::JournalForm;
use dioxus::prelude::*;
use use_case_home_wire_client::FormEntry;
use use_case_home_wire_client::FormId;
use use_case_home_wire_client::render_to_element;

#[derive(Debug, Clone, PartialEq, Routable)]
pub(crate) enum Route {
    #[layout(RootLayout)]
    #[route("/")]
    SignIn {},
    #[route("/sign_up")]
    SignUp {},
    #[route("/home")]
    Home {},
}

#[component]
fn RootLayout() -> Element {
    let desired_route = use_memo(move || {
        with_model(|m| {
            if m.user_uuid().is_some() {
                Route::Home {}
            } else {
                match m.navigator {
                    Navigator::SignIn => Route::SignIn {},
                    Navigator::SignUp => Route::SignUp {},
                }
            }
        })
    });

    use_effect(move || {
        navigator().push(desired_route());
    });

    rsx! {
        Outlet::<Route> {}
    }
}

#[component]
pub(crate) fn App() -> Element {
    init_commander_and_model();

    let (is_expand_all, errors) = with_model(|m| {
        (
            m.page_error_handler.is_expand_all.clone(),
            m.page_error_handler.errors.clone(),
        )
    });

    rsx! {
        Router::<Route> {}
        use_case_error_handler_wire_client::Component {
            sender: move |i| send(use_case_error_handler_wire_client::Message::Intent(i)),
            is_expand_all,
            errors,
        }
    }
}

#[component]
fn Home() -> Element {
    let (
        user_name,
        is_loading_companies,
        companies,
        branches,
        selected_company,
        selected_company_name,
        selected_branch,
        selected_branch_name,
        forms,
    ) = with_model(|m| {
        let page = &m.page_select_default_company;
        (
            m.user_name
                .clone()
                .or_else(|| Some(m.user_id.clone()))
                .unwrap_or_default(),
            page.async_state.clone().is_loading(),
            page.list_of_companies(),
            page.list_of_branches(m),
            m.selected_company_uuid.clone(),
            page.selected_company_name(m),
            m.selected_company_branch_uuid.clone(),
            page.selected_company_branch_name(m),
            m.page_home.forms.clone(),
        )
    });

    rsx! {
        use_case_select_default_company_wire_client::Component {
            sender: move |i| send(use_case_select_default_company_wire_client::Message::Intent(i)),
            user_name,
            is_loading: is_loading_companies,
            companies,
            branches,
            selected_company,
            selected_company_name,
            selected_branch,
            selected_branch_name,
        }

        div {
            button {
                onclick: move |_| send(
                    use_case_home_wire_client::Message::Intent(
                        use_case_home_wire_client::Intent::AddForm(
                            Box::new(AccountForm::default()),
                        ),
                    ),
                ),
                "+ Account"
            }
            button {
                onclick: move |_| send(
                    use_case_home_wire_client::Message::Intent(
                        use_case_home_wire_client::Intent::AddForm(
                            Box::new(BranchForm::default()),
                        ),
                    ),
                ),
                "+ Branch"
            }
            button {
                onclick: move |_| send(
                    use_case_home_wire_client::Message::Intent(
                        use_case_home_wire_client::Intent::AddForm(
                            Box::new(CompanyForm::default()),
                        ),
                    ),
                ),
                "+ Company"
            }
            button {
                onclick: move |_| send(
                    use_case_home_wire_client::Message::Intent(
                        use_case_home_wire_client::Intent::AddForm(
                            Box::new(AccountForBranchForm::default()),
                        ),
                    ),
                ),
                "+ Account for Branch"
            }
            button {
                onclick: move |_| send(
                    use_case_home_wire_client::Message::Intent(
                        use_case_home_wire_client::Intent::AddForm(
                            Box::new(JournalForm::default()),
                        ),
                    ),
                ),
                "+ Journal"
            }
        }

        for entry in forms.iter() {
            FormInstance { key: "{entry.id:?}", entry: entry.clone() }
        }
    }
}

#[component]
fn FormInstance(entry: FormEntry) -> Element {
    let form_id: FormId = entry.id;
    let form = entry.form;

    let is_loading = form.is_loading();

    let model_snapshot: TypeGlobalModel = with_model(|m| m.clone());
    let rendered = render_to_element(form.as_ref(), form_id, &model_snapshot, &COMMANDER);

    rsx! {
        div {
            button {
                onclick: move |_| send(
                    use_case_home_wire_client::Message::Intent(
                        use_case_home_wire_client::Intent::CopyForm(form_id),
                    ),
                ),
                "Copy"
            }
            button {
                disabled: is_loading,
                onclick: move |_| send(
                    use_case_home_wire_client::Message::Intent(
                        use_case_home_wire_client::Intent::DeleteForm(form_id),
                    ),
                ),
                "X"
            }

            {rendered}
        }
    }
}

#[component]
fn SignIn() -> Element {
    let (show_dialog, user_id, password, error_user_id, error_password, is_loading) =
        with_model(|m| {
            let page = &m.page_sign_in;
            (
                page.show_dialog.clone(),
                m.user_id.clone(),
                m.user_password.clone(),
                page.error_user_id().map(|e| format!("{e:?}")),
                page.error_password().map(|e| format!("{e:?}")),
                page.async_state.clone().is_loading(),
            )
        });

    rsx! {
        use_case_sign_in_wire_client::Component {
            sender: move |i| send(use_case_sign_in_wire_client::Message::Intent(i)),
            on_go_to_sign_up: move || send(Message::Intent(NavIntent::GoToSignUp)),
            show_dialog,
            user_id,
            password,
            error_user_id,
            error_password,
            is_loading,
        }
    }
}

#[component]
fn SignUp() -> Element {
    let (show_dialog, user_id, user_name, password, error_user_id, error_user_name, is_loading) =
        with_model(|m| {
            let page = &m.page_sign_up;
            (
                page.show_dialog.clone(),
                m.user_id.clone(),
                m.user_name.clone(),
                m.user_password.clone(),
                page.error_user_id().map(|e| format!("{e:?}")),
                page.error_user_name().map(|e| format!("{e:?}")),
                page.async_state.clone().is_loading(),
            )
        });

    rsx! {
        use_case_sign_up_wire_client::Component {
            sender: move |i| send(use_case_sign_up_wire_client::Message::Intent(i)),
            on_back: move || send(Message::Intent(NavIntent::GoToSignIn)),
            show_dialog,
            user_id,
            user_name,
            password,
            error_user_id,
            error_user_name,
            is_loading,
        }
    }
}
