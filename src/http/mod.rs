use axum::routing::get;
use axum::Router;
use axum_sessions::SessionLayer;
use rand::Rng;
use tokio_graceful_shutdown::SubsystemHandle;
use tracing::info;

use crate::configure::get_config;
use crate::http::rocksdb_session_store::RocksdbStore;

mod rocksdb_session_store;
mod routes;
mod ws;
mod error;
mod model;

pub async fn start_http_server(handle: SubsystemHandle) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/", get(routes::hello_world))
        .route("/ws", get(ws::ws_route))
        .layer(SessionLayer::new(
            RocksdbStore::new()?,
            &rand::thread_rng().gen::<[u8; 64]>(),
        ));

    axum::Server::bind(&get_config().http.bind)
        .serve(app.into_make_service())
        .with_graceful_shutdown(async move {
            handle.on_shutdown_requested().await;
            info!("http server is shutting down...");
        })
        .await
        .map_err(Into::into)
}
