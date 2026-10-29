use crate::domain::Error;
use crate::domain::Input;
use crate::domain::MyResult;
use crate::domain::Ok;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;
use infrastructure::actors::Mpsc;
use infrastructure::actors::MultiProducerSingleConsumer;
use infrastructure::actors::Receiver;
use infrastructure::actors::Sender;
use kernel::client::Cache;
use kernel::types::DatabaseRead;
use kernel::types::MyErrorTrait;
use std::any::Any;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::Arc;
use utility::cache::CacheStruct;
use utility::cache::CachingStrategy;
use utility::cache::ResourceName;
use utility::cache::Response;
use utility::cache::ResponseFunction;
use utility::cache::TraitOperationClientError;
use utility::cache::TraitOperationClientInput;
use utility::cache::TraitOperationClientOk;
use utility::cache::TypeOperationClientError;
use utility::cache::TypeOperationClientInput;
use utility::cache::TypeOperationClientResult;
use utility::cache::new_resource_name;
use utility::dtos::TxnNumber;

const RESOURCES_NAME_TO_POKE: &[ResourceName] = &[new_resource_name("journals")];

impl TraitOperationClientOk for Ok {
    fn subs_to_poke(&self) -> &'static [ResourceName] {
        RESOURCES_NAME_TO_POKE
    }
}

impl TraitOperationClientError for Error {
    fn subs_to_poke(&self) -> &'static [ResourceName] {
        RESOURCES_NAME_TO_POKE
    }
}

impl TraitOperationClientInput for Input {
    fn user_uuid(&self) -> Option<[u8; 16]> {
        Some(*self.user_uuid.deref().deref())
    }

    fn state_less_check(&self) -> Option<TypeOperationClientError> {
        let errr = self.state_less_check();
        if errr.is_there_error() {
            Some(Box::new(errr))
        } else {
            None
        }
    }
}

pub async fn check_input<
    Ch: Cache,
    DBReader: for<'a> DatabaseRead<Db<'a> = Ch, Input = ReadInput, Output = ReadOutput>,
>(
    input: &Input,
    cache: &mut Ch,
) -> Result<TypeOperationClientResult> {
    let errr = input.state_less_check();

    if errr.is_there_error() {
        return Ok(Err(Box::new(errr)));
    }

    match input.state_full_operation::<DBReader>(cache).await? {
        Ok(ok) => Ok(Ok(Arc::new(ok))),
        Err(err) => Ok(Err(Box::new(err))),
    }
}

pub async fn get_data(
    mut cache: CacheStruct,
    is_from_server: bool,
    input: Input,
) -> Result<MyResult> {
    let data: TypeOperationClientInput = Arc::new(input);

    let (sender, receiver) = Mpsc::channel();

    let f: ResponseFunction =
        Box::new(|a: Response| -> Pin<Box<dyn Future<Output = Result<()>>>> {
            Box::pin(async { sender.send(a).await })
        });

    let caching_strategy = if is_from_server {
        CachingStrategy::ReadServerOnly
    } else {
        CachingStrategy::ReadCacheOnly
    };

    cache
        .send_to_cache_actor(caching_strategy, TxnNumber::default(), data, f)
        .await?;

    let a = receiver.recv().await?;

    let a = match a {
        Response::ServerCannotBeReached => bail!("server_cannot_be_reached"),
        Response::Data { data } => match data {
            Ok(ok) => {
                let a: &dyn Any = &*ok;
                let a: &Ok = a.downcast_ref().context("downcast error")?;
                Ok(a.clone())
            }
            Err(err) => {
                let a: Box<dyn Any> = err;
                let a: Box<Error> = a.downcast().map_err(|_| anyhow!("downcast error"))?;
                Err(a.as_ref().clone())
            }
        },
    };

    Ok(a)
}

pub async fn fetch(mut cache: CacheStruct, input: Input) -> Result<()> {
    let data: TypeOperationClientInput = Arc::new(input);

    let f: ResponseFunction =
        Box::new(|a: Response| -> Pin<Box<dyn Future<Output = Result<()>>>> {
            Box::pin(async { Ok(()) })
        });

    cache
        .send_to_cache_actor(
            CachingStrategy::ReadServerOnly,
            TxnNumber::default(),
            data,
            f,
        )
        .await?;

    Ok(())
}
