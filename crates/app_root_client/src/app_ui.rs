use crate::use_case_navigator::client::Intent as NavIntent;
use crate::use_case_navigator::client::Message;
use crate::use_case_navigator::client::Navigator;
use crate::utils::async_send;
use crate::utils::init_commander_and_model;
use crate::utils::send;
use crate::utils::with_model;
use crate::wire::create_account_for_branch_form::WrapperForm as AccountForBranchForm;
use crate::wire::create_account_form::WrapperForm as AccountForm;
use crate::wire::create_branch_form::WrapperForm as BranchForm;
use crate::wire::create_company_form::WrapperForm as CompanyForm;
use anyhow::Result;
use cache::utils::MyUuidConverter;
use dioxus::prelude::*;
use use_case_home::client::FormEntry;
use use_case_home::client::FormId;
use utility::ui_effect::Replier;
use utility::ui_effect::new_replier;

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

    let replier: Replier = new_replier(async |msg| -> Result<()> { async_send(msg).await });

    rsx! {
        Router::<Route> {}
        use_case_error_handler::ui::Component {
            sender: move |i| send(replier.clone(), use_case_error_handler::client::Message::Intent(i)),
            is_expand_all,
            errors,
        }
    }
}

#[component]
fn Home() -> Element {
    let (
        user_name,
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
            page.list_of_companies(),
            page.list_of_branches(m),
            m.selected_company_uuid.clone(),
            page.selected_company_name(m),
            m.selected_company_branch_uuid.clone(),
            page.selected_company_branch_name(m),
            m.page_home.forms.clone(),
        )
    });

    let replier: Replier = new_replier(async |msg| -> Result<()> { async_send(msg).await });
    let replier1 = replier.clone();
    let replier2 = replier.clone();
    let replier3 = replier.clone();
    let replier4 = replier.clone();

    rsx! {
        use_case_select_default_company::ui::Component {
            sender: move |i| send(
                replier.clone(),
                use_case_select_default_company::client::Message::Intent(i),
            ),
            user_name,
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
                    replier1.clone(),
                    use_case_home::client::Message::Intent(
                        use_case_home::client::Intent::AddForm(Box::new(AccountForm(
                            Default::default(),
                        ))),
                    ),
                ),
                "+ Account"
            }
            button {
                onclick: move |_| send(
                    replier2.clone(),
                    use_case_home::client::Message::Intent(
                        use_case_home::client::Intent::AddForm(Box::new(BranchForm(
                            Default::default(),
                        ))),
                    ),
                ),
                "+ Branch"
            }
            button {
                onclick: move |_| send(
                    replier3.clone(),
                    use_case_home::client::Message::Intent(
                        use_case_home::client::Intent::AddForm(Box::new(CompanyForm(
                            Default::default(),
                        ))),
                    ),
                ),
                "+ Company"
            }
            button {
                onclick: move |_| send(
                    replier4.clone(),
                    use_case_home::client::Message::Intent(
                        use_case_home::client::Intent::AddForm(Box::new(
                            AccountForBranchForm(Default::default()),
                        )),
                    ),
                ),
                "+ Account for Branch"
            }
        }

        for (index, entry) in forms.iter().enumerate() {
            FormInstance { key: "{entry.id:?}", index, entry: entry.clone() }
        }
    }
}

#[component]
fn FormInstance(index: usize, entry: FormEntry) -> Element {
    let form_id: FormId = entry.id;
    let form = entry.form;

    let replier: Replier = new_replier(async |msg| -> Result<()> { async_send(msg).await });
    let replier1 = replier.clone();

    let is_loading = form.is_loading();
    let rendered = use_case_home::ui::render_to_element(form.as_ref(), form_id);

    rsx! {
        div {
            button {
                onclick: move |_| send(
                    replier.clone(),
                    use_case_home::client::Message::Intent(
                        use_case_home::client::Intent::CopyForm(index),
                    ),
                ),
                "Copy"
            }
            button {
                disabled: is_loading,
                onclick: move |_| send(
                    replier1.clone(),
                    use_case_home::client::Message::Intent(
                        use_case_home::client::Intent::DeleteForm(index),
                    ),
                ),
                "X"
            }

            {rendered}
        }
    }
}

#[component]
pub(crate) fn CreateAccountForm(
    form_id: FormId,
    lm: use_case_create_account::client::LocalModel,
) -> Element {
    let (
        account_name_error,
        company_name_error,
        resolved_company_uuid,
        list_of_company_name_and_uuid,
    ) = with_model(|m| {
        (
            lm.error_account_name().map(|e| format!("{e:?}")),
            lm.error_company_name(m).map(|e| format!("{e:?}")),
            lm.resolved_company_uuid(m).map(|uuid| uuid.to_string()),
            lm.list_of_companies_to_display(m),
        )
    });

    let replier: Replier = new_replier(move |msg| async move {
        async_send(Box::new(use_case_home::client::Message::FormRelated {
            form_id,
            message: msg,
        }))
        .await
    });

    rsx! {
        use_case_create_account::ui::Component {
            sender: move |i| send(
                replier.clone(),
                use_case_home::client::Message::FormRelated {
                    form_id,
                    message: Box::new(use_case_create_account::client::Message::Intent(i)),
                },
            ),
            show_dialog: lm.show_dialog.clone(),
            is_loading: lm.async_state.clone().is_loading(),
            is_debit: lm.is_debit.clone(),
            is_permanent_account: lm.is_permanent_account.clone(),
            account_name: lm.account_name.clone(),
            unit_of_measurement_of_quantity: lm.unit_of_measurement_of_quantity.clone(),
            account_name_error,
            company_name_error,
            selected_company_name: lm.selected_company_name.clone(),
            resolved_company_uuid,
            list_of_company_name_and_uuid,
        }
    }
}

