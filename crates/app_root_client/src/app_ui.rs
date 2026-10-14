use crate::use_case_home;
use crate::use_case_navigator::client::Intent as NavIntent;
use crate::use_case_navigator::client::Message;
use crate::use_case_navigator::client::Navigator;
use crate::utils::async_send;
use crate::utils::init_commander_and_model;
use crate::utils::send;
use crate::utils::with_model;
use anyhow::Result;
use cache::utils::MyUuidConverter;
use dioxus::prelude::*;
use use_case_home::client::FormLocalModel;
use use_case_home::client::FormsToAdd;
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
            sender: move |i| send(
                replier.clone(),
                use_case_error_handler::client::Message::Intent(i)
            ),
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
            use_case_select_default_company::client::list_of_companies(page),
            use_case_select_default_company::client::list_of_branches(page, m),
            m.selected_company_uuid.clone(),
            use_case_select_default_company::client::selected_company_name(page, m),
            m.selected_company_branch_uuid.clone(),
            use_case_select_default_company::client::selected_company_branch_name(page, m),
            m.page_home.forms.clone(),
        )
    });

    let replier: Replier = new_replier(async |msg| -> Result<()> { async_send(msg).await });
    let replier1 = replier.clone();
    let replier2 = replier.clone();
    let replier3 = replier.clone();
    let replier4 = replier.clone();
    let replier5 = replier.clone();

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
                    use_case_home::client::Intent::AddForm(FormsToAdd::CreateAccount),
                )),
                "+ Account"
            }
            button {
                onclick: move |_| send(
                    replier2.clone(),
                    use_case_home::client::Message::Intent(
                    use_case_home::client::Intent::AddForm(FormsToAdd::CreateBranch),
                )),
                "+ Branch"
            }
            button {
                onclick: move |_| send(
                    replier3.clone(),
                    use_case_home::client::Message::Intent(
                    use_case_home::client::Intent::AddForm(FormsToAdd::CreateCompany),
                )),
                "+ Company"
            }
            button {
                onclick: move |_| send(
                    replier4.clone(),
                    use_case_home::client::Message::Intent(
                    use_case_home::client::Intent::AddForm(FormsToAdd::CreateAccountForBranch),
                )),
                "+ Account for Branch"
            }
            button {
                onclick: move |_| send(
                    replier5.clone(),
                    use_case_home::client::Message::Intent(
                    use_case_home::client::Intent::AddForm(FormsToAdd::CreateJournalEntry),
                )),
                "+ Journal Entry"
            }
        }

        for (index, form) in forms.iter().enumerate() {
            FormInstance {
                key: "{index}",
                index,
                form: form.clone(),
            }
        }
    }
}

#[component]
fn FormInstance(index: usize, form: FormLocalModel) -> Element {
    let replier: Replier = new_replier(async |msg| -> Result<()> { async_send(msg).await });
    let replier1 = replier.clone();

    rsx! {
        div {
            button {
                onclick: move |_| send(
                    replier.clone(),
                    use_case_home::client::Message::Intent(
                    use_case_home::client::Intent::CopyForm(index),
                )),
                "Copy"
            }
            button {
                onclick: move |_| send(
                    replier1.clone(),
                    use_case_home::client::Message::Intent(
                    use_case_home::client::Intent::DeleteForm(index),
                )),
                "X"
            }

            {
                match form {
                    FormLocalModel::CreateAccount(lm) => rsx! {
                        CreateAccountForm { index, lm }
                    },
                    FormLocalModel::CreateBranch(lm) => rsx! {
                        CreateBranchForm { index, lm }
                    },
                    FormLocalModel::CreateCompany(lm) => rsx! {
                        CreateCompanyForm { index, lm }
                    },
                    FormLocalModel::CreateAccountForBranch(lm) => rsx! {
                        CreateAccountForBranchForm { index, lm }
                    },
                    FormLocalModel::CreateJournalEntry(lm) => rsx! {
                        CreateJournalEntryForm { index, lm }
                    },
                }
            }
        }
    }
}

