use anyhow::Result;
use infrastructure::actors::MpscReceiver;
use infrastructure::actors::MpscSender;
use infrastructure::actors::Receiver;
use infrastructure::runtime::Rt;
use infrastructure::runtime::Runtime;
use std::fmt::Debug;
use std::sync::Arc;
use utility::cache::CacheStruct;
use utility::process_manager::MessageToProcessManager;
use utility::ui_effect::Aborters;
use utility::ui_effect::MessageTrait;
use utility_ui::domain::HashimSignal;

pub fn spawn_listener(
    mut receiver_to_error: MpscReceiver<anyhow::Error>,
    external_errors_signal: Arc<impl LocalModel>,
) {
    Rt::spawn_local(async move {
        let error = external_errors_signal.errors();

        loop {
            let new_err = receiver_to_error.recv().await.unwrap();

            let mut errors = error.read();
            errors.push(new_err.to_string());
            error.set(errors);
        }
    });
}

#[derive(Debug, Clone)]
pub enum Message {
    CloseError(usize),
}

impl MessageTrait for Message {}

pub trait GlobalModel: 'static {}

pub trait LocalModel: 'static {
    fn errors(&self) -> impl HashimSignal<Vec<String>>;
}

pub async fn update_generic(
    message: Message,
    global_model: Arc<impl GlobalModel>,
    local_model: Arc<impl LocalModel>,
    cache: CacheStruct,
    mut sender_to_process_manager: MpscSender<MessageToProcessManager>,
    aborters: Aborters,
) -> Result<()> {
    match message {
        Message::CloseError(i) => {
            let mut err = local_model.errors().read();

            err.remove(i);

            local_model.errors().set(err);
        }
    }

    Ok(())
}
