pub(crate) mod client {
    use anyhow::Result;
    use dioxus::core::bail;
    use serde::Deserialize;
    use serde::Serialize;
    use std::any::Any;
    use utility::ui_effect::MessageTrait;
    use utility::ui_effect::Replier;
    use utility::ui_effect::UiContext;

    // =====================================================================
    // Form data — one variant per form type
    // =====================================================================

    #[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
    pub(crate) enum FormLocalModel {
        CreateBranch(use_case_create_branch::client::LocalModel),
        CreateCompany(use_case_create_company::client::LocalModel),
        CreateAccount(use_case_create_account::client::LocalModel),
        CreateAccountForBranch(use_case_create_account_for_branch::client::LocalModel),
        CreateJournalEntry(use_case_create_journal_entry::client::LocalModel),
    }

    // =====================================================================
    // Form messages — one variant per form type
    // =====================================================================

    #[derive(Debug, Clone)]
    pub(crate) enum FormMessage {
        CreateBranch(use_case_create_branch::client::Message),
        CreateCompany(use_case_create_company::client::Message),
        CreateAccount(use_case_create_account::client::Message),
        CreateAccountForBranch(use_case_create_account_for_branch::client::Message),
        CreateJournalEntry(use_case_create_journal_entry::client::Message),
    }

    #[derive(Debug, Clone)]
    pub(crate) enum FormsToAdd {
        CreateBranch,
        CreateCompany,
        CreateAccount,
        CreateAccountForBranch,
        CreateJournalEntry,
    }

    pub(crate) trait GlobalModel:
        use_case_create_branch::client::GlobalModel
        + use_case_create_company::client::GlobalModel
        + use_case_create_account::client::GlobalModel
        + use_case_create_account_for_branch::client::GlobalModel
        + use_case_create_journal_entry::client::GlobalModel
    {
    }

    #[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
    pub(crate) struct LocalModel {
        pub(crate) forms: Vec<FormLocalModel>,
    }

    // =====================================================================
    // Home message / intent / observe
    // =====================================================================

    #[derive(Debug, Clone)]
    pub(crate) enum Message {
        FormRelated {
            form_index: usize,
            message: Box<dyn MessageTrait>,
        },
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

    // =====================================================================
    // Home change / global change / effect
    // =====================================================================

    #[derive(Debug, Clone)]
    pub(crate) enum LocalChange {
        Forms(Vec<FormLocalModel>),
    }

    #[derive(Debug, Clone)]
    pub(crate) enum GlobalChange {
        CreateBranch(use_case_create_branch::client::GlobalChange),
        CreateCompany(use_case_create_company::client::GlobalChange),
        CreateAccount(use_case_create_account::client::GlobalChange),
        CreateAccountForBranch(use_case_create_account_for_branch::client::GlobalChange),
        CreateJournalEntry(use_case_create_journal_entry::client::GlobalChange),
    }

    #[derive(Debug, Clone)]
    pub(crate) enum Effect {
        CreateBranch(use_case_create_branch::client::Effect),
        CreateCompany(use_case_create_company::client::Effect),
        CreateAccount(use_case_create_account::client::Effect),
        CreateAccountForBranch(use_case_create_account_for_branch::client::Effect),
        CreateJournalEntry(use_case_create_journal_entry::client::Effect),
    }

    impl MessageTrait for Message {}

    // =====================================================================
    // update
    // =====================================================================

    fn form_is_submit_ok(form: &FormLocalModel) -> bool {
        match form {
            FormLocalModel::CreateBranch(lm) => lm.is_submit_ok(),
            FormLocalModel::CreateCompany(lm) => lm.is_submit_ok(),
            FormLocalModel::CreateAccount(lm) => lm.is_submit_ok(),
            FormLocalModel::CreateAccountForBranch(lm) => lm.is_submit_ok(),
            FormLocalModel::CreateJournalEntry(lm) => lm.is_submit_ok(),
        }
    }

