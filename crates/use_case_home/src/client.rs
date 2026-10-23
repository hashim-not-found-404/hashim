use anyhow::Result;
use dyn_clone::DynClone;
use dyn_clone::clone_trait_object;
use infrastructure::random_number::RandomNumber;
use infrastructure::random_number::Rn;
use serde::Deserialize;
use serde::Serialize;
use std::any::Any;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use utility::ui_effect::MessageTrait;
use utility::ui_effect::Replier;
use utility::ui_effect::UiContext;

// =====================================================================
// Form id — home-owned identity for a live form instance
// =====================================================================

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct FormId(u32);

impl Default for FormId {
    fn default() -> Self {
        FormId(Rn::generate() as u32)
    }
}

// =====================================================================
// The plug-in contract
//
// `FormLocalModel` is what the home stores. It does not name any use
// case. Each use case implements it for its own `LocalModel`. The
// message downcast (to the use case's concrete `Message`) and the
// global downcast (to the app's `TypeGlobalModel`) both happen inside
// `handle`, in the use case crate.
// =====================================================================

pub trait FormLocalModel: Debug + Send + Sync + 'static + DynClone {
    /// Clone of this form with transient state (errors, async state,
    /// dialog) reset. Used by `Intent::CopyForm`.
    fn clone_user_input(&self) -> Box<dyn FormLocalModel>;

    /// Whether the last submit succeeded. The home checks this before
    /// and after `handle` to auto-close a form when its submit lands.
    fn is_submit_ok(&self) -> bool;

    /// Route one message addressed to this form into the form's own
    /// reducer. Downcasting and dispatching live inside the impl.
    fn handle(&mut self, message: Box<dyn MessageTrait>, global: &dyn Any) -> FormUpdate;
}

clone_trait_object!(FormLocalModel);

/// What `handle` returns. The form's *new state* is `&mut self` — no
/// separate local channel is needed. Only global changes and effects
/// need to be ferried back out, because their concrete types belong
/// to the use case and the home cannot name them.
pub struct FormUpdate {
    pub global: Vec<Box<dyn FormGlobalChange>>,
    pub effects: Vec<Box<dyn FormEffect>>,
}

impl FormUpdate {
    pub fn empty() -> Self {
        Self {
            global: vec![],
            effects: vec![],
        }
    }
}

/// One global-model change. The impl downcasts `global` to the concrete
/// type it is generic over and calls its own `global_apply`.
pub trait FormGlobalChange: Debug + Send + Sync + 'static {
    fn apply(&self, global: &mut dyn Any);
}

/// One side effect. The impl owns the concrete effect and calls its
/// own `effect` function inside `run`.
pub trait FormEffect: Debug + Send + Sync + 'static {
    fn run(
        self: Box<Self>,
        replier: Replier,
        context: UiContext,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>>;
}

// =====================================================================
// Form entry
// =====================================================================

#[derive(Debug, Clone)]
pub struct FormEntry {
    pub(crate) id: FormId,
    pub(crate) form: Box<dyn FormLocalModel>,
}

impl FormEntry {
    pub(crate) fn new(form: Box<dyn FormLocalModel>) -> Self {
        Self {
            id: FormId::default(),
            form,
        }
    }
}

// =====================================================================
// Global model
//
// Marker trait. Any concrete global type implements it for free because
// `Any` is blanket-impl'd for `'static`. `&impl GlobalModel` can be
// cast to `&dyn Any` with `as &dyn Any`.
// =====================================================================

pub trait GlobalModel: Any {}

// =====================================================================
// Local model
// =====================================================================

#[derive(Debug, Clone, Default)]
pub struct LocalModel {
    pub(crate) forms: Vec<FormEntry>,
}

// =====================================================================
// Home message / intent / observe
// =====================================================================

#[derive(Debug, Clone)]
pub enum Message {
    FormRelated {
        form_id: FormId,
        message: Box<dyn MessageTrait>,
    },
    Intent(Intent),
    Observe(Observe),
}

