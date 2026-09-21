use crate::model::TypeModel;
use infrastructure::actors::MpscSender;
use std::pin::Pin;
use std::sync::Arc;
use utility::cache::CacheStruct;
use utility::process_manager::MessageToProcessManager;
use utility::ui_effect::Aborters;
use utility::ui_effect::UpdaterTrait;

struct WrapperMessage(use_case_create_account::client::Message);
impl UpdaterTrait for WrapperMessage {
    type Mdl = TypeModel;

    fn update(
        self: Box<Self>,
        model: Arc<Self::Mdl>,
        cache: CacheStruct,
        sender_to_process_manager: MpscSender<MessageToProcessManager>,
        aborters: Aborters,
    ) -> Pin<Box<dyn Future<Output = ()>>> {
        Box::pin(async move {
            self.0
                .update_generic(
                    model.clone(),
                    model.page_create_account.clone(),
                    cache,
                    sender_to_process_manager,
                )
                .await
        })
    }
}
