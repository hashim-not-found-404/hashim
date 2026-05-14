#[cfg(not(target_arch = "wasm32"))]
pub mod m {
    use crate::prelude::*;
    use actix_ws::{AggregatedMessage, AggregatedMessageStream, Session};
    use futures::StreamExt;

    pub struct S {
        session: Session,
        stream: AggregatedMessageStream,
    }

    impl S {
        pub fn new(session: Session, stream: AggregatedMessageStream) -> Self {
            Self { session, stream }
        }
    }

    impl WSServer for S {
        async fn send_bin(&mut self, bin: Vec<u8>) -> Result<(), DynamicError> {
            self.session.binary(bin).await?;
            Ok(())
        }

        async fn receive(&mut self) -> Result<server::WSMessage, DynamicError> {
            match self.stream.next().await {
                Some(msg) => match msg? {
                    AggregatedMessage::Binary(data) => {
                        return Ok(server::WSMessage::Binary(data.to_vec()));
                    }
                    AggregatedMessage::Text(_) => {
                        return Err("we dont use text".into());
                    }
                    AggregatedMessage::Ping(data) => {
                        todo!()
                    }
                    AggregatedMessage::Pong(data) => {
                        todo!()
                    }
                    AggregatedMessage::Close(reason) => {
                        return Ok(server::WSMessage::Close);
                    }
                },
                None => {
                    return Err(dbg!("WebSocket connection closed").into());
                }
            }
        }

        async fn close(self) -> Result<(), DynamicError> {
            self.session.clone().close(None).await?;
            Ok(())
        }
    }
}
