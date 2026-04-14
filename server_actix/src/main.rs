use actix_cors::Cors;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, post, rt, web};
use actix_ws::AggregatedMessage;
use db_client_cockroach::{CockroachClient, CockroachDB};
use futures_util::StreamExt as _;
use impls_for_wasm::a1::RowId;
use my_core::impls::StateLessCheck;
use my_core::{impls::StateFullCheck, request_response::*, traits::BackendRouts};
use server_logic::{authentication::HashedPassword, functions::Functions, jwt::Key};
use std::fs::File;
use std::io::BufReader;

type GG = StateLessCheck<
    StateFullCheck<CockroachDB, CockroachClient, Key, HashedPassword, Functions, RowId>,
    RowId,
>;

#[actix_web::main]
async fn main() {
    println!("started server");

    let actions: GG =
        StateLessCheck::new(StateFullCheck::new(CockroachDB::default(), Key::default()));

    let actions = web::Data::new(actions);

    HttpServer::new(move || {
        let cors = Cors::default()
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

async fn ws_handler(req: HttpRequest, stream: web::Payload) -> HttpResponse {
    let (res, mut session, stream) = actix_ws::handle(&req, stream).unwrap();

    let mut stream = stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(16));

    rt::spawn(async move {
        while let Some(Ok(msg)) = stream.next().await {
            if let AggregatedMessage::Text(text) = msg {
                if let Ok(input) = serde_json::from_str::<business_layer::Input>(&text) {
                    // TODO if sign in or up check jwt
                    let _ = session.text(serde_json::to_string(&resp).unwrap()).await;
                } else {
                    let _ = session.text(format!("Error: {}", e)).await;
                }
            }
        }
    });

    res
}

fn get_tls_config() -> rustls::ServerConfig {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

    let mut certs_file = BufReader::new(File::open("privet/cert.pem").unwrap());
    let mut key_file = BufReader::new(File::open("privet/key.pem").unwrap());

    // load TLS certs and key
    // to create a self-signed temporary cert for testing:
    // `openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj '/CN=localhost'`
    let tls_certs = rustls_pemfile::certs(&mut certs_file)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let tls_key = rustls_pemfile::pkcs8_private_keys(&mut key_file)
        .next()
        .unwrap()
        .unwrap();

    // set up TLS config options
    let tls_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(tls_certs, rustls::pki_types::PrivateKeyDer::Pkcs8(tls_key))
        .unwrap();

    return tls_config;
}
