use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

use axum::{
    body::Body,
    extract::connect_info::ConnectInfo,
    http::{HeaderMap, Request, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::middleware::ip::{real_ip, RealIp};

#[tokio::test]
async fn test_real_ip_with_cf_connecting_ip() {
    // Arrange
    let connect_info = ConnectInfo(SocketAddr::from_str("192.168.1.1:1234").unwrap());
    let mut headers = HeaderMap::new();
    headers.insert("cf-connecting-ip", "203.0.113.1".parse().unwrap());
    
    let request = Request::builder().body(Body::empty()).unwrap();
    
    // Act
    let response = real_ip(
        connect_info, 
        headers, 
        request, 
        Next::new(|req| async move {
            // Assert
            let real_ip = req.extensions().get::<RealIp>().unwrap();
            assert_eq!(real_ip.0, IpAddr::from_str("203.0.113.1").unwrap());
            
            Ok::<_, std::convert::Infallible>(Response::new(Body::empty()))
        }),
    ).await;
    
    // Verify response was successful
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_real_ip_with_x_real_ip() {
    // Arrange
    let connect_info = ConnectInfo(SocketAddr::from_str("192.168.1.1:1234").unwrap());
    let mut headers = HeaderMap::new();
    headers.insert("x-real-ip", "203.0.113.2".parse().unwrap());
    
    let request = Request::builder().body(Body::empty()).unwrap();
    
    // Act
    let response = real_ip(
        connect_info, 
        headers, 
        request, 
        Next::new(|req| async move {
            // Assert
            let real_ip = req.extensions().get::<RealIp>().unwrap();
            assert_eq!(real_ip.0, IpAddr::from_str("203.0.113.2").unwrap());
            
            Ok::<_, std::convert::Infallible>(Response::new(Body::empty()))
        }),
    ).await;
    
    // Verify response was successful
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_real_ip_with_x_forwarded_for() {
    // Arrange
    let connect_info = ConnectInfo(SocketAddr::from_str("192.168.1.1:1234").unwrap());
    let mut headers = HeaderMap::new();
    headers.insert("x-forwarded-for", "203.0.113.3, 10.0.0.1".parse().unwrap());
    
    let request = Request::builder().body(Body::empty()).unwrap();
    
    // Act
    let response = real_ip(
        connect_info, 
        headers, 
        request, 
        Next::new(|req| async move {
            // Assert
            let real_ip = req.extensions().get::<RealIp>().unwrap();
            assert_eq!(real_ip.0, IpAddr::from_str("203.0.113.3").unwrap());
            
            Ok::<_, std::convert::Infallible>(Response::new(Body::empty()))
        }),
    ).await;
    
    // Verify response was successful
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_real_ip_no_headers_uses_socket_addr() {
    // Arrange
    let connect_info = ConnectInfo(SocketAddr::from_str("192.168.1.1:1234").unwrap());
    let headers = HeaderMap::new();
    
    let request = Request::builder().body(Body::empty()).unwrap();
    
    // Act
    let response = real_ip(
        connect_info, 
        headers, 
        request, 
        Next::new(|req| async move {
            // Assert
            let real_ip = req.extensions().get::<RealIp>().unwrap();
            assert_eq!(real_ip.0, IpAddr::from_str("192.168.1.1").unwrap());
            
            Ok::<_, std::convert::Infallible>(Response::new(Body::empty()))
        }),
    ).await;
    
    // Verify response was successful
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_real_ip_header_priority() {
    // Arrange - should prefer cf-connecting-ip over others
    let connect_info = ConnectInfo(SocketAddr::from_str("192.168.1.1:1234").unwrap());
    let mut headers = HeaderMap::new();
    headers.insert("cf-connecting-ip", "203.0.113.1".parse().unwrap());
    headers.insert("x-real-ip", "203.0.113.2".parse().unwrap());
    headers.insert("x-forwarded-for", "203.0.113.3".parse().unwrap());
    
    let request = Request::builder().body(Body::empty()).unwrap();
    
    // Act
    let response = real_ip(
        connect_info, 
        headers, 
        request, 
        Next::new(|req| async move {
            // Assert
            let real_ip = req.extensions().get::<RealIp>().unwrap();
            assert_eq!(real_ip.0, IpAddr::from_str("203.0.113.1").unwrap());
            
            Ok::<_, std::convert::Infallible>(Response::new(Body::empty()))
        }),
    ).await;
    
    // Verify response was successful
    assert_eq!(response.status(), StatusCode::OK);
} 