    pub fn update(
        msg: Message,
        local_model: &LocalModel,
        global_model: &impl GlobalModel,
    ) -> (Vec<LocalChange>, Vec<GlobalChange>, Vec<Effect>) {
        match msg {
            // -------------------------------------------------------------
            // Form management
            // -------------------------------------------------------------
            Message::Intent(intent) => {
                let change = match intent {
                    Intent::AddForm(forms_to_add) => {
                        let mut forms = local_model.forms.clone();
                        forms.push(map_form_command_to_value(forms_to_add));
                        LocalChange::Forms(forms)
                    }
                    Intent::CopyForm(v) => {
                        let mut forms = local_model.forms.clone();
                        let Some(form) = forms.get(v).cloned() else {
                            return (vec![], vec![], vec![]);
                        };
                        let form = reset_transient_state(form);
                        forms.insert(v + 1, form);
                        LocalChange::Forms(forms)
                    }
                    Intent::DeleteForm(v) => {
                        let mut forms = local_model.forms.clone();
                        if v >= forms.len() {
                            return (vec![], vec![], vec![]);
                        }
                        forms.remove(v);
                        LocalChange::Forms(forms)
                    }
                };
                (vec![change], vec![], vec![])
            }

            Message::Observe(observe) => match observe {},

            // -------------------------------------------------------------
            // Form-related message: dispatch to the right sub-reducer
            // -------------------------------------------------------------
            Message::FormRelated {
                form_index,
                message,
            } => {
                let Some(form_local_model) = local_model.forms.get(form_index) else {
                    return (vec![], vec![], vec![]);
                };

                let Ok(message) = downcast_message(message) else {
                    return (vec![], vec![], vec![]);
                };

                // Static dispatch. The types of `sub_local`, `sub_global`
                // and `sub_effect` differ per arm, so we do everything in
                // one match: run the sub-reducer, apply the sub-local
                // changes to a clone, wrap the sub-global/effects.
                let mut new_forms = local_model.forms.clone();

                macro_rules! dispatch {
                    ($variant:ident, $sub:ident, $global:ident, $effect:ident) => {{
                        let (FormLocalModel::$variant(lm), FormMessage::$variant(m)) =
                            (form_local_model, message)
                        else {
                            return (vec![], vec![], vec![]);
                        };

                        let was_ok_before = form_is_submit_ok(&new_forms[form_index]);

                        let (sub_local, sub_global, sub_effect) =
                            $sub::client::update(m, lm, global_model);

                        let FormLocalModel::$variant(lm) = &mut new_forms[form_index] else {
                            unreachable!()
                        };
                        for c in sub_local {
                            $sub::client::local_apply(c, lm);
                        }

                        let is_ok_after = form_is_submit_ok(&new_forms[form_index]);
                        if !was_ok_before && is_ok_after {
                            new_forms.remove(form_index);
                        }

                        let sub_global: Vec<GlobalChange> =
                            sub_global.into_iter().map(GlobalChange::$global).collect();
                        let sub_effect: Vec<Effect> =
                            sub_effect.into_iter().map(Effect::$effect).collect();

                        (vec![LocalChange::Forms(new_forms)], sub_global, sub_effect)
                    }};
                }

                match form_local_model {
                    FormLocalModel::CreateBranch(_) => {
                        dispatch!(
                            CreateBranch,
                            use_case_create_branch,
                            CreateBranch,
                            CreateBranch
                        )
                    }
                    FormLocalModel::CreateCompany(_) => {
                        dispatch!(
                            CreateCompany,
                            use_case_create_company,
                            CreateCompany,
                            CreateCompany
                        )
                    }
                    FormLocalModel::CreateAccount(_) => {
                        dispatch!(
                            CreateAccount,
                            use_case_create_account,
                            CreateAccount,
                            CreateAccount
                        )
                    }
                    FormLocalModel::CreateAccountForBranch(_) => {
                        dispatch!(
                            CreateAccountForBranch,
                            use_case_create_account_for_branch,
                            CreateAccountForBranch,
                            CreateAccountForBranch
                        )
                    }
                    FormLocalModel::CreateJournalEntry(_) => {
                        dispatch!(
                            CreateJournalEntry,
                            use_case_create_journal_entry,
                            CreateJournalEntry,
                            CreateJournalEntry
                        )
                    }
                }
            }
        }
    }

