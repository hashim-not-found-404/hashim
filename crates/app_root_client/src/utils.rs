use crate::model::TypeGlobalModel;
use crate::wire::MyCaster;
use cache::cache_adapter;
use infrastructure::actors::Mpsc;
use infrastructure::actors::MultiProducerSingleConsumer;
use kernel::ui_construct::new;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;
use use_case_error_handler::client::spawn_listener;
use utility::cache::ProcessId;
use utility::ui_effect::Commander;
use utility::ui_effect::MessageTrait;
use utility::ui_effect::SigNaaaaaaaaaaaaaaaaal;
use utility_ui::my_signal::MySignal;

pub(crate) static MODEL: LazyLock<NewType> =
    LazyLock::new(|| -> NewType { NewType(Default::default()) });

struct NewType(MySignal<TypeGlobalModel>);

impl SigNaaaaaaaaaaaaaaaaal<TypeGlobalModel> for NewType {
    fn with_mut<R>(&self, f: impl FnOnce(&mut TypeGlobalModel) -> R) -> R {
        self.0.with_mut(f)
    }
}

pub(crate) fn with_model<T>(f: impl FnOnce(&TypeGlobalModel) -> T) -> T {
    let guard = MODEL.0.with_ref(f);
    guard
}

static COMMANDER: LazyLock<Commander> = LazyLock::new(|| {
    let (sender_to_error, receiver_to_error) = Mpsc::channel();

    let model = MODEL;

    let commander = new::<cache_adapter::S, TypeGlobalModel, MyCaster, MyCaster, MyCaster>(
        model,
        sender_to_error,
    );

    spawn_listener(receiver_to_error, commander.clone());

    commander
});

pub(crate) fn send<Msg: MessageTrait>(process_id: ProcessId, msg: Msg) {
    COMMANDER.send(process_id, msg);
}

pub(crate) fn init_commander_and_model() {
    LazyLock::force(&MODEL);
    LazyLock::force(&COMMANDER);
}
