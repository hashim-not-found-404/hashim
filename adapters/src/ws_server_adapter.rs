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
        async fn send_bin(&self, bin: Vec<u8>) -> Result<(), DynamicError> {
            todo!()
        }

        async fn receive(&mut self) -> Result<server::WSMessage, DynamicError> {
            if let Some(msg) = self.stream.next().await {
                match msg {
                    Ok(AggregatedMessage::Binary(data)) => {
                        self.session.binary(data).await.unwrap();
                    }
                    Ok(AggregatedMessage::Text(text)) => {
                        println!("Received text message: {}", text);
                        // Echo back
                        if let Err(e) = self.session.text(text).await {
                            eprintln!("Error responding: {}", e);
                        }
                    }
                    Ok(AggregatedMessage::Ping(data)) => {
                        // Auto-respond to pings
                        if let Err(e) = self.session.pong(&data).await {
                            eprintln!("Error sending pong: {}", e);
                        }
                    }
                    Ok(AggregatedMessage::Pong(data)) => {
                        if let Err(e) = self.session.ping(&data).await {
                            eprintln!("Error sending pong: {}", e);
                        }
                    }
                    Ok(AggregatedMessage::Close(reason)) => {
                        println!("Client requested close: {:?}", reason);
                        if let Err(e) = self.session.clone().close(None).await {
                            eprintln!("Error closing session: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("WebSocket error: {}", e);
                    }
                }
            }
            println!("WebSocket connection closed");
            todo!()
        }

        async fn close(&self) -> Result<(), DynamicError> {
            todo!()
        }
    }
}
