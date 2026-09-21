mod wire;

use crate::wire::MyCaster;
use server::app;

#[actix_web::main]
async fn main() {
    app::main::<MyCaster>().await;
}
