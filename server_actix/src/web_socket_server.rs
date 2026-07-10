use actix_ws::{AggregatedMessage, AggregatedMessageStream, Session};
use futures_util::StreamExt;
use my_core::{
    server::server_methods::{self, WSServer},
    utility::{traits, utils::LogError},
};

pub(crate) struct S {
    session: Session,
    stream: AggregatedMessageStream,
}

impl S {
    pub(crate) fn new(session: Session, stream: AggregatedMessageStream) -> Self {
        Self { session, stream }
    }
}

impl WSServer for S {
    async fn send_bin(&mut self, bin: Vec<u8>) -> Result<(), traits::DynamicError> {
        self.session.binary(bin).await.log()?;
        Ok(())
    }

    async fn receive(&mut self) -> Result<server_methods::WSMessage, traits::DynamicError> {
        match self.stream.next().await {
            Some(msg) => match msg.log()? {
                AggregatedMessage::Binary(data) => {
                    Ok(server_methods::WSMessage::Binary(data.to_vec()))
                }
                AggregatedMessage::Text(_) => Err("we dont use text".into()),
                AggregatedMessage::Ping(_data) => {
                    todo!()
                }
                AggregatedMessage::Pong(_data) => {
                    todo!()
                }
                AggregatedMessage::Close(_) => Ok(server_methods::WSMessage::Close),
            },
            None => Err(dbg!("WebSocket connection closed").into()),
        }
    }

    async fn close(self) -> Result<(), traits::DynamicError> {
        self.session.clone().close(None).await.log()?;
        Ok(())
    }
}
