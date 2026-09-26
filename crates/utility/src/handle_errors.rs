use anyhow::Error;
use anyhow::Result;
use infrastructure::actors::MpscSender;
use infrastructure::actors::Sender;

pub async fn handle_error<T, F>(mut sender_to_error: MpscSender<Error>, mut f: F) -> T
where
    F: AsyncFnMut() -> Result<T>,
{
    loop {
        let r = f().await;
        match r {
            Ok(ok) => return ok,
            Err(err) => {
                let _ = sender_to_error.send(err).await;
            }
        }
    }
}

pub async fn handle_error_one_time<F>(mut sender_to_error: MpscSender<Error>, mut f: F)
where
    F: AsyncFnOnce() -> Result<()>,
{
    let r = f().await;
    match r {
        Ok(_) => return,
        Err(err) => {
            let _ = sender_to_error.send(err).await;
        }
    }
}
