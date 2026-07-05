use actix_ws::{AggregatedMessage, AggregatedMessageStream, Session};
use futures_util::StreamExt;
use my_core::{
    server::server_traits::{self, WSServer},
    utility::utils::{self, LogError},
};

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
    async fn send_bin(&mut self, bin: Vec<u8>) -> Result<(), utils::DynamicError> {
        self.session.binary(bin).await.log()?;
        Ok(())
    }

    async fn receive(&mut self) -> Result<server_traits::WSMessage, utils::DynamicError> {
        match self.stream.next().await {
            Some(msg) => match msg.log()? {
                AggregatedMessage::Binary(data) => {
                    Ok(server_traits::WSMessage::Binary(data.to_vec()))
                }
                AggregatedMessage::Text(_) => Err("we dont use text".into()),
                AggregatedMessage::Ping(data) => {
                    todo!()
                }
                AggregatedMessage::Pong(data) => {
                    todo!()
                }
                AggregatedMessage::Close(_) => Ok(server_traits::WSMessage::Close),
            },
            None => Err(dbg!("WebSocket connection closed").into()),
        }
    }

    async fn close(self) -> Result<(), utils::DynamicError> {
        self.session.clone().close(None).await.log()?;
        Ok(())
    }
}
