use std::time::Duration;

use tracing::info;

use crate::configure::init_configure;
use crate::http::start_http_server;
use crate::log::init_tracing_subscriber;

mod configure;
mod http;
mod log;

#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

async fn main2() -> anyhow::Result<()> {
    let wait_for_shutdown = init_tracing_subscriber();
    init_configure()?;

    info!("Hello, world!");

    tokio_graceful_shutdown::Toplevel::new()
        .start("http server", start_http_server)
        .catch_signals()
        .handle_shutdown_requests(Duration::from_secs(3))
        .await?;

    // 在tokio_graceful_shutdown关闭后等待
    // 因为异步写入的情况下有时候会出现日志文件写入器已经关闭而前面的日志才发送的情况
    tokio::time::sleep(Duration::from_millis(200)).await;
    wait_for_shutdown.await?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(main2())
}