#[component]
fn CreateAccountForm(index: usize, lm: use_case_create_account::client::LocalModel) -> Element {
    let (
        account_name_error,
        company_name_error,
        resolved_company_uuid,
        list_of_company_name_and_uuid,
    ) = with_model(|m| {
        (
            use_case_create_account::client::error_account_name(&lm).map(|e| format!("{e:?}")),
            use_case_create_account::client::error_company_name(&lm, m).map(|e| format!("{e:?}")),
            use_case_create_account::client::resolved_company_uuid(&lm, m)
                .map(|uuid| uuid.to_string()),
            use_case_create_account::client::list_of_companies_to_display(&lm, m),
        )
    });

    let replier: Replier = new_replier(move |msg| async move {
        async_send(Box::new(use_case_home::client::Message::FormRelated {
            form_index: index,
            message: msg,
        }))
        .await
    });

    rsx! {
        use_case_create_account::ui::Component {
            sender: move |i| send(
                replier.clone(),
                use_case_home::client::Message::FormRelated {
                    form_index: index,
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
fn CreateBranchForm(index: usize, lm: use_case_create_branch::client::LocalModel) -> Element {
    let (company_name_error, resolved_company_uuid, list_of_company_name_and_uuid) =
        with_model(|m| {
            (
                use_case_create_branch::client::company_name_error(&lm, m)
                    .map(|e| format!("{e:?}")),
                use_case_create_branch::client::resolved_company_uuid(&lm, m)
                    .map(|uuid| uuid.to_string()),
                use_case_create_branch::client::list_of_companies_to_display(&lm, m),
            )
        });

    let replier: Replier = new_replier(move |msg| async move {
        async_send(Box::new(use_case_home::client::Message::FormRelated {
            form_index: index,
            message: msg,
        }))
        .await
    });

    rsx! {
        use_case_create_branch::ui::Component {
            sender: move |i| send(
                replier.clone(),
                use_case_home::client::Message::FormRelated {
                    form_index: index,
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
            branch_name_error: use_case_create_branch::client::error_branch_name(&lm)
                .map(|e| format!("{e:?}")),
            location_error: use_case_create_branch::client::error_location(&lm)
                .map(|e| format!("{e:?}")),
            list_of_company_name_and_uuid,
        }
    }
}

#[component]
fn CreateCompanyForm(index: usize, lm: use_case_create_company::client::LocalModel) -> Element {
    let replier: Replier = new_replier(move |msg| async move {
        async_send(Box::new(use_case_home::client::Message::FormRelated {
            form_index: index,
            message: msg,
        }))
        .await
    });

    rsx! {
        use_case_create_company::ui::Component {
            sender: move |i| send(
                replier.clone(),
                use_case_home::client::Message::FormRelated {
                    form_index: index,
                    message: Box::new(use_case_create_company::client::Message::Intent(i)),
                },
            ),
            show_dialog: lm.show_dialog.clone(),
            is_loading: lm.async_state.clone().is_loading(),
            company_name: lm.company_name.clone(),
            currency: lm.currency.clone(),
            company_name_error: use_case_create_company::client::error_company_name(&lm)
                .map(|e| format!("{e:?}")),
        }
    }
}

#[component]
fn CreateAccountForBranchForm(
    index: usize,
    lm: use_case_create_account_for_branch::client::LocalModel,
) -> Element {
    let list_of_account_name_and_uuid = with_model(|m| {
        use_case_create_account_for_branch::client::list_of_accounts_to_display(&lm, m)
    });

    let replier: Replier = new_replier(move |msg| async move {
        async_send(Box::new(use_case_home::client::Message::FormRelated {
            form_index: index,
            message: msg,
        }))
        .await
    });

    rsx! {
        use_case_create_account_for_branch::ui::Component {
            sender: move |i| send(
                replier.clone(),
                use_case_home::client::Message::FormRelated {
                    form_index: index,
                    message: Box::new(use_case_create_account_for_branch::client::Message::Intent(i)),
                },
            ),
            show_dialog: lm.show_dialog.clone(),
            is_loading: lm.async_state.clone().is_loading(),
            account_name: lm.account_name.clone(),
            outflow_type: lm.outflow_type.clone(),
            inflow_type: lm.inflow_type.clone(),
            list_of_account_name_and_uuid,
        }
    }
}

#[component]
fn CreateJournalEntryForm(
    index: usize,
    lm: use_case_create_journal_entry::client::LocalModel,
) -> Element {
    let replier: Replier = new_replier(move |msg| async move {
        async_send(Box::new(use_case_home::client::Message::FormRelated {
            form_index: index,
            message: msg,
        }))
        .await
    });

    rsx! {
        use_case_create_journal_entry::ui::Component {
            sender: move |i| send(
                replier.clone(),
                use_case_home::client::Message::FormRelated {
                    form_index: index,
                    message: Box::new(use_case_create_journal_entry::client::Message::Intent(i)),
                },
            ),
            show_dialog: lm.show_dialog.clone(),
            is_loading: lm.is_loading.clone(),
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
                use_case_sign_in::client::error_user_id(page).map(|e| format!("{e:?}")),
                use_case_sign_in::client::error_password(page).map(|e| format!("{e:?}")),
                page.async_state.clone().is_loading(),
            )
        });

    let replier: Replier = new_replier(async |msg| -> Result<()> { async_send(msg).await });
    let replier1 = replier.clone();

    rsx! {
        use_case_sign_in::ui::Component {
            sender: move |i| send(replier.clone(),use_case_sign_in::client::Message::Intent(i)),
            on_go_to_sign_up: move || send(replier1.clone(),Message::Intent(NavIntent::GoToSignUp)),
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
                use_case_sign_up::client::error_user_id(page).map(|e| format!("{e:?}")),
                use_case_sign_up::client::error_user_name(page).map(|e| format!("{e:?}")),
                page.async_state.clone().is_loading(),
            )
        });

    let replier: Replier = new_replier(async |msg| -> Result<()> { async_send(msg).await });
    let replier1 = replier.clone();

    rsx! {
        use_case_sign_up::ui::Component {
            sender: move |i| send(replier.clone(),use_case_sign_up::client::Message::Intent(i)),
            on_back: move || send(replier1.clone(),Message::Intent(NavIntent::GoToSignIn)),
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
