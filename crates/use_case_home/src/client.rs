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

impl From<u32> for FormId {
    fn from(value: u32) -> Self {
        FormId(value)
    }
}

impl Default for FormId {
    fn default() -> Self {
        FormId(Rn::generate() as u32)
    }
}

// =====================================================================
// The plug-in contract
//
// `Any` is a supertrait so callers can upcast `&dyn FormLocalModel` to
// `&dyn Any` and downcast to a concrete use-case `LocalModel`.
//
// `#[typetag::serde]` makes `Box<dyn FormLocalModel>` Serialize/Deserialize
// so that `FormEntry` and `LocalModel` can derive serde traits (needed by
// `TypeGlobalModel` in `app_root_client`).
// =====================================================================

#[typetag::serde]
pub trait FormLocalModel: Any + Debug + Send + Sync + 'static + DynClone {
    fn clone_user_input(&self) -> Box<dyn FormLocalModel>;
    fn is_submit_ok(&self) -> bool;
    fn is_loading(&self) -> bool;
    fn dyn_eq(&self, other: &dyn FormLocalModel) -> bool;
    fn handle(&mut self, message: Box<dyn MessageTrait>, global: &dyn Any) -> FormUpdate;
    fn render(&self, form_id: FormId) -> Box<dyn Any>;
}

clone_trait_object!(FormLocalModel);

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

pub trait FormGlobalChange: Debug + Send + Sync + 'static + DynClone {
    fn apply(&self, global: &mut dyn Any);
}

clone_trait_object!(FormGlobalChange);

pub trait FormEffect: Debug + Send + Sync + 'static + DynClone {
    fn effect(
        &self,
        replier: Replier,
        context: UiContext,
    ) -> Pin<Box<dyn Future<Output = Result<()>>>>;
}

clone_trait_object!(FormEffect);

// =====================================================================
// Form entry
//
// Fields are `pub` so `app_root_client` can read `entry.id` and
// `entry.form`. `PartialEq` is manual because `Box<dyn FormLocalModel>`
// does not auto-derive it (the JSON form still derives it via typetag,
// but `Box<dyn Trait>` needs a hand-written impl for the memoization
// that Dioxus `#[component]` requires).
// =====================================================================

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FormEntry {
    pub id: FormId,
    pub form: Box<dyn FormLocalModel>,
}

impl PartialEq for FormEntry {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.form.dyn_eq(other.form.as_ref())
    }
}

impl FormEntry {
    pub fn new(form: Box<dyn FormLocalModel>) -> Self {
        Self {
            id: FormId::default(),
            form,
        }
    }
}

// =====================================================================
// Global model
// =====================================================================

pub trait GlobalModel: Any {}

// =====================================================================
// Local model — `forms` is `pub` so `app_ui.rs` can read it
// =====================================================================

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct LocalModel {
    pub forms: Vec<FormEntry>,
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
    CopyForm(FormId),
    DeleteForm(FormId),
}

#[derive(Debug, Clone)]
pub enum Observe {}

// =====================================================================
// Home change
//
// `GlobalChange` and `Effect` gained `Clone` and `Debug` (wire.rs
// requires them — it wraps and clones these in its own wire structs).
// =====================================================================

#[derive(Debug, Clone)]
pub enum LocalChange {
    Forms(Vec<FormEntry>),
}

#[derive(Debug, Clone, Default)]
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

#[derive(Debug, Clone, Default)]
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
                    (vec![LocalChange::Forms(forms)], vec![])
                }
                Intent::CopyForm(id) => {
                    let mut forms = local_model.forms.clone();
                    let Some(position) = forms.iter().position(|e| e.id == id) else {
                        return (vec![], vec![], vec![]);
                    };
                    let fresh_form = forms[position].form.clone_user_input();
                    forms.insert(position + 1, FormEntry::new(fresh_form));
                    (vec![LocalChange::Forms(forms)], vec![])
                }
                Intent::DeleteForm(id) => {
                    let mut forms = local_model.forms.clone();
                    let Some(position) = forms.iter().position(|e| e.id == id) else {
                        return (vec![], vec![], vec![]);
                    };
                    if forms[position].form.is_loading() {
                        return (vec![], vec![], vec![]);
                    }
                    forms.remove(position);
                    (vec![LocalChange::Forms(forms)], vec![])
                }
            };
            (change, vec![], effect)
        }

        Message::Observe(observe) => match observe {},

        Message::FormRelated { form_id, message } => {
            let Some(form_index) = local_model.forms.iter().position(|e| e.id == form_id) else {
                return (vec![], vec![], vec![]);
            };

            let mut new_forms = local_model.forms.clone();
            let was_ok_before = new_forms[form_index].form.is_submit_ok();

            let update = new_forms[form_index]
                .form
                .handle(message, global_model as &dyn Any);

            let is_ok_after = new_forms[form_index].form.is_submit_ok();

            let mut effects: Vec<Effect> = update.effects.into_iter().map(Effect::new).collect();

            if !was_ok_before && is_ok_after {
                new_forms.remove(form_index);
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
        e.effect(replier.clone(), context.clone()).await?;
    }
    Ok(())
}
