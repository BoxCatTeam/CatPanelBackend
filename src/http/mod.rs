use axum::Router;
use axum::routing::get;
use tokio_graceful_shutdown::SubsystemHandle;
use tracing::info;
use crate::configure::get_config;

mod routes;

pub async fn start_http_server(handle: SubsystemHandle) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/", get(routes::hello_world));

    axum::Server::bind(&get_config().http.bind)
        .serve(app.into_make_service())
        .with_graceful_shutdown(async move {
            handle.on_shutdown_requested().await;
            info!("http server is shutting down...");
        })
        .await
        .map_err(Into::into)
}
