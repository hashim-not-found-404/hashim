use crate::web_socket_server;
use actix_web::App;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::HttpServer;
use actix_web::web;
use actix_web::web::Data;
use actix_web::web::Payload;
use adapters::actors;
use adapters::authentication;
use adapters::encode_decode;
use adapters::functions;
use adapters::jwt;
use adapters::random_number;
use adapters::row_id;
use adapters::runtime;
use adapters::time;
use db_client_cockroach::db_bundle;
use db_client_cockroach::utility::db;
use my_core::domain::utility::types::HOST;
use my_core::domain::utility::types::PORT;
use my_core::server::server_methods;

type ServerMethodsType = server_methods::ServerMethods<actors::target::S, jwt::target::S, db::S>;

pub(crate) async fn main() {
    println!("started server");
    let actions = Data::new(ServerMethodsType::new::<runtime::target::S>().await);

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
    .bind((HOST, PORT))
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

    let stream = stream.aggregate_continuations().max_continuation_size(2_usize.pow(16));

    let session = web_socket_server::S::new(session, stream);
    let state = req.app_data::<Data<ServerMethodsType>>().unwrap();
    state
        .clone()
        .into_inner()
        .server_actor::<runtime::target::S, web_socket_server::S, random_number::target::S,encode_decode::target::S,row_id::target::S,        time::target::S,functions::target::S,authentication::target::S,db_bundle::S>(
            session,
        );

    response
}

#[allow(dead_code)]
fn get_tls_config() -> rustls::ServerConfig {
    rustls::crypto::aws_lc_rs::default_provider().install_default().unwrap();

    const CERT_PEM: &[u8] = include_bytes!("../../privet/cert.pem");
    const KEY_PEM: &[u8] = include_bytes!("../../privet/key.pem");

    use std::io::BufReader;

    let mut certs_file = BufReader::new(CERT_PEM);
    let mut key_file = BufReader::new(KEY_PEM);

    // load TLS certs and key
    // to create a self-signed temporary cert for testing:
    // `openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj '/CN=localhost'`
    let tls_certs = rustls_pemfile::certs(&mut certs_file).collect::<Result<_, _>>().unwrap();
    let tls_key = rustls_pemfile::pkcs8_private_keys(&mut key_file).next().unwrap().unwrap();

    // set up TLS config options
    rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(tls_certs, rustls::pki_types::PrivateKeyDer::Pkcs8(tls_key))
        .unwrap()
}
