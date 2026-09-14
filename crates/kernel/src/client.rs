use crate::new_types::JsonWebTokenType;
use crate::new_types::UserUuid;
use utility::process_manager::Dialog as ProcessDialog;
use utility_ui::domain::Dialog as UiDialog;
use utility_ui::domain::HashimSignal;

pub trait Cache: 'static {
    fn get_jwt(&self, user_uuid: &UserUuid) -> impl Future<Output = Option<JsonWebTokenType>>;
}

#[derive(Clone)]
pub struct DialogSignalAdapter<S: HashimSignal<UiDialog>>(pub S);

impl<S: HashimSignal<UiDialog>> ProcessDialog for DialogSignalAdapter<S> {
    fn show(&self) {
        self.0.set(UiDialog::Show);
    }

    fn hide(&self) {
        self.0.set(UiDialog::Hide);
    }
}