#[component]
pub(crate) fn CreateBranchForm(
    form_id: FormId,
    lm: use_case_create_branch::client::LocalModel,
) -> Element {
    let (company_name_error, resolved_company_uuid, list_of_company_name_and_uuid) =
        with_model(|m| {
            (
                lm.company_name_error(m).map(|e| format!("{e:?}")),
                lm.resolved_company_uuid(m).map(|uuid| uuid.to_string()),
                lm.list_of_companies_to_display(m),
            )
        });

    let replier: Replier = new_replier(move |msg| async move {
        async_send(Box::new(use_case_home::client::Message::FormRelated {
            form_id,
            message: msg,
        }))
        .await
    });

    rsx! {
        use_case_create_branch::ui::Component {
            sender: move |i| send(
                replier.clone(),
                use_case_home::client::Message::FormRelated {
                    form_id,
                    message: Box::new(use_case_create_branch::client::Message::Intent(i)),
                },
            ),
            show_dialog: lm.show_dialog.clone(),
            is_loading: lm.async_state.clone().is_loading(),
            company_name: lm.company_name.clone(),
            company_name_error,
            resolved_company_uuid,
            branch_name: lm.branch_name.clone(),
            currency: lm.currency.clone(),
            location: lm.location.clone(),
            branch_name_error: lm.error_branch_name().map(|e| format!("{e:?}")),
            location_error: lm.error_location().map(|e| format!("{e:?}")),
            list_of_company_name_and_uuid,
        }
    }
}

#[component]
pub(crate) fn CreateCompanyForm(
    form_id: FormId,
    lm: use_case_create_company::client::LocalModel,
) -> Element {
    let replier: Replier = new_replier(move |msg| async move {
        async_send(Box::new(use_case_home::client::Message::FormRelated {
            form_id,
            message: msg,
        }))
        .await
    });

    rsx! {
        use_case_create_company::ui::Component {
            sender: move |i| send(
                replier.clone(),
                use_case_home::client::Message::FormRelated {
                    form_id,
                    message: Box::new(use_case_create_company::client::Message::Intent(i)),
                },
            ),
            show_dialog: lm.show_dialog.clone(),
            is_loading: lm.async_state.clone().is_loading(),
            company_name: lm.company_name.clone(),
            currency: lm.currency.clone(),
            company_name_error: lm.error_company_name().map(|e| format!("{e:?}")),
        }
    }
}

#[component]
pub(crate) fn CreateAccountForBranchForm(
    form_id: FormId,
    lm: use_case_create_account_for_branch::client::LocalModel,
) -> Element {
    let list_of_account_name_and_uuid = with_model(|m| lm.list_of_accounts_to_display(m));

    let replier: Replier = new_replier(move |msg| async move {
        async_send(Box::new(use_case_home::client::Message::FormRelated {
            form_id,
            message: msg,
        }))
        .await
    });

    rsx! {
        use_case_create_account_for_branch::ui::Component {
            sender: move |i| send(
                replier.clone(),
                use_case_home::client::Message::FormRelated {
                    form_id,
                    message: Box::new(
                        use_case_create_account_for_branch::client::Message::Intent(i),
                    ),
                },
            ),
            show_dialog: lm.show_dialog.clone(),
            is_loading: lm.async_state.clone().is_loading(),
            account_name: lm.account_name.clone(),
            outflow_type: lm.outflow_type,
            inflow_type: lm.inflow_type,
            list_of_account_name_and_uuid,
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
                m.feature_state_auth.user_password.clone(),
                page.error_user_id().map(|e| format!("{e:?}")),
                page.error_password().map(|e| format!("{e:?}")),
                page.async_state.clone().is_loading(),
            )
        });

    let replier: Replier = new_replier(async |msg| -> Result<()> { async_send(msg).await });
    let replier1 = replier.clone();

    rsx! {
        use_case_sign_in::ui::Component {
            sender: move |i| send(replier.clone(), use_case_sign_in::client::Message::Intent(i)),
            on_go_to_sign_up: move || send(replier1.clone(), Message::Intent(NavIntent::GoToSignUp)),
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
                m.feature_state_auth.user_password.clone(),
                page.error_user_id().map(|e| format!("{e:?}")),
                page.error_user_name().map(|e| format!("{e:?}")),
                page.async_state.clone().is_loading(),
            )
        });

    let replier: Replier = new_replier(async |msg| -> Result<()> { async_send(msg).await });
    let replier1 = replier.clone();

    rsx! {
        use_case_sign_up::ui::Component {
            sender: move |i| send(replier.clone(), use_case_sign_up::client::Message::Intent(i)),
            on_back: move || send(replier1.clone(), Message::Intent(NavIntent::GoToSignIn)),
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
