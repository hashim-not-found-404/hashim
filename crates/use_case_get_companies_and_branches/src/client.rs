use crate::domain::Error;
use crate::domain::Input;
use crate::domain::Ok;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::Arc;
use use_case_prelude_client::*;

const RESOURCES_NAME_TO_POKE: &[ResourceName] = &[
    new_resource_name("companies"),
    new_resource_name("branches"),
];

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

    let ok = input.state_full_operation::<DBReader>(cache).await?;

    Ok(Ok(Arc::new(ok)))
}

pub async fn fetch(mut cache: CacheStruct, user_uuid: UserUuid) -> Result<()> {
    let input: TypeOperationClientInput = Arc::new(Input { user_uuid });

    let f: ResponseFunction =
        Box::new(|_: Response| -> Pin<Box<dyn Future<Output = Result<()>>>> {
            Box::pin(async { Ok(()) })
        });

    cache
        .send_to_cache_actor(
            CachingStrategy::ReadServerOnly,
            TxnNumber::default(),
            input,
            f,
        )
        .await?;

    Ok(())
}
