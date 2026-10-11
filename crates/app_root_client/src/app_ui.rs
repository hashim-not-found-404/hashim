use crate::home::FormVariants;
use crate::home::FormsToAdd;
use crate::navigator::Intent as NavIntent;
use crate::navigator::Message;
use crate::navigator::Navigator;
use crate::utils::MODEL;
use crate::utils::init_commander_and_model;
use crate::utils::send;
use cache::utils::MyUuidConverter;
use dioxus::prelude::*;
use use_case_create_account::client::LocalModel as _;
use use_case_create_account_for_branch::client::LocalModel as _;
use use_case_create_branch::client::LocalModel as _;
use use_case_create_company::client::LocalModel as _;
use use_case_create_journal_entry::client::LocalModel as _;
use use_case_error_handler::client::LocalModel as _;
use use_case_sign_in::client::LocalModel as _;
use use_case_sign_up::client::LocalModel as _;
use utility::cache::ProcessId;
use utility_ui::domain::HashimSignal;

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
    match MODEL.navigator.read() {
        Navigator::SignIn => {
            navigator().push(Route::SignIn {});
        }
        Navigator::SignUp => {
            navigator().push(Route::SignUp {});
        }
    }

    if MODEL.user_uuid().is_some() {
        navigator().push(Route::Home {});
    }

    rsx! {
        Outlet::<Route> {}
    }
}

#[component]
pub(crate) fn App() -> Element {
    init_commander_and_model();

    let process_id = use_hook(ProcessId::default);

    rsx! {
        Router::<Route> {}
        use_case_error_handler::ui::Component {
            sender: move |i| send(process_id, use_case_error_handler::client::Message::Intent(i)),
            is_expand_all: MODEL.page_error_handler.is_expand_all().read(),
            errors: MODEL.page_error_handler.errors().read(),
        }
    }
}

#[component]
fn Home() -> Element {
    let select_company_process_id = use_hook(ProcessId::default);
    let home_process_id = use_hook(ProcessId::default);

    let forms = &MODEL.page_home.forms;

    rsx! {
        use_case_select_default_company::ui::Component {
            sender: move |i| send(
                select_company_process_id,
                use_case_select_default_company::client::Message::Intent(i),
            ),
            user_name: MODEL
                .user_name
                .read()
                .or_else(|| MODEL.user_id.read().into())
                .unwrap_or_default(),
            companies: use_case_select_default_company::client::list_of_companies(
                &MODEL.page_select_default_company,
            ),
            branches: use_case_select_default_company::client::list_of_branches(
                &MODEL.page_select_default_company,
                MODEL.as_ref(),
            ),
            selected_company: MODEL.selected_company_uuid.read(),
            selected_company_name: use_case_select_default_company::client::selected_company_name(
                &MODEL.page_select_default_company,
                MODEL.as_ref(),
            ),
            selected_branch: MODEL.selected_company_branch_uuid.read(),
            selected_branch_name: use_case_select_default_company::client::selected_company_branch_name(
                &MODEL.page_select_default_company,
                MODEL.as_ref(),
            ),
        }

        div {
            button {
                onclick: move |_| send(home_process_id, crate::home::Message::Intent(
                    crate::home::Intent::AddForm(FormsToAdd::CreateAccount),
                )),
                "+ Account"
            }
            button {
                onclick: move |_| send(home_process_id, crate::home::Message::Intent(
                    crate::home::Intent::AddForm(FormsToAdd::CreateBranch),
                )),
                "+ Branch"
            }
            button {
                onclick: move |_| send(home_process_id, crate::home::Message::Intent(
                    crate::home::Intent::AddForm(FormsToAdd::CreateCompany),
                )),
                "+ Company"
            }
            button {
                onclick: move |_| send(home_process_id, crate::home::Message::Intent(
                    crate::home::Intent::AddForm(FormsToAdd::CreateAccountForBranch),
                )),
                "+ Account for Branch"
            }
            button {
                onclick: move |_| send(home_process_id, crate::home::Message::Intent(
                    crate::home::Intent::AddForm(FormsToAdd::CreateJournalEntry),
                )),
                "+ Journal Entry"
            }
        }

        for (index, form) in forms.read().iter().enumerate() {
            FormInstance {
                key: "{index}",
                index,
                process_id: form.process_id,
                form: form.form.clone(),
                home_process_id,
            }
        }
    }
}

