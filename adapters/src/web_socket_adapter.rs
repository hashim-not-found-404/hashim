use crate::prelude::*;

pub struct MyClient {
    write: Mutex<SplitSink<WebSocketStream, Message>>,
    read: Mutex<SplitStream<WebSocketStream>>,
}

impl MyClient {
    pub async fn new() -> Self {
        let url = format!("ws://{}/ws", ADDRESS);

        let ws_stream = connect(url).await;

        let ws_stream = match ws_stream {
            Ok(o) => o,
            Err(e) => {
                panic!("{:?}", e)
            }
        };

        let (write, read) = ws_stream.split();

        Self {
            write: Mutex::new(write),
            read: Mutex::new(read),
        }
    }
}

impl WebSocketOp for MyClient {
    type Error = MyError;

    async fn send_bin(&self, data: Vec<u8>) -> Result<(), Self::Error> {
        self.write
            .lock()
            .unwrap()
            .send(Message::Binary(data.into()))
            .await
            .unwrap();

        Ok(())
    }

    async fn try_receive_bin(&self) -> Result<Vec<u8>, Self::Error> {
        let mut guard = self.read.lock().unwrap();

        match guard.next().await {
            Some(Ok(o)) => match o {
                Message::Text(utf8_bytes) => todo!(),
                Message::Binary(bytes) => return Ok(bytes.into()),
                Message::Close(close_frame) => todo!(),
            },
            Some(Err(o)) => return Err(MyError::Closed),
            None => todo!(),
        }
    }
}
