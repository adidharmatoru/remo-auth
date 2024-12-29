use futures_channel::mpsc::unbounded;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use crate::models::state::State;

#[tokio::test]
async fn test_state_new() {
    let state = State::new();
    let locked_state = state.lock().await;
    assert!(locked_state.sessions.is_empty());
    assert!(locked_state.server_socket_addr_to_room.is_empty());
    assert!(locked_state.peers.is_empty());
    assert!(locked_state.room_update_subscribers.is_empty());
}

#[tokio::test]
async fn test_add_server() {
    let state = State::new();
    let (tx, _rx) = unbounded();
    let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);

    let mut locked_state = state.lock().await;
    let result = locked_state.add_server(
        "test_room".to_string(),
        "test_name".to_string(),
        "test_os".to_string(),
        "1.0".to_string(),
        true,
        tx,
        socket_addr,
    );

    assert!(result.is_ok());
    assert_eq!(locked_state.sessions.len(), 1);
    assert_eq!(locked_state.server_socket_addr_to_room.len(), 1);
    assert_eq!(locked_state.peers.len(), 1);
}

#[tokio::test]
async fn test_add_viewer() {
    let state = State::new();
    let (server_tx, _rx) = unbounded();
    let (viewer_tx, _rx) = unbounded();
    let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);

    let mut locked_state = state.lock().await;

    // First add a server
    locked_state
        .add_server(
            "test_room".to_string(),
            "test_name".to_string(),
            "test_os".to_string(),
            "1.0".to_string(),
            true,
            server_tx,
            socket_addr,
        )
        .unwrap();

    // Then add a viewer
    let result = locked_state.add_viewer("viewer1".to_string(), "test_room".to_string(), viewer_tx);

    assert!(result.is_ok());
    assert_eq!(
        locked_state
            .sessions
            .get("test_room")
            .unwrap()
            .viewers
            .len(),
        1
    );
}

#[tokio::test]
async fn test_leave_session() {
    let state = State::new();
    let (server_tx, _rx) = unbounded();
    let (viewer_tx, _rx) = unbounded();
    let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);

    let mut locked_state = state.lock().await;

    // Setup test environment
    locked_state
        .add_server(
            "test_room".to_string(),
            "test_name".to_string(),
            "test_os".to_string(),
            "1.0".to_string(),
            true,
            server_tx,
            socket_addr,
        )
        .unwrap();

    locked_state
        .add_viewer("viewer1".to_string(), "test_room".to_string(), viewer_tx)
        .unwrap();

    // Test viewer leaving
    let result = locked_state.leave_session("viewer1".to_string());
    assert!(result.is_ok());
    assert_eq!(
        locked_state
            .sessions
            .get("test_room")
            .unwrap()
            .viewers
            .len(),
        0
    );

    // Test server leaving
    let result = locked_state.leave_session("test_room".to_string());
    assert!(result.is_ok());
    assert!(locked_state.sessions.is_empty());
}

#[tokio::test]
async fn test_get_available_rooms() {
    let state = State::new();
    let (tx, _rx) = unbounded();
    let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);

    let mut locked_state = state.lock().await;

    // Add test rooms
    locked_state
        .add_server(
            "room1".to_string(),
            "test1".to_string(),
            "windows".to_string(),
            "1.0".to_string(),
            true,
            tx.clone(),
            socket_addr,
        )
        .unwrap();

    let (rooms, total_count) = locked_state.get_available_rooms(
        Some("windows"),
        Some("1.0"),
        None,
        Some("test"),
        Some("desc"),
        Some(true),
        Some(1),
        Some(10),
    );

    assert_eq!(total_count, 1);
    assert_eq!(rooms.len(), 1);
    assert!(rooms.contains_key("room1"));
}
