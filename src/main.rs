use crate::configure::init_configure;
use tracing::info;

use crate::log::init_tracing_subscriber;

mod configure;
mod log;

#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing_subscriber();
    init_configure()?;

    info!("Hello, world!");

    Ok(())
}
