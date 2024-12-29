use std::net::IpAddr;

use axum::{
    body::Body,
    extract::connect_info::ConnectInfo,
    http::{HeaderMap, Request},
    middleware::Next,
    response::Response,
};

pub async fn real_ip(
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    // Try to get real IP from X-Forwarded-For header
    let real_ip = headers
        .get("x-forwarded-for")
        .and_then(|hv| hv.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse::<IpAddr>().ok())
        .unwrap_or(addr.ip());

    // Store the real IP in request extensions
    request.extensions_mut().insert(RealIp(real_ip));

    next.run(request).await
}

// Wrapper type for the real IP address
#[derive(Clone, Copy)]
pub struct RealIp(pub IpAddr);
