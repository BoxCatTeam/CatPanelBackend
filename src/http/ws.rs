use axum::extract::WebSocketUpgrade;
use axum::extract::ws::WebSocket;
use axum::response::IntoResponse;

pub async fn ws_route(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle)
}

async fn handle(mut ws: WebSocket) {

}
