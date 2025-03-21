use futures_channel::mpsc::unbounded;
use std::env;
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
            "Windows".to_string(), // Note: capitalized for case-insensitive test
            "1.0".to_string(),
            true,
            tx.clone(),
            socket_addr,
        )
        .unwrap();

    // Add a second room with mixed case
    let socket_addr2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081);
    locked_state
        .add_server(
            "room2".to_string(),
            "TeStName".to_string(), // Mixed case for name
            "MacOS".to_string(),    // Different OS
            "1.0".to_string(),
            true,
            tx.clone(),
            socket_addr2,
        )
        .unwrap();

    // Test with exact case match
    let (rooms, total_count) = locked_state.get_available_rooms(
        Some("Windows"),
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

    // Test with case-insensitive OS filter
    let (rooms, total_count) = locked_state.get_available_rooms(
        Some("windows"), // lowercase, should still match "Windows"
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

    // Test case-insensitive name filter
    let (rooms, total_count) = locked_state.get_available_rooms(
        None,
        Some("1.0"),
        None,
        Some("testname"), // lowercase, should match "TeStName"
        Some("desc"),
        Some(true),
        Some(1),
        Some(10),
    );

    assert_eq!(total_count, 1);
    assert_eq!(rooms.len(), 1);
    assert!(rooms.contains_key("room2"));
}

#[tokio::test]
async fn test_get_ice_servers() {
    let state = State::new();
    let locked_state = state.lock().await;

    // Test with no environment variables set
    let ice_servers = locked_state.get_ice_servers("test_id".to_string()).await;
    assert_eq!(ice_servers.len(), 0);

    // Test with STUN servers
    env::set_var(
        "STUN_SERVERS",
        "stun:stun.example.com:3478,stun:stun2.example.com:3478",
    );
    let ice_servers = locked_state.get_ice_servers("test_id".to_string()).await;
    assert_eq!(ice_servers.len(), 2);
    assert_eq!(ice_servers[0].url, "stun:stun.example.com:3478");
    assert_eq!(ice_servers[1].url, "stun:stun2.example.com:3478");

    // Test with TURN servers using shared credentials
    env::set_var(
        "TURN_SERVERS",
        "turn:turn.example.com:3478,turn:turn2.example.com:3478",
    );
    env::set_var("TURN_USERNAME", "username");
    env::set_var("TURN_CREDENTIAL", "password");
    let ice_servers = locked_state.get_ice_servers("test_id".to_string()).await;
    // Should now have both STUN and TURN servers
    assert_eq!(ice_servers.len(), 4);

    // Check STUN servers still exist
    assert!(ice_servers
        .iter()
        .any(|s| s.url == "stun:stun.example.com:3478"));
    assert!(ice_servers
        .iter()
        .any(|s| s.url == "stun:stun2.example.com:3478"));

    // Check TURN servers
    let turn_servers: Vec<_> = ice_servers
        .iter()
        .filter(|s| s.url.starts_with("turn:"))
        .collect();
    assert_eq!(turn_servers.len(), 2);
    assert!(turn_servers
        .iter()
        .any(|s| s.url == "turn:turn.example.com:3478"));
    assert!(turn_servers
        .iter()
        .any(|s| s.url == "turn:turn2.example.com:3478"));
    for turn_server in turn_servers {
        assert_eq!(turn_server.username, "username");
        assert_eq!(turn_server.credential, "password");
        assert_eq!(turn_server.credential_type, "password");
    }

    // Clean up environment
    env::remove_var("STUN_SERVERS");
    env::remove_var("TURN_SERVERS");
    env::remove_var("TURN_USERNAME");
    env::remove_var("TURN_CREDENTIAL");

    // Test with TURN servers with individual credentials
    env::set_var(
        "TURN_SERVER_CONFIGS",
        "turn:turn1.example.com:3478|user1|pass1,turn:turn2.example.com:3478|user2|pass2",
    );
    let ice_servers = locked_state.get_ice_servers("test_id".to_string()).await;
    assert_eq!(ice_servers.len(), 2);

    // Check first TURN server
    let turn1 = ice_servers
        .iter()
        .find(|s| s.url == "turn:turn1.example.com:3478")
        .unwrap();
    assert_eq!(turn1.username, "user1");
    assert_eq!(turn1.credential, "pass1");
    assert_eq!(turn1.credential_type, "password");

    // Check second TURN server
    let turn2 = ice_servers
        .iter()
        .find(|s| s.url == "turn:turn2.example.com:3478")
        .unwrap();
    assert_eq!(turn2.username, "user2");
    assert_eq!(turn2.credential, "pass2");
    assert_eq!(turn2.credential_type, "password");

    env::remove_var("TURN_SERVER_CONFIGS");

    // Test combining all types of servers
    env::set_var("STUN_SERVERS", "stun:stun.example.com:3478");
    env::set_var("TURN_SERVERS", "turn:shared.example.com:3478");
    env::set_var("TURN_USERNAME", "shared_user");
    env::set_var("TURN_CREDENTIAL", "shared_pass");
    env::set_var(
        "TURN_SERVER_CONFIGS",
        "turn:individual.example.com:3478|ind_user|ind_pass",
    );

    let ice_servers = locked_state.get_ice_servers("test_id".to_string()).await;
    assert_eq!(ice_servers.len(), 3);

    // Cleanup all
    env::remove_var("STUN_SERVERS");
    env::remove_var("TURN_SERVERS");
    env::remove_var("TURN_USERNAME");
    env::remove_var("TURN_CREDENTIAL");
    env::remove_var("TURN_SERVER_CONFIGS");

    // Test with whitelist
    env::set_var("STUN_SERVERS", "stun:stun.example.com:3478");
    env::set_var("ICE_SERVER_WHITELIST", "allowed_id,another_id");
    let ice_servers = locked_state.get_ice_servers("test_id".to_string()).await;
    assert_eq!(ice_servers.len(), 0); // Not in whitelist
    let ice_servers = locked_state.get_ice_servers("allowed_id".to_string()).await;
    assert_eq!(ice_servers.len(), 1); // In whitelist
    env::remove_var("STUN_SERVERS");
    env::remove_var("ICE_SERVER_WHITELIST");
}
