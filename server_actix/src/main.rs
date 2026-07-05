pub mod my_types;
pub mod web_socket_server;

use actix_web::{
    App, HttpRequest, HttpResponse, HttpServer,
    web::{self, Data, Payload},
};
use my_core::{accounting_domain::types, server::server_methods};

type ServerMethodsType = server_methods::ServerMethods<my_types::S>;

#[actix_web::main]
async fn main() {
    println!("started server");
    let actions = Data::new(ServerMethodsType::new().await);

    HttpServer::new(move || {
        let cors = actix_cors::Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(actions.clone())
            .route("/ws", web::get().to(ws_handler))
    })
    // .bind_rustls_0_23((HOST, PORT), get_tls_config())
    .bind((types::HOST, types::PORT))
    .unwrap()
    .run()
    .await
    .unwrap()
}

async fn ws_handler(req: HttpRequest, stream: Payload) -> HttpResponse {
    let (response, session, stream) = match actix_ws::handle(&req, stream) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Error upgrading to WebSocket: {}", e);
            return HttpResponse::BadRequest().finish();
        }
    };

    let stream = stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(16));

    let session = web_socket_server::S::new(session, stream);
    let state = req.app_data::<Data<ServerMethodsType>>().unwrap();
    state.clone().into_inner().server_actor(session);

    response
}

fn get_tls_config() -> rustls::ServerConfig {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

    const CERT_PEM: &[u8] = include_bytes!("../../privet/cert.pem");
    const KEY_PEM: &[u8] = include_bytes!("../../privet/key.pem");

    use std::io::BufReader;

    let mut certs_file = BufReader::new(CERT_PEM);
    let mut key_file = BufReader::new(KEY_PEM);

    // load TLS certs and key
    // to create a self-signed temporary cert for testing:
    // `openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj '/CN=localhost'`
    let tls_certs = rustls_pemfile::certs(&mut certs_file)
        .collect::<Result<_, _>>()
        .unwrap();
    let tls_key = rustls_pemfile::pkcs8_private_keys(&mut key_file)
        .next()
        .unwrap()
        .unwrap();

    // set up TLS config options
    rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(tls_certs, rustls::pki_types::PrivateKeyDer::Pkcs8(tls_key))
        .unwrap()
}
