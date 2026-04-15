use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::main]
async fn main() {
    let (mut ws_stream, _) = connect_async("ws://127.0.0.1:8080/ws").await.unwrap();

    println!("Connected to server!\n");

    ws_stream
        .send(Message::Text("msg".to_string().into()))
        .await
        .unwrap();

    if let Some(Ok(reply)) = ws_stream.next().await {
        println!("📥 Got: {}\n", reply);
    }
}
