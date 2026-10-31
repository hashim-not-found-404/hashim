use crate::domain::Error;
use crate::domain::Input;
use crate::domain::MyResult;
use crate::domain::Ok;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use std::any::Any;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::Arc;
use use_case_prelude_client::*;

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

    let (sender, mut receiver) = Mpsc::channel();

    let f: ResponseFunction = Box::new(
        move |a: Response| -> Pin<Box<dyn Future<Output = Result<()>>>> {
            let mut sender = sender.clone();
            Box::pin(async move {
                sender.send(a).await?;
                Ok(())
            })
        },
    );

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
        Box::new(|_: Response| -> Pin<Box<dyn Future<Output = Result<()>>>> {
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
