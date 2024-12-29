use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::models::state::RoomInfo;

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct IceServer {
    pub url: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SignallerMessage {
    Offer {
        from: String,
        to: String,
    },
    Answer {
        from: String,
        to: String,
    },
    Ice {
        from: String,
        to: String,
    },
    Join {
        from: String,
        room: String,
    },
    JoinDeclined {
        to: String,
        reason: String,
    },
    Start {
        room: String,
        name: String,
        os: String,
        version: String,
        control: bool,
    },
    StartResponse {
        room: String,
    },
    Leave {
        from: String,
    },
    ServerClosed {
        to: String,
        room: String,
    },
    KeepAlive {},
    IceServers {},
    IceServersResponse {
        ice_servers: Vec<IceServer>,
    },
    GetRoomList {
        os: Option<String>,
        name: Option<String>,
        version: Option<String>,
        server: Option<String>,
        sort: Option<String>,
        control: Option<bool>,
        page: Option<usize>,
        per_page: Option<usize>,
    },
    RoomListResponse {
        rooms: HashMap<String, RoomInfo>,
        total_count: usize,
        page: Option<usize>,
        per_page: Option<usize>,
    },
    SubscribeRoomUpdates {},
    UnsubscribeRoomUpdates {},
    NewRoomNotification {
        room: String,
    },
}