#[component]
fn FormInstance(
    index: usize,
    process_id: ProcessId,
    form: FormVariants,
    home_process_id: ProcessId,
) -> Element {
    rsx! {
        div {
            button {
                onclick: move |_| send(home_process_id, crate::home::Message::Intent(
                    crate::home::Intent::CopyForm(index),
                )),
                "Copy"
            }
            button {
                onclick: move |_| send(home_process_id, crate::home::Message::Intent(
                    crate::home::Intent::DeleteForm(index),
                )),
                "X"
            }

            {
                match form {
                    FormVariants::CreateAccount(lm) => rsx! {
                        CreateAccountForm { process_id, lm }
                    },
                    FormVariants::CreateBranch(lm) => rsx! {
                        CreateBranchForm { process_id, lm }
                    },
                    FormVariants::CreateCompany(lm) => rsx! {
                        CreateCompanyForm { process_id, lm }
                    },
                    FormVariants::CreateAccountForBranch(lm) => rsx! {
                        CreateAccountForBranchForm { process_id, lm }
                    },
                    FormVariants::CreateJournalEntry(lm) => rsx! {
                        CreateJournalEntryForm { process_id, lm }
                    },
                }
            }
        }
    }
}

#[component]
fn CreateAccountForm(
    process_id: ProcessId,
    lm: use_case_create_account::ui::TypeLocalModel,
) -> Element {
    rsx! {
        use_case_create_account::ui::Component {
            sender: move |i| send(process_id, use_case_create_account::client::Message::Intent(i)),
            show_dialog: lm.show_dialog().read(),
            is_loading: lm.async_state().read().is_loading(),
            is_debit: lm.is_debit().read(),
            is_permanent_account: lm.is_permanent_account().read(),
            account_name: lm.account_name().read(),
            unit_of_measurement_of_quantity: lm.unit_of_measurement_of_quantity().read(),
            account_name_error: use_case_create_account::client::error_account_name(&lm)
                .map(|e| format!("{e:?}")),
            company_name_error: use_case_create_account::client::error_company_name(
                &lm,
                MODEL.as_ref(),
            )
            .map(|e| format!("{e:?}")),
            selected_company_name: lm.selected_company_name().read(),
            resolved_company_uuid: use_case_create_account::client::resolved_company_uuid(
                &lm,
                MODEL.as_ref(),
            )
            .map(|uuid| uuid.to_string()),
            list_of_company_name_and_uuid: use_case_create_account::client::list_of_companies_to_display(
                &lm,
                MODEL.as_ref(),
            ),
        }
    }
}

#[component]
fn CreateBranchForm(
    process_id: ProcessId,
    lm: use_case_create_branch::ui::TypeLocalModel,
) -> Element {
    rsx! {
        use_case_create_branch::ui::Component {
            sender: move |i| send(process_id, use_case_create_branch::client::Message::Intent(i)),
            show_dialog: lm.show_dialog().read(),
            is_loading: lm.async_state().read().is_loading(),
            company_name: lm.company_name().read(),
            company_name_error: use_case_create_branch::client::company_name_error(
                &lm,
                MODEL.as_ref(),
            )
            .map(|e| format!("{e:?}")),
            resolved_company_uuid: use_case_create_branch::client::resolved_company_uuid(
                &lm,
                MODEL.as_ref(),
            )
            .map(|uuid| uuid.to_string()),
            branch_name: lm.branch_name().read(),
            currency: lm.currency().read(),
            location: lm.location().read(),
            branch_name_error: use_case_create_branch::client::error_branch_name(&lm)
                .map(|e| format!("{e:?}")),
            location_error: use_case_create_branch::client::error_location(&lm)
                .map(|e| format!("{e:?}")),
            list_of_company_name_and_uuid: use_case_create_branch::client::list_of_companies_to_display(
                &lm,
                MODEL.as_ref(),
            ),
        }
    }
}

