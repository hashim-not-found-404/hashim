use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use utility::cache::ProcessId;
use utility::ui_effect::MessageTrait;
use utility::ui_effect::UiContext;
use utility_ui::domain::HashimSignal;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct Form {
    pub(crate) process_id: ProcessId,
    pub(crate) form: FormVariants,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) enum FormVariants {
    CreateBranch(use_case_create_branch::ui::TypeLocalModel),
    CreateCompany(use_case_create_company::ui::TypeLocalModel),
    CreateAccount(use_case_create_account::ui::TypeLocalModel),
    CreateAccountForBranch(use_case_create_account_for_branch::ui::TypeLocalModel),
    CreateJournalEntry(use_case_create_journal_entry::ui::TypeLocalModel),
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

pub(crate) trait LocalModel {
    fn forms(&self) -> impl HashimSignal<Vec<Form>>;
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
pub(crate) enum Change {
    Forms(Vec<Form>),
}

#[derive(Debug, Clone)]
pub(crate) enum Effect {}

impl MessageTrait for Message {}

pub fn update(
    msg: Message,
    local_model: &impl LocalModel,
    global_model: &impl GlobalModel,
) -> (Vec<Change>, Vec<Effect>) {
    match msg {
        Message::Intent(intent) => match intent {
            Intent::AddForm(forms_to_add) => {
                let mut forms = local_model.forms().read();

                let form = map_form_command_to_value(forms_to_add);

                forms.push(Form {
                    process_id: Default::default(),
                    form,
                });

                let change = vec![Change::Forms(forms)];

                (change, vec![])
            }
            Intent::CopyForm(v) => {
                let mut forms = local_model.forms().read();

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

                let change = vec![Change::Forms(forms)];

                (change, vec![])
            }
            Intent::DeleteForm(v) => {
                let mut forms = local_model.forms().read();
                forms.remove(v);

                let change = vec![Change::Forms(forms)];

                (change, vec![])
            }
        },
        Message::Observe(observe) => todo!(),
    }
}

pub fn apply(msg: Change, local_model: &impl LocalModel, _: &impl GlobalModel) {
    match msg {
        Change::Forms(i) => local_model.forms().set(i),
    }
}

pub async fn effect(msg: Effect, process_id: ProcessId, context: UiContext) -> Result<()> {
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

fn get_model(forms: Vec<Form>, process_id: ProcessId) -> Option<FormVariants> {
    for i in forms {
        if i.process_id == process_id {
            return Some(i.form);
        }
    }

    None
}
