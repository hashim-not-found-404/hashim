pub mod app;
pub mod web_socket_server;

#[actix_web::main]
async fn main() {
    app::main().await;
}
