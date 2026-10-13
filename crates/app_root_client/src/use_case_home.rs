pub(crate) mod client {
    use anyhow::Result;
    use serde::Deserialize;
    use serde::Serialize;
    use utility::cache::ProcessId;
    use utility::ui_effect::MessageTrait;
    use utility::ui_effect::Replier;
    use utility::ui_effect::UiContext;

    #[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
    pub(crate) struct Form {
        pub(crate) process_id: ProcessId,
        pub(crate) form: FormVariants,
    }

    #[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
    pub(crate) enum FormVariants {
        CreateBranch(use_case_create_branch::client::LocalModel),
        CreateCompany(use_case_create_company::client::LocalModel),
        CreateAccount(use_case_create_account::client::LocalModel),
        CreateAccountForBranch(use_case_create_account_for_branch::client::LocalModel),
        CreateJournalEntry(use_case_create_journal_entry::client::LocalModel),
    }

    #[derive(Debug, Clone)]
    pub(crate) enum FormsToAdd {
        CreateBranch,
        CreateCompany,
        CreateAccount,
        CreateAccountForBranch,
        CreateJournalEntry,
    }

    pub(crate) trait GlobalModel {}

    #[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
    pub(crate) struct LocalModel {
        pub(crate) forms: Vec<Form>,
    }

    #[derive(Debug, Clone)]
    pub(crate) enum Message {
        Intent(Intent),
        Observe(Observe),
    }

    #[derive(Debug, Clone)]
    pub(crate) enum Intent {
        AddForm(FormsToAdd),
        CopyForm(usize),
        DeleteForm(usize),
    }

    #[derive(Debug, Clone)]
    pub(crate) enum Observe {}

    #[derive(Debug, Clone)]
    pub(crate) enum LocalChange {
        Forms(Vec<Form>),
    }

    #[derive(Debug, Clone)]
    pub(crate) enum Effect {}

    impl MessageTrait for Message {}

    pub fn update(
        msg: Message,
        local_model: &LocalModel,
        global_model: &impl GlobalModel,
    ) -> (Vec<LocalChange>, Vec<Effect>) {
        match msg {
            Message::Intent(intent) => match intent {
                Intent::AddForm(forms_to_add) => {
                    let mut forms = local_model.forms.clone();

                    let form = map_form_command_to_value(forms_to_add);

                    forms.push(Form {
                        process_id: Default::default(),
                        form,
                    });

                    let change = vec![LocalChange::Forms(forms)];

                    (change, vec![])
                }
                Intent::CopyForm(v) => {
                    let mut forms = local_model.forms.clone();

                    let Some(form) = forms.get(v) else {
                        return (vec![], vec![]);
                    };

                    forms.insert(
                        v,
                        Form {
                            process_id: Default::default(),
                            form: form.form.clone(),
                        },
                    );

                    let change = vec![LocalChange::Forms(forms)];

                    (change, vec![])
                }
                Intent::DeleteForm(v) => {
                    let mut forms = local_model.forms.clone();
                    forms.remove(v);

                    let change = vec![LocalChange::Forms(forms)];

                    (change, vec![])
                }
            },
            Message::Observe(observe) => todo!(),
        }
    }

    pub fn local_apply(msg: LocalChange, local_model: &mut LocalModel) {
        match msg {
            LocalChange::Forms(i) => local_model.forms = i,
        }
    }

    pub async fn effect(msg: Effect, replier: Replier, context: UiContext) -> Result<()> {
        Ok(())
    }

    fn map_form_command_to_value(forms_to_add: FormsToAdd) -> FormVariants {
        match forms_to_add {
            FormsToAdd::CreateBranch => FormVariants::CreateBranch(Default::default()),
            FormsToAdd::CreateCompany => FormVariants::CreateCompany(Default::default()),
            FormsToAdd::CreateAccount => FormVariants::CreateAccount(Default::default()),
            FormsToAdd::CreateAccountForBranch => {
                FormVariants::CreateAccountForBranch(Default::default())
            }
            FormsToAdd::CreateJournalEntry => FormVariants::CreateJournalEntry(Default::default()),
        }
    }
}

pub(crate) mod ui {}
