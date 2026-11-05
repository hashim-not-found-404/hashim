use std::any::Any;
use use_case_home_client::FormId;
use use_case_home_client::FormLocalModel;
use use_case_prelude_ui::*;

/// Render a form to a Dioxus `Element`.
///
/// `FormLocalModel::render` returns a `Box<dyn Any>` — opaque to the
/// `client` module, which never names a Dioxus type. This function is
/// the single place in `use_case_home` where `Element` is named: it
/// downcasts the opaque value back to `Element`.
///
/// `model` and `commander` are supplied by the caller (`FormInstance` in
/// `app_root_client`). Each `WrapperForm` downcasts `model` to the type
/// it was created with, and uses `commander` to wire up the form's
/// `form_sender`.
pub fn render_to_element(
    form: &dyn FormLocalModel,
    form_id: FormId,
    model: &dyn Any,
    commander: &Commander,
) -> Element {
    match form.render(form_id, model, commander).downcast::<Element>() {
        Ok(e) => *e,
        Err(_) => rsx! {
            div { "form render failed" }
        },
    }
}