    fn reset_transient_state(form: FormLocalModel) -> FormLocalModel {
        match form {
            FormLocalModel::CreateBranch(lm) => FormLocalModel::CreateBranch(lm.clone_user_input()),
            FormLocalModel::CreateCompany(lm) => {
                FormLocalModel::CreateCompany(lm.clone_user_input())
            }
            FormLocalModel::CreateAccount(lm) => {
                FormLocalModel::CreateAccount(lm.clone_user_input())
            }
            FormLocalModel::CreateAccountForBranch(lm) => {
                FormLocalModel::CreateAccountForBranch(lm.clone_user_input())
            }
            FormLocalModel::CreateJournalEntry(lm) => {
                FormLocalModel::CreateJournalEntry(lm.clone_user_input())
            }
        }
    }

    fn downcast_message(message: Box<dyn MessageTrait + 'static>) -> Result<FormMessage> {
        let message: Box<dyn Any> = message;
        if let Some(v) = message.downcast_ref::<use_case_create_account::client::Message>() {
            return Ok(FormMessage::CreateAccount(v.clone()));
        }

        if let Some(v) =
            message.downcast_ref::<use_case_create_account_for_branch::client::Message>()
        {
            return Ok(FormMessage::CreateAccountForBranch(v.clone()));
        }

        if let Some(v) = message.downcast_ref::<use_case_create_branch::client::Message>() {
            return Ok(FormMessage::CreateBranch(v.clone()));
        }

        if let Some(v) = message.downcast_ref::<use_case_create_company::client::Message>() {
            return Ok(FormMessage::CreateCompany(v.clone()));
        }

        if let Some(v) = message.downcast_ref::<use_case_create_journal_entry::client::Message>() {
            return Ok(FormMessage::CreateJournalEntry(v.clone()));
        }

        bail!("downcast error")
    }

    // =====================================================================
    // local_apply / global_apply / effect
    // =====================================================================

    pub fn local_apply(msg: LocalChange, local_model: &mut LocalModel) {
        match msg {
            LocalChange::Forms(i) => local_model.forms = i,
        }
    }

    pub fn global_apply(msg: GlobalChange, global_model: &mut impl GlobalModel) {
        match msg {
            GlobalChange::CreateBranch(c) => {
                use_case_create_branch::client::global_apply(c, global_model)
            }
            GlobalChange::CreateCompany(c) => {
                use_case_create_company::client::global_apply(c, global_model)
            }
            GlobalChange::CreateAccount(c) => {
                use_case_create_account::client::global_apply(c, global_model)
            }
            GlobalChange::CreateAccountForBranch(c) => {
                use_case_create_account_for_branch::client::global_apply(c, global_model)
            }
            GlobalChange::CreateJournalEntry(c) => {
                use_case_create_journal_entry::client::global_apply(c, global_model)
            }
        }
    }

    pub async fn effect(msg: Effect, replier: Replier, context: UiContext) -> Result<()> {
        match msg {
            Effect::CreateBranch(e) => {
                use_case_create_branch::client::effect(e, replier, context).await?
            }
            Effect::CreateCompany(e) => {
                use_case_create_company::client::effect(e, replier, context).await?
            }
            Effect::CreateAccount(e) => {
                use_case_create_account::client::effect(e, replier, context).await?
            }
            Effect::CreateAccountForBranch(e) => {
                use_case_create_account_for_branch::client::effect(e, replier, context).await?
            }
            Effect::CreateJournalEntry(e) => {
                use_case_create_journal_entry::client::effect(e, replier, context).await?
            }
        }
        Ok(())
    }

    // =====================================================================
    // Helpers
    // =====================================================================

    fn map_form_command_to_value(forms_to_add: FormsToAdd) -> FormLocalModel {
        match forms_to_add {
            FormsToAdd::CreateBranch => FormLocalModel::CreateBranch(Default::default()),
            FormsToAdd::CreateCompany => FormLocalModel::CreateCompany(Default::default()),
            FormsToAdd::CreateAccount => FormLocalModel::CreateAccount(Default::default()),
            FormsToAdd::CreateAccountForBranch => {
                FormLocalModel::CreateAccountForBranch(Default::default())
            }
            FormsToAdd::CreateJournalEntry => {
                FormLocalModel::CreateJournalEntry(Default::default())
            }
        }
    }
}

pub(crate) mod ui {}
