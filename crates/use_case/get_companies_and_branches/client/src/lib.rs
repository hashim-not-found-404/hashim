use std::pin::Pin;
use std::sync::Arc;
use use_case_get_companies_and_branches_domain::Input;
use use_case_get_companies_and_branches_domain::ReadInput;
use use_case_get_companies_and_branches_domain::ReadOutput;
use use_case_prelude_client::*;

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
