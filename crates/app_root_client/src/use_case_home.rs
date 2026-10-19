pub(crate) mod client {
    use anyhow::Result;
    use anyhow::bail;
    use infrastructure::random_number::RandomNumber;
    use infrastructure::random_number::Rn;
    use serde::Deserialize;
    use serde::Serialize;
    use std::any::Any;
    use utility::ui_effect::MessageTrait;
    use utility::ui_effect::Replier;
    use utility::ui_effect::UiContext;

    // =====================================================================
    // Form id — home-owned identity for a live form instance
    // =====================================================================

    /// Stable identity for one live form instance.
    ///
    /// The home assigns it when a form is created and keeps it for the
    /// form’s whole life. It is *not* the subscription’s `ProcessId` and
    /// it is not visible to any use case. It exists so that messages
    /// which fire asynchronously — e.g. a 5-second submit timer — can be
    /// routed back to the correct form even if other forms have been
    /// added, removed, or reordered in the meantime.
    #[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Deserialize, Serialize)]
    pub(crate) struct FormId(u32);

    impl Default for FormId {
        fn default() -> Self {
            FormId(Rn::generate() as u32)
        }
    }

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

    /// A form together with the home-owned id that identifies it.
    ///
    /// `forms` stores these instead of bare `FormLocalModel`s so that any
    /// message addressed by `FormId` can find its form regardless of how
    /// the list has shifted since the message was dispatched.
    #[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
    pub(crate) struct FormEntry {
        pub(crate) id: FormId,
        pub(crate) form: FormLocalModel,
    }

    impl FormEntry {
        pub(crate) fn new(form: FormLocalModel) -> Self {
            Self {
                id: FormId::default(),
                form,
            }
        }
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
        pub(crate) forms: Vec<FormEntry>,
    }

    // =====================================================================
    // Home message / intent / observe
    // =====================================================================

    #[derive(Debug, Clone)]
    pub(crate) enum Message {
        FormRelated {
            form_id: FormId,
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
        Forms(Vec<FormEntry>),
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
    // Helpers
    // =====================================================================

    /// `true` iff the given form’s last submit ended in `Success`.
    ///
    /// Journal-entry’s `LocalModel` does not yet carry an `AsyncState`,
    /// so it always reports `false` and is never auto-closed — that is
    /// intentional for this pass.
    fn form_is_submit_ok(form: &FormLocalModel) -> bool {
        match form {
            FormLocalModel::CreateBranch(lm) => lm.is_submit_ok(),
            FormLocalModel::CreateCompany(lm) => lm.is_submit_ok(),
            FormLocalModel::CreateAccount(lm) => lm.is_submit_ok(),
            FormLocalModel::CreateAccountForBranch(lm) => lm.is_submit_ok(),
            FormLocalModel::CreateJournalEntry(_) => false,
        }
    }

    /// Asks each use case to produce its own unsubscribe effect by
    /// feeding it the `Intent::UnSubscribe` it already understands, and
    /// wrapping the resulting effect in the home `Effect` enum.
    ///
    /// This is the only place in the home where subscriptions are torn
    /// down. It runs when the user clicks X (`Intent::DeleteForm`) and
    /// when a submit flips a form to `Success` inside `dispatch!`. The
    /// use case itself never needs to know about lifecycle — it just
    /// answers the intent it already knows how to answer.
    ///
    /// `create_company` does not subscribe, hence the empty arm.
    fn teardown_effects<G>(form: &FormLocalModel, global_model: &G) -> Vec<Effect>
    where
        G: GlobalModel,
    {
        match form {
            FormLocalModel::CreateBranch(lm) => {
                let msg = use_case_create_branch::client::Message::Intent(
                    use_case_create_branch::client::Intent::UnSubscribe,
                );
                let (_, _, effects) = use_case_create_branch::client::update(msg, lm, global_model);
                effects.into_iter().map(Effect::CreateBranch).collect()
            }
            FormLocalModel::CreateAccount(lm) => {
                let msg = use_case_create_account::client::Message::Intent(
                    use_case_create_account::client::Intent::UnSubscribe,
                );
                let (_, _, effects) =
                    use_case_create_account::client::update(msg, lm, global_model);
                effects.into_iter().map(Effect::CreateAccount).collect()
            }
            FormLocalModel::CreateAccountForBranch(lm) => {
                let msg = use_case_create_account_for_branch::client::Message::Intent(
                    use_case_create_account_for_branch::client::Intent::UnSubscribe,
                );
                let (_, _, effects) =
                    use_case_create_account_for_branch::client::update(msg, lm, global_model);
                effects
                    .into_iter()
                    .map(Effect::CreateAccountForBranch)
                    .collect()
            }
            FormLocalModel::CreateJournalEntry(lm) => {
                let msg = use_case_create_journal_entry::client::Message::Intent(
                    use_case_create_journal_entry::client::Intent::UnSubscribe,
                );
                let (_, _, effects) =
                    use_case_create_journal_entry::client::update(msg, lm, global_model);
                effects
                    .into_iter()
                    .map(Effect::CreateJournalEntry)
                    .collect()
            }
            // No subscription → no teardown effect.
            FormLocalModel::CreateCompany(_) => Vec::new(),
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

    fn map_form_command_to_value(forms_to_add: FormsToAdd) -> FormEntry {
        match forms_to_add {
            FormsToAdd::CreateBranch => {
                FormEntry::new(FormLocalModel::CreateBranch(Default::default()))
            }
            FormsToAdd::CreateCompany => {
                FormEntry::new(FormLocalModel::CreateCompany(Default::default()))
            }
            FormsToAdd::CreateAccount => {
                FormEntry::new(FormLocalModel::CreateAccount(Default::default()))
            }
            FormsToAdd::CreateAccountForBranch => {
                FormEntry::new(FormLocalModel::CreateAccountForBranch(Default::default()))
            }
            FormsToAdd::CreateJournalEntry => {
                FormEntry::new(FormLocalModel::CreateJournalEntry(Default::default()))
            }
        }
    }

    // =====================================================================
    // update
    // =====================================================================

    pub fn update(
        msg: Message,
        local_model: &LocalModel,
        global_model: &impl GlobalModel,
    ) -> (Vec<LocalChange>, Vec<GlobalChange>, Vec<Effect>) {
        match msg {
            // -------------------------------------------------------------
            // Form management (index-based — driven by synchronous clicks)
            // -------------------------------------------------------------
            Message::Intent(intent) => {
                let (change, effect) = match intent {
                    Intent::AddForm(forms_to_add) => {
                        let mut forms = local_model.forms.clone();
                        forms.push(map_form_command_to_value(forms_to_add));
                        (LocalChange::Forms(forms), vec![])
                    }
                    Intent::CopyForm(v) => {
                        let mut forms = local_model.forms.clone();
                        let Some(entry) = forms.get(v).cloned() else {
                            return (vec![], vec![], vec![]);
                        };
                        let fresh_form = reset_transient_state(entry.form);
                        // Fresh `FormEntry::new` gives the copy its own
                        // brand-new `FormId` — the two forms never collide.
                        forms.insert(v + 1, FormEntry::new(fresh_form));
                        (LocalChange::Forms(forms), vec![])
                    }
                    Intent::DeleteForm(v) => {
                        let mut forms = local_model.forms.clone();
                        if v >= forms.len() {
                            return (vec![], vec![], vec![]);
                        }
                        let removed = forms.remove(v);
                        let effect = teardown_effects(&removed.form, global_model);
                        (LocalChange::Forms(forms), effect)
                    }
                };
                (vec![change], vec![], effect)
            }

            Message::Observe(observe) => match observe {},

            // -------------------------------------------------------------
            // Form-related message: routed by `FormId`, not by position.
            // -------------------------------------------------------------
            Message::FormRelated { form_id, message } => {
                // Find the *current* position of the form with this id.
                // If the form has been removed (or the id never existed),
                // the message is dropped — a stale message for a real
                // removal, not a spurious drop caused by reordering.
                let Some(form_index) = local_model.forms.iter().position(|e| e.id == form_id)
                else {
                    return (vec![], vec![], vec![]);
                };

                let form_local_model = &local_model.forms[form_index].form;

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

                        // Snapshot the form’s submit state *before* the
                        // sub-reducer runs. If the sub-reducer flips it
                        // from not-success to success, this message is the
                        // one that completed the submit → the home removes
                        // the form and asks the use case for the teardown
                        // effect.
                        let was_ok_before = form_is_submit_ok(&new_forms[form_index].form);

                        let (sub_local, sub_global, sub_effect) =
                            $sub::client::update(m, lm, global_model);

                        let FormLocalModel::$variant(lm) = &mut new_forms[form_index].form else {
                            unreachable!()
                        };
                        for c in sub_local {
                            $sub::client::local_apply(c, lm);
                        }

                        let is_ok_after = form_is_submit_ok(&new_forms[form_index].form);

                        let sub_global: Vec<GlobalChange> =
                            sub_global.into_iter().map(GlobalChange::$global).collect();
                        let mut sub_effect: Vec<Effect> =
                            sub_effect.into_iter().map(Effect::$effect).collect();

                        if !was_ok_before && is_ok_after {
                            let removed = new_forms.remove(form_index);
                            sub_effect.extend(teardown_effects(&removed.form, global_model));
                        }

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
}

pub(crate) mod ui {}
