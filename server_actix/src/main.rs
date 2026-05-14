use actix_cors::Cors;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, rt, web};
use actix_ws::AggregatedMessage;
use adapters::prelude::*;
use db_client_cockroach::{CockroachClient, CockroachDB};
use futures_util::StreamExt;
use my_core::prelude::*;
use std::{fs::File, io::BufReader};

type GG = server_methods::ServerMethods<
    CockroachDB,
    CockroachClient,
    jwt::m::S,
    authentication::m::S,
    functions::m::S,
    row_id::m::S,
    actors::m::S,
    runtime::m::S,
>;

#[actix_web::main]
async fn main() {
    println!("started server");

    let actions = web::Data::new(GG::new().await);

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
    let (response, mut session, stream) = match actix_ws::handle(&req, stream) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Error upgrading to WebSocket: {}", e);
            return HttpResponse::BadRequest().finish();
        }
    };

    let mut stream = stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(16));

    let session = ws_server_adapter::m::S::new(session, stream);

    let state = req.app_data::<web::Data<GG>>().unwrap();
    server::server_actor::<
        CockroachDB,
        CockroachClient,
        jwt::m::S,
        authentication::m::S,
        functions::m::S,
        row_id::m::S,
        encode_decode::m::S,
        runtime::m::S,
        ws_server_adapter::m::S,
        actors::m::S,
    >(
        state.clone().into_inner(),
        session,
        state.clone().into_inner().sender_to_broker.clone(),
    );

    response
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
