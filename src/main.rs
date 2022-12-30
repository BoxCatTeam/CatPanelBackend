use tracing::info;

use crate::log::init_tracing_subscriber;

mod log;

#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing_subscriber();

    info!("Hello, world!");
    Ok(())
}
