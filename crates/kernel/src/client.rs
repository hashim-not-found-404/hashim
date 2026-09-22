use crate::new_types::UserUuid;
use anyhow::Result;
use infrastructure::jwt::JsonWebTokenType;
use utility::cache::MarkerCache;
use utility::dtos::Txn;
use utility::dtos::TxnNumber;
use utility::process_manager::Dialog as ProcessDialog;
use utility_ui::domain::Dialog as UiDialog;
use utility_ui::domain::HashimSignal;

pub trait Cache: MarkerCache + Sized {
    fn new() -> impl Future<Output = Result<Self>>;

    fn get_all_pending_txn(&self) -> impl Future<Output = Result<Vec<Txn<Vec<u8>>>>>;
    fn write_txn_input(&self, txn: &Txn<Vec<u8>>) -> impl Future<Output = Result<()>>;
    fn write_txn_result(&self, txn: &Txn<Vec<u8>>) -> impl Future<Output = Result<()>>;
    fn mark_input_txn_as_faild(&self, txn_number: TxnNumber) -> impl Future<Output = Result<()>>;
    fn delete_input_txn(&self, txn_number: TxnNumber) -> impl Future<Output = Result<()>>;
    fn clear_pending_txn_state(&self) -> impl Future<Output = Result<()>>;
    fn start_pending_txn_state(&self) -> impl Future<Output = Result<()>>;

    fn get_jwt(
        &self,
        user_uuid: &UserUuid,
    ) -> impl Future<Output = Result<Option<JsonWebTokenType>>>;
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
