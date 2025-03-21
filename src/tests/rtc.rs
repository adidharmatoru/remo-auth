use crate::models::rtc::{IceServer, SignallerMessage};

#[test]
fn test_signaller_message_serialization() {
    let msg = SignallerMessage::Start {
        room: "test_room".to_string(),
        name: "test_name".to_string(),
        os: "test_os".to_string(),
        version: "1.0".to_string(),
        control: true,
    };

    let serialized = serde_json::to_string(&msg).unwrap();
    let deserialized: SignallerMessage = serde_json::from_str(&serialized).unwrap();

    match deserialized {
        SignallerMessage::Start {
            room,
            name,
            os,
            version,
            control,
        } => {
            assert_eq!(room, "test_room");
            assert_eq!(name, "test_name");
            assert_eq!(os, "test_os");
            assert_eq!(version, "1.0");
            assert!(control);
        }
        _ => panic!("Deserialized to wrong variant"),
    }
}

#[test]
fn test_ice_server_default() {
    let ice_server = IceServer::default();
    assert_eq!(ice_server.url, "");
    assert_eq!(ice_server.username, "");
    assert_eq!(ice_server.credential, "");
    assert_eq!(ice_server.credential_type, "");
}
