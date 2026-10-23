use crate::client::FormId;
use crate::client::FormLocalModel;
use dioxus::prelude::*;

/// Render a form to a Dioxus `Element`.
///
/// `FormLocalModel::render` returns a `Box<dyn Any>` — opaque to the
/// `client` module, which never names a Dioxus type. This function is
/// the single place in `use_case_home` where `Element` is named: it
/// downcasts the opaque value back to `Element`.
pub fn render_to_element(form: &dyn FormLocalModel, form_id: FormId) -> Element {
    match form.render(form_id).downcast::<Element>() {
        Ok(e) => *e,
        Err(_) => rsx! {
            div { "form render failed" }
        },
    }
}
