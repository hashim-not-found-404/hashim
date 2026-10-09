use crate::cache::ResourceName;
use crate::process_manager::ProcessId;
use crate::ui_effect::Aborter;
use crate::ui_effect::AborterReturnType;
use crate::ui_effect::MessageTrait;
use crate::ui_effect::UiContext;
use anyhow::Result;
use infrastructure::random_number::RandomNumber;
use infrastructure::random_number::Rn;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

pub async fn spawn_listener(
    mut context: UiContext,
    list_of_subscribtion: &'static [ResourceName],
    process_id: ProcessId,
    msg: impl MessageTrait + Clone,
) -> Result<()> {
    let component_id = Rn::generate() as u16;

    context
        .cache
        .send_subs_to_cache_actor(component_id, list_of_subscribtion, move || {
            context.sender_to_commander.send(process_id, msg.clone());
        })
        .await?;

    let a = move || -> AborterReturnType {
        Box::pin(async move {
            context
                .cache
                .send_unsubs_to_cache_actor(component_id)
                .await?;

            Ok(())
        })
    };
    let aborter = Aborter::new(a);

    context.aborters.register(process_id, aborter);
    Ok(())
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(bound(
    deserialize = "Input: DeserializeOwned, Ok: DeserializeOwned, Error: DeserializeOwned",
    serialize = "Input: Serialize, Ok: Serialize, Error: Serialize",
))]
pub enum GenricAsyncState<Input, Ok, Error>
where
    Input: Debug + Clone,
    Ok: Debug + Clone,
    Error: Debug + Clone,
{
    #[default]
    Idle,
    Loading {
        input: Input,
    },
    Success {
        input: Input,
        ok: Ok,
    },
    Failure {
        input: Input,
        error: Error,
    },
}

impl<Input, Ok, Error> GenricAsyncState<Input, Ok, Error>
where
    Input: Debug + Clone,
    Ok: Debug + Clone,
    Error: Debug + Clone,
{
    pub fn is_loading(&self) -> bool {
        matches!(self, GenricAsyncState::Loading { .. })
    }
}