#[derive(Debug, Clone)]
pub enum Intent {
    AddForm(Box<dyn FormLocalModel>),
    CopyForm(usize),
    DeleteForm(usize),
}

#[derive(Debug, Clone)]
pub enum Observe {}

// =====================================================================
// Home change
// =====================================================================

#[derive(Debug, Clone)]
pub enum LocalChange {
    Forms(Vec<FormEntry>),
}

/// Container for global changes produced by one dispatch. Each element
/// is one boxed change, so the home can `Vec<GlobalChange>` uniformly
/// while the concrete types inside stay per-use-case.
#[derive(Debug, Default)]
pub struct GlobalChange {
    pub(crate) changes: Vec<Box<dyn FormGlobalChange>>,
}

impl GlobalChange {
    pub(crate) fn new(change: Box<dyn FormGlobalChange>) -> Self {
        Self {
            changes: vec![change],
        }
    }
}

/// Same shape for effects.
#[derive(Default)]
pub struct Effect {
    pub(crate) effects: Vec<Box<dyn FormEffect>>,
}

impl Effect {
    pub(crate) fn new(effect: Box<dyn FormEffect>) -> Self {
        Self {
            effects: vec![effect],
        }
    }
}

impl MessageTrait for Message {}

// =====================================================================
// update
// =====================================================================

pub fn update(
    msg: Message,
    local_model: &LocalModel,
    global_model: &impl GlobalModel,
) -> (Vec<LocalChange>, Vec<GlobalChange>, Vec<Effect>) {
    match msg {
        Message::Intent(intent) => {
            let (change, effect) = match intent {
                Intent::AddForm(form) => {
                    let mut forms = local_model.forms.clone();
                    forms.push(FormEntry::new(form));
                    (LocalChange::Forms(forms), vec![])
                }
                Intent::CopyForm(v) => {
                    let mut forms = local_model.forms.clone();
                    let Some(entry) = forms.get(v).cloned() else {
                        return (vec![], vec![], vec![]);
                    };
                    let fresh_form = entry.form.clone_user_input();
                    forms.insert(v + 1, FormEntry::new(fresh_form));
                    (LocalChange::Forms(forms), vec![])
                }
                Intent::DeleteForm(v) => {
                    let mut forms = local_model.forms.clone();
                    if v >= forms.len() {
                        return (vec![], vec![], vec![]);
                    }
                    forms.remove(v);
                    (LocalChange::Forms(forms), vec![])
                }
            };
            (vec![change], vec![], effect)
        }

        Message::Observe(observe) => match observe {},

        Message::FormRelated { form_id, message } => {
            let Some(form_index) = local_model.forms.iter().position(|e| e.id == form_id) else {
                return (vec![], vec![], vec![]);
            };

            // Work on a clone so the reducer stays pure — the caller
            // applies the result via `local_apply`.
            let mut new_forms = local_model.forms.clone();

            let was_ok_before = new_forms[form_index].form.is_submit_ok();

            // `handle` mutates `new_forms[form_index].form` in place
            // and returns the non-local channels.
            let update = new_forms[form_index]
                .form
                .handle(message, global_model as &dyn Any);

            let is_ok_after = new_forms[form_index].form.is_submit_ok();

            let mut effects: Vec<Effect> = update.effects.into_iter().map(Effect::new).collect();

            // If this message completed the submit, auto-close the form.
            if !was_ok_before && is_ok_after {
                new_forms.remove(form_index);
                // (teardown hook — add when a use case actually needs one)
            }

            let global_changes: Vec<GlobalChange> =
                update.global.into_iter().map(GlobalChange::new).collect();

            (
                vec![LocalChange::Forms(new_forms)],
                global_changes,
                effects.drain(..).collect(),
            )
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

pub fn global_apply(msg: GlobalChange, global_model: &mut dyn Any) {
    for change in msg.changes {
        change.apply(global_model);
    }
}

pub async fn effect(msg: Effect, replier: Replier, context: UiContext) -> Result<()> {
    for e in msg.effects {
        e.run(replier.clone(), context.clone()).await?;
    }
    Ok(())
}
