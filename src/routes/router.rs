use axum::{middleware::from_fn, routing::get, Router};
use tower_http::trace::TraceLayer;

use crate::{
    args::Args,
    controllers::{health::health_check, websocket::websocket_handler},
    middleware::real_ip::real_ip,
    models::state::StateType,
};

pub fn create_router(state: StateType, args: Args) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/", get(websocket_handler))
        .layer(from_fn(real_ip))
        .layer(TraceLayer::new_for_http())
        .with_state((state, args))
}
