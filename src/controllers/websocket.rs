use axum::{
    extract::{ConnectInfo, State, WebSocketUpgrade},
    response::IntoResponse,
    Extension,
};
use std::net::SocketAddr;

use crate::{
    args::Args, middleware::real_ip::RealIp, models::state::StateType,
    services::websocket::handle_connection,
};

pub async fn websocket_handler(
    State((state, args)): State<(StateType, Args)>,
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Extension(RealIp(real_ip)): Extension<RealIp>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        handle_connection(args, state, socket, addr, Some(&real_ip)).await
    })
}
