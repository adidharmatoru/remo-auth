use std::net::{IpAddr, SocketAddr};

use axum::{
    body::Body, extract::connect_info::ConnectInfo, http::Request, middleware, response::Response,
    routing::get, Router,
};
use tower::ServiceExt;

use crate::middleware::ip::RealIp;

async fn test_handler(req: Request<Body>) -> Response {
    let mut response = Response::new(Body::empty());
    if let Some(real_ip) = req.extensions().get::<RealIp>().cloned() {
        response.extensions_mut().insert(real_ip);
    }
    response
}

#[tokio::test]
async fn test_real_ip_from_cf_connecting_ip() {
    let socket_addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    let app = Router::new()
        .route("/", get(test_handler))
        .route_layer(middleware::from_fn(crate::middleware::ip::real_ip));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/")
                .header("cf-connecting-ip", "1.1.1.1")
                .extension(ConnectInfo(socket_addr))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success());
    let extensions = response.extensions();
    let real_ip = extensions.get::<RealIp>().unwrap();
    assert_eq!(
        real_ip.0,
        "1.1.1.1".parse::<IpAddr>().unwrap(),
        "Should use IP from cf-connecting-ip header"
    );
}

#[tokio::test]
async fn test_real_ip_from_x_real_ip() {
    let socket_addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    let app = Router::new()
        .route("/", get(test_handler))
        .route_layer(middleware::from_fn(crate::middleware::ip::real_ip));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/")
                .header("x-real-ip", "2.2.2.2")
                .extension(ConnectInfo(socket_addr))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success());
    let extensions = response.extensions();
    let real_ip = extensions.get::<RealIp>().unwrap();
    assert_eq!(
        real_ip.0,
        "2.2.2.2".parse::<IpAddr>().unwrap(),
        "Should use IP from x-real-ip header"
    );
}

#[tokio::test]
async fn test_real_ip_from_x_forwarded_for() {
    let socket_addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    let app = Router::new()
        .route("/", get(test_handler))
        .route_layer(middleware::from_fn(crate::middleware::ip::real_ip));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/")
                .header("x-forwarded-for", "3.3.3.3, 10.0.0.1")
                .extension(ConnectInfo(socket_addr))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success());
    let extensions = response.extensions();
    let real_ip = extensions.get::<RealIp>().unwrap();
    assert_eq!(
        real_ip.0,
        "3.3.3.3".parse::<IpAddr>().unwrap(),
        "Should use first IP from x-forwarded-for header"
    );
}

#[tokio::test]
async fn test_real_ip_fallback_to_socket_addr() {
    let socket_addr = SocketAddr::from(([192, 168, 1, 1], 8080));

    let app = Router::new()
        .route("/", get(test_handler))
        .route_layer(middleware::from_fn(crate::middleware::ip::real_ip));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/")
                .extension(ConnectInfo(socket_addr))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success());
    let extensions = response.extensions();
    let real_ip = extensions.get::<RealIp>().unwrap();
    assert_eq!(
        real_ip.0,
        socket_addr.ip(),
        "Should fallback to socket address IP when no headers present"
    );
}