#[component]
fn CreateCompanyForm(
    process_id: ProcessId,
    lm: use_case_create_company::ui::TypeLocalModel,
) -> Element {
    rsx! {
        use_case_create_company::ui::Component {
            sender: move |i| send(process_id, use_case_create_company::client::Message::Intent(i)),
            show_dialog: lm.show_dialog().read(),
            is_loading: lm.async_state().read().is_loading(),
            company_name: lm.company_name().read(),
            currency: lm.currency().read(),
            company_name_error: use_case_create_company::client::error_company_name(&lm)
                .map(|e| format!("{e:?}")),
        }
    }
}

#[component]
fn CreateAccountForBranchForm(
    process_id: ProcessId,
    lm: use_case_create_account_for_branch::ui::TypeLocalModel,
) -> Element {
    rsx! {
        use_case_create_account_for_branch::ui::Component {
            sender: move |i| send(
                process_id,
                use_case_create_account_for_branch::client::Message::Intent(i),
            ),
            show_dialog: lm.show_dialog().read(),
            is_loading: lm.async_state().read().is_loading(),
            account_name: lm.account_name().read(),
            outflow_type: lm.outflow_type().read(),
            inflow_type: lm.inflow_type().read(),
            list_of_account_name_and_uuid: use_case_create_account_for_branch::client::list_of_accounts_to_display(
                &lm,
                MODEL.as_ref(),
            ),
        }
    }
}

#[component]
fn CreateJournalEntryForm(
    process_id: ProcessId,
    lm: use_case_create_journal_entry::ui::TypeLocalModel,
) -> Element {
    rsx! {
        use_case_create_journal_entry::ui::Component {
            sender: move |i| send(
                process_id,
                use_case_create_journal_entry::client::Message::Intent(i),
            ),
            show_dialog: lm.show_dialog().read(),
            is_loading: lm.is_loading().read(),
        }
    }
}

#[component]
fn SignIn() -> Element {
    let process_id = use_hook(ProcessId::default);
    let process_id1 = use_hook(ProcessId::default);

    rsx! {
        use_case_sign_in::ui::Component {
            sender: move |i| send(process_id, use_case_sign_in::client::Message::Intent(i)),
            on_go_to_sign_up: move || send(process_id1, Message::Intent(NavIntent::GoToSignUp)),
            show_dialog: MODEL.page_sign_in.show_dialog().read(),
            user_id: MODEL.user_id.read(),
            password: MODEL.feature_state_auth.user_password.read(),
            error_user_id: use_case_sign_in::client::error_user_id(&MODEL.page_sign_in)
                .map(|e| format!("{e:?}")),
            error_password: use_case_sign_in::client::error_password(&MODEL.page_sign_in)
                .map(|e| format!("{e:?}")),
            is_loading: MODEL.page_sign_in.async_state().read().is_loading(),
        }
    }
}

#[component]
fn SignUp() -> Element {
    let process_id = use_hook(ProcessId::default);
    let process_id1 = use_hook(ProcessId::default);

    rsx! {
        use_case_sign_up::ui::Component {
            sender: move |i| send(process_id, use_case_sign_up::client::Message::Intent(i)),
            on_back: move || send(process_id1, Message::Intent(NavIntent::GoToSignIn)),
            show_dialog: MODEL.page_sign_up.show_dialog().read(),
            user_id: MODEL.user_id.read(),
            user_name: MODEL.user_name.read(),
            password: MODEL.feature_state_auth.user_password.read(),
            error_user_id: use_case_sign_up::client::error_user_id(&MODEL.page_sign_up)
                .map(|e| format!("{e:?}")),
            error_user_name: use_case_sign_up::client::error_user_name(&MODEL.page_sign_up)
                .map(|e| format!("{e:?}")),
            is_loading: MODEL.page_sign_up.async_state().read().is_loading(),
        }
    }
}
