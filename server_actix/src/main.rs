use actix_cors::Cors;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use adapters::prelude::*;
use db_client_cockroach::prelude::*;
use my_core::prelude::*;
use std::{fs::File, io::BufReader};

mod web_socket_server {
    use actix_ws::{AggregatedMessage, AggregatedMessageStream, Session};
    use futures_util::StreamExt;
    use my_core::prelude::*;

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
        async fn send_bin(&mut self, bin: Vec<u8>) -> Result<(), DynamicError> {
            self.session.binary(bin).await?;
            Ok(())
        }

        async fn receive(&mut self) -> Result<server_methods::WSMessage, DynamicError> {
            match self.stream.next().await {
                Some(msg) => match msg? {
                    AggregatedMessage::Binary(data) => {
                        return Ok(server_methods::WSMessage::Binary(data.to_vec()));
                    }
                    AggregatedMessage::Text(_) => {
                        return Err("we dont use text".into());
                    }
                    AggregatedMessage::Ping(data) => {
                        todo!()
                    }
                    AggregatedMessage::Pong(data) => {
                        todo!()
                    }
                    AggregatedMessage::Close(_) => {
                        return Ok(server_methods::WSMessage::Close);
                    }
                },
                None => {
                    return Err(dbg!("WebSocket connection closed").into());
                }
            }
        }

        async fn close(self) -> Result<(), DynamicError> {
            self.session.clone().close(None).await?;
            Ok(())
        }
    }
}

type GG = server_methods::ServerMethods<
    db::S,
    db_client::S,
    jwt::m::S,
    authentication::m::S,
    functions::m::S,
    row_id::m::S,
    actors::m::S,
    runtime::m::S,
    encode_decode::m::S,
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

    let state = req.app_data::<web::Data<GG>>().unwrap();
    GG::server_actor(
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
