mod cast;
pub mod wire;

use crate::cast::MyCaster;
use server::app;

#[actix_web::main]
async fn main() {
    app::main::<MyCaster>().await;
}
