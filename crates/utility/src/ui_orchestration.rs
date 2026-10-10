use crate::cache::ProcessId;
use crate::cache::ResourceName;
use crate::ui_effect::MessageTrait;
use crate::ui_effect::UiContext;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

#[derive(Debug, Clone, Copy)]
pub enum UserConsent {
    WaitForServerResponse,
    DontWaitForServerResponse,
    CancelOperation,
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

use crate::cache::CachingStrategy;
use crate::cache::Response;
use crate::cache::ResponseFunction;
use crate::cache::TraitOperationClientError;
use crate::cache::TraitOperationClientInput;
use crate::cache::TraitOperationClientOk;
use crate::cache::TypeOperationClientInput;
use crate::dtos::TxnNumber;
use anyhow::Context;
use anyhow::anyhow;
use std::any::Any;
use std::pin::Pin;
use std::sync::Arc;

pub trait UseCaseClient: 'static {
    type Input: TraitOperationClientInput + 'static;
    type AsyncInput: Clone + 'static;
    type Ok: TraitOperationClientOk + Clone + 'static;
    type Error: TraitOperationClientError + 'static;
    type Message: MessageTrait;

    fn build_input(input: &Self::AsyncInput) -> Self::Input;
    fn msg_success_submit(input: Self::AsyncInput, ok: Self::Ok) -> Self::Message;
    fn msg_failure(input: Self::AsyncInput, error: Self::Error) -> Self::Message;
    fn msg_timeout() -> Self::Message;
    fn msg_success_check() -> Self::Message;
    fn msg_refresh() -> Self::Message;
}

pub async fn handle_submit<T: UseCaseClient>(
    process_id: ProcessId,
    input: T::AsyncInput,
    mut context: UiContext,
    is_to_server: bool,
) -> Result<()> {
    let data: TypeOperationClientInput = Arc::new(T::build_input(&input));
    let sender = context.sender_to_commander.clone();

    let f: ResponseFunction = Box::new(move |a| -> Pin<Box<dyn Future<Output = Result<()>>>> {
        Box::pin({
            let sender = sender.clone();
            let input = input.clone();
            async move {
                let msg = match a {
                    Response::ServerCannotBeReached => T::msg_timeout(),
                    Response::Data { data } => match data {
                        Ok(ok) => {
                            let a: &dyn Any = &*ok;
                            let a: &T::Ok = a.downcast_ref().context("downcast error")?;
                            T::msg_success_submit(input.clone(), a.clone())
                        }
                        Err(err) => {
                            let a: Box<dyn Any> = err;
                            let a: Box<T::Error> =
                                a.downcast().map_err(|_| anyhow!("downcast error"))?;
                            T::msg_failure(input, *a)
                        }
                    },
                };
                sender.async_send(process_id, msg).await?;
                Ok(())
            }
        })
    });

    let strategy = if is_to_server {
        CachingStrategy::WriteServerOnly
    } else {
        CachingStrategy::WriteCacheOnly
    };

    context
        .cache
        .send_to_cache_actor(strategy, TxnNumber::default(), data, f)
        .await
}

pub async fn handle_check<T: UseCaseClient>(
    process_id: ProcessId,
    input: T::AsyncInput,
    mut context: UiContext,
) -> Result<()> {
    let data: TypeOperationClientInput = Arc::new(T::build_input(&input));
    let sender = context.sender_to_commander.clone();

    let f: ResponseFunction = Box::new(move |a| -> Pin<Box<dyn Future<Output = Result<()>>>> {
        Box::pin({
            let sender = sender.clone();
            let input = input.clone();
            async move {
                if let Response::Data { data } = a {
                    let msg = match data {
                        Ok(_) => T::msg_success_check(),
                        Err(err) => {
                            let a: Box<dyn Any> = err;
                            let a: Box<T::Error> =
                                a.downcast().map_err(|_| anyhow!("downcast error"))?;
                            T::msg_failure(input, *a)
                        }
                    };
                    sender.async_send(process_id, msg).await?;
                }
                Ok(())
            }
        })
    });

    context
        .cache
        .send_to_cache_actor(
            CachingStrategy::ReadCacheOnly,
            TxnNumber::default(),
            data,
            f,
        )
        .await
}

pub async fn handle_refresh<T: UseCaseClient>(
    process_id: ProcessId,
    input: T::AsyncInput,
    mut context: UiContext,
) -> Result<()> {
    let data: TypeOperationClientInput = Arc::new(T::build_input(&input));
    let sender = context.sender_to_commander.clone();

    let f: ResponseFunction = Box::new(move |a| -> Pin<Box<dyn Future<Output = Result<()>>>> {
        Box::pin({
            let sender = sender.clone();
            let input = input.clone();
            async move {
                if let Response::Data { data } = a {
                    let msg = match data {
                        Ok(ok) => {
                            let a: Arc<dyn Any> = ok;
                            let a: &T::Ok = a.downcast_ref().context("downcast error")?;
                            T::msg_success_submit(input, a.clone())
                        }
                        Err(err) => {
                            let a: Box<dyn Any> = err;
                            let a: Box<T::Error> =
                                a.downcast().map_err(|_| anyhow!("downcast error"))?;
                            T::msg_failure(input, *a)
                        }
                    };
                    sender.async_send(process_id, msg).await?;
                }
                Ok(())
            }
        })
    });

    context
        .cache
        .send_to_cache_actor(
            CachingStrategy::ReadCacheOnly,
            TxnNumber::default(),
            data,
            f,
        )
        .await
}

pub async fn handle_subscribe<T: UseCaseClient>(
    process_id: ProcessId,
    mut context: UiContext,
    list_of_subscribtion: &'static [ResourceName],
) -> Result<()> {
    context
        .cache
        .send_subs_to_cache_actor(process_id, list_of_subscribtion, move || {
            context
                .sender_to_commander
                .send(process_id, T::msg_refresh());
        })
        .await
}

pub async fn handle_unsubscribe<T: UseCaseClient>(
    process_id: ProcessId,
    mut context: UiContext,
) -> Result<()> {
    context.cache.send_unsubs_to_cache_actor(process_id).await
}
