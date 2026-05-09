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
>;

#[actix_web::main]
async fn main() {
    println!("started server");

    let actions = GG::new(CockroachDB::default(), jwt::m::S::default());

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

    rt::spawn(async move {
        // Keep the connection alive
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(AggregatedMessage::Binary(data)) => {
                    let recived_msg =
                        encode_decode::m::S::decode::<web_socket::MessageType>(&data.to_vec())
                            .unwrap();
                    let state = req.app_data::<web::Data<GG>>().unwrap();

                    let msg_to_send = match recived_msg {
                        web_socket::MessageType::TwoWay { id, path, payload } => {
                            let payload = match path.as_str() {
                                sign_up::PATH => {
                                    let input =
                                        encode_decode::m::S::decode::<sign_up::Input>(&payload);

                                    let result = match input {
                                        Ok(input) => match state.sign_up(&input).await {
                                            Ok(o) => Ok(o),
                                            Err(_) => Err(HashimError::InternalServerError),
                                        },
                                        Err(_) => Err(HashimError::DecodingErrorAtServer),
                                    };

                                    encode_decode::m::S::encode(&result)
                                }
                                sign_in::PATH => {
                                    let input =
                                        encode_decode::m::S::decode::<sign_in::Input>(&payload);

                                    let result = match input {
                                        Ok(input) => match state.sign_in(&input).await {
                                            Ok(o) => Ok(o),
                                            Err(_) => Err(HashimError::InternalServerError),
                                        },
                                        Err(_) => Err(HashimError::DecodingErrorAtServer),
                                    };

                                    encode_decode::m::S::encode(&result)
                                }
                                _ => todo!(),
                            };

                            let msg_to_send = web_socket::MessageType::TwoWay { id, path, payload };

                            msg_to_send
                        }
                        web_socket::MessageType::OneWay { path, payload } => todo!(),
                    };

                    session
                        .binary(encode_decode::m::S::encode(&msg_to_send))
                        .await
                        .unwrap();
                }
                Ok(AggregatedMessage::Text(text)) => {
                    println!("Received text message: {}", text);
                    // Echo back
                    if let Err(e) = session.text(text).await {
                        eprintln!("Error responding: {}", e);
                        break;
                    }
                }
                Ok(AggregatedMessage::Ping(data)) => {
                    // Auto-respond to pings
                    if let Err(e) = session.pong(&data).await {
                        eprintln!("Error sending pong: {}", e);
                        break;
                    }
                }
                Ok(AggregatedMessage::Pong(data)) => {
                    if let Err(e) = session.ping(&data).await {
                        eprintln!("Error sending pong: {}", e);
                        break;
                    }
                }
                Ok(AggregatedMessage::Close(reason)) => {
                    println!("Client requested close: {:?}", reason);
                    if let Err(e) = session.close(None).await {
                        eprintln!("Error closing session: {}", e);
                    }
                    break;
                }
                Err(e) => {
                    eprintln!("WebSocket error: {}", e);
                    break;
                }
            }
        }
        println!("WebSocket connection closed");
    });

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
