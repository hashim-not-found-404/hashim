mod wire;

use crate::wire::MyCaster;
use anyhow::Result;
use server::app;

#[actix_web::main]
async fn main() -> Result<()> {
    app::main::<MyCaster>().await?;
    Ok(())
}
