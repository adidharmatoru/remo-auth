use crate::{
    models::{rtc::SignallerMessage, state::State},
    services::websocket::handle_message,
};

#[tokio::test]
async fn test_handle_message_start() {
    let state = State::new();
    let (tx, _rx) = futures_channel::mpsc::unbounded();
    let socket_addr = std::net::SocketAddr::from(([127, 0, 0, 1], 8080));

    let mut locked_state = state.lock().await;

    let msg = SignallerMessage::Start {
        room: "test_room".to_string(),
        name: "test_name".to_string(),
        os: "test_os".to_string(),
        version: "1.0".to_string(),
        control: true,
    };

    let result = handle_message(
        &mut locked_state,
        &tx,
        &serde_json::to_string(&msg).unwrap(),
        socket_addr,
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(locked_state.sessions.len(), 1);
}

#[tokio::test]
async fn test_handle_message_join() {
    let state = State::new();
    let (tx, _rx) = futures_channel::mpsc::unbounded();
    let socket_addr = std::net::SocketAddr::from(([127, 0, 0, 1], 8080));

    let mut locked_state = state.lock().await;

    // First create a room
    let start_msg = SignallerMessage::Start {
        room: "test_room".to_string(),
        name: "test_name".to_string(),
        os: "test_os".to_string(),
        version: "1.0".to_string(),
        control: true,
    };

    handle_message(
        &mut locked_state,
        &tx,
        &serde_json::to_string(&start_msg).unwrap(),
        socket_addr,
    )
    .await
    .unwrap();

    // Then try to join
    let join_msg = SignallerMessage::Join {
        from: "viewer1".to_string(),
        room: "test_room".to_string(),
    };

    let result = handle_message(
        &mut locked_state,
        &tx,
        &serde_json::to_string(&join_msg).unwrap(),
        socket_addr,
    )
    .await;

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
