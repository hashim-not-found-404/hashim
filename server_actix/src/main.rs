use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, post, web};
use db_client_cockroach::{CockroachClient, CockroachDB};
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
            .service(service_sign_up)
            .service(service_sign_in)
    })
    // .bind_rustls_0_23((HOST, PORT), get_tls_config())
    .bind((HOST, PORT))
    .unwrap()
    .run()
    .await
    .unwrap()
}

macro_rules! create_route {
    ($server_route:ident,$route:ident,$route_string:literal) => {
        #[post($route_string)]
        async fn $server_route(
            data: web::Json<business_layer::Input<$route::Input>>,
            state: web::Data<GG>,
        ) -> HttpResponse {
            let result = state.$route(data.into_inner()).await;
            return HttpResponse::Ok().json(result);
        }
    };
}

create_route!(service_sign_up, sign_up, "/sign_up");
create_route!(service_sign_in, sign_in, "/sign_in");
create_route!(service_create_company, create_company, "/create_company");
create_route!(
    service_create_company_branch,
    create_company_branch,
    "/create_company_branch"
);

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
