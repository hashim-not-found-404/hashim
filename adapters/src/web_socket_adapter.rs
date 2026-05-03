use crate::prelude::*;

pub struct MyClient {
    write: Mutex<SplitSink<WebSocketStream, Message>>,
    read: Mutex<SplitStream<WebSocketStream>>,
}

impl WebSocketOp for MyClient {
    async fn connect(url: &str) -> Result<Self, DynamicError> {
        let (write, read) = connect(url).await?.split();

        Ok(Self {
            write: Mutex::new(write),
            read: Mutex::new(read),
        })
    }

    async fn send_bin(&self, data: Vec<u8>) -> Result<(), DynamicError> {
        self.write
            .lock()
            .unwrap()
            .send(Message::Binary(data.into()))
            .await?;

        Ok(())
    }

    async fn try_receive_bin(&self) -> Result<Vec<u8>, DynamicError> {
        let mut guard = self.read.lock().unwrap();

        match guard.next().await {
            Some(Ok(o)) => match o {
                Message::Text(utf8_bytes) => return Err("it's text".into()),
                Message::Binary(bytes) => return Ok(bytes.into()),
                Message::Close(close_frame) => todo!("{:?}", close_frame),
            },
            Some(Err(o)) => return Err(Box::new(o)),
            None => return Err("it's None".into()),
        }
    }
}
