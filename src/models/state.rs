use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::ws::Message;
use failure::{format_err, Error};
use futures_channel::mpsc::UnboundedSender;
use log::info;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::models::peer::{Peer, PeerType};
use crate::models::rtc::{IceServer, SignallerMessage};
use crate::models::session::Session;

type Result<T> = std::result::Result<T, Error>;
type Tx = UnboundedSender<Message>;

pub struct State {
    pub sessions: HashMap<String, Session>,
    pub server_socket_addr_to_room: HashMap<SocketAddr, String>,
    pub peers: HashMap<String, Peer>,
    pub room_update_subscribers: HashSet<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RoomInfo {
    pub server: String,
    pub viewer_count: usize,
    pub viewers: Vec<String>,
    pub os: String,
    pub version: String,
    pub name: String,
    pub control: bool,
}

pub type StateType = Arc<Mutex<State>>;

impl State {
    pub fn new() -> StateType {
        Arc::new(Mutex::new(State {
            sessions: Default::default(),
            server_socket_addr_to_room: Default::default(),
            peers: Default::default(),
            room_update_subscribers: Default::default(),
        }))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_server(
        &mut self,
        room: String,
        name: String,
        os: String,
        version: String,
        control: bool,
        sender: Tx,
        socket_addr: SocketAddr,
    ) -> Result<()> {
        if self.sessions.contains_key(&room) {
            return Err(format_err!("Device is currently online"));
        }
        self.sessions.insert(
            room.clone(),
            Session::new(room.clone(), socket_addr, name, os, version, control),
        );
        self.server_socket_addr_to_room
            .insert(socket_addr, room.clone());
        self.peers.insert(
            room.clone(),
            Peer {
                room,
                sender,
                peer_type: PeerType::Server {},
            },
        );
        Ok(())
    }

    pub fn add_viewer(&mut self, id: String, room: String, sender: Tx) -> Result<()> {
        if !self.sessions.contains_key(&room) {
            return Err(format_err!("Device is offline"));
        }
        self.sessions
            .get_mut(&room)
            .unwrap()
            .viewers
            .insert(id.clone());
        self.peers.insert(
            id,
            Peer {
                room,
                sender,
                peer_type: PeerType::Viewer {},
            },
        );
        Ok(())
    }

    fn remove_session(&mut self, room: &String) {
        info!("Removing session {}", room);
        let session = self.sessions.remove(room).unwrap();
        self.server_socket_addr_to_room
            .remove(&session.server_socket_addr);
        let duration_sec = session.start_time.elapsed().unwrap().as_secs_f64();
        info!("Ended session with duration: {}s", duration_sec);
        for viewer in session.viewers {
            let _ = self.peers[&viewer].sender.unbounded_send(Message::Text(
                serde_json::to_string(&SignallerMessage::ServerClosed {
                    to: viewer.clone(),
                    room: room.clone(),
                })
                .unwrap(),
            ));
            self.peers.remove(&viewer);
        }
        self.peers.remove(&session.server);
    }

    /// Leave a session. id is the id of the viewer or the server.
    pub fn leave_session(&mut self, id: String) -> Result<()> {
        if self.sessions.contains_key(&id) {
            // id is host. remove session
            self.remove_session(&id);
        } else {
            let peer = self
                .peers
                .get(&id)
                .ok_or_else(|| format_err!("Peer does not exist"))?;
            let session = self.sessions.get_mut(&peer.room).unwrap();
            session.viewers.remove(&id);
            self.peers.remove(&id);
        }
        Ok(())
    }

    pub fn on_disconnect(&mut self, socket_addr: &SocketAddr) {
        if let Some(room) = self.server_socket_addr_to_room.get(socket_addr) {
            self.remove_session(&room.clone());
        }
    }

    pub async fn get_ice_servers(&self, id: String) -> Vec<IceServer> {
        // Check if the peer ID is in the whitelist
        if let Ok(whitelist) = std::env::var("ICE_SERVER_WHITELIST") {
            let whitelisted_ids: Vec<&str> = whitelist.split(',').map(|s| s.trim()).collect();
            if !whitelisted_ids.is_empty() && !whitelisted_ids.contains(&id.as_str()) {
                return vec![];
            }
        }

        let mut servers = Vec::new();

        // Get STUN servers from environment variables
        if let Ok(stun_servers) = std::env::var("STUN_SERVERS") {
            let stun_server_list: Vec<IceServer> = stun_servers
                .split(',')
                .map(|url| IceServer {
                    urls: vec![url.trim().to_string()],
                    ..Default::default()
                })
                .collect();

            servers.extend(stun_server_list);
        }

        // Get TURN servers with individual credentials
        if let Ok(turn_server_config) = std::env::var("TURN_SERVER_CONFIGS") {
            // Format: url|username|credential,url2|username2|credential2
            for config in turn_server_config.split(',') {
                let parts: Vec<&str> = config.split('|').collect();
                if parts.len() >= 3 {
                    servers.push(IceServer {
                        urls: vec![parts[0].trim().to_string()],
                        username: parts[1].trim().to_string(),
                        credential: parts[2].trim().to_string(),
                        credential_type: "password".to_string(),
                    });
                }
            }
        }

        // Legacy support for common credential TURN servers
        if let (Ok(turn_urls), Ok(turn_username), Ok(turn_credential)) = (
            std::env::var("TURN_SERVERS"),
            std::env::var("TURN_USERNAME"),
            std::env::var("TURN_CREDENTIAL"),
        ) {
            let turn_server_list: Vec<IceServer> = turn_urls
                .split(',')
                .map(|url| IceServer {
                    urls: vec![url.trim().to_string()],
                    username: turn_username.clone(),
                    credential: turn_credential.clone(),
                    credential_type: "password".to_string(),
                })
                .collect();

            servers.extend(turn_server_list);
        }

        servers
    }

    #[allow(clippy::too_many_arguments)]
    pub fn get_available_rooms(
        &self,
        os: Option<&str>,
        version: Option<&str>,
        server: Option<&str>,
        name: Option<&str>,
        sort: Option<&str>,
        control: Option<bool>,
        page: Option<usize>,
        per_page: Option<usize>,
    ) -> (HashMap<String, RoomInfo>, usize) {
        let mut sessions_vec: Vec<_> = self.sessions.iter().collect();

        // Sort sessions by start_time
        sessions_vec.sort_by(|a, b| {
            match sort {
                Some("asc") => a.1.start_time.cmp(&b.1.start_time), // Ascending order
                _ => b.1.start_time.cmp(&a.1.start_time),           // Default to descending order
            }
        });

        // Filter sessions based on provided criteria
        let filtered_sessions: Vec<_> = sessions_vec
            .into_iter()
            .filter(|(_, session)| {
                (os.is_none()
                    || os
                        .map(|os_val| session.os.to_lowercase() == os_val.to_lowercase())
                        .unwrap())
                    && (version.is_none()
                        || version
                            .map(|ver| session.version.to_lowercase() == ver.to_lowercase())
                            .unwrap())
                    && (server.is_none()
                        || server
                            .map(|srv| session.server.to_lowercase() == srv.to_lowercase())
                            .unwrap())
                    && (name.is_none()
                        || name
                            .map(|name_val| {
                                session
                                    .name
                                    .to_lowercase()
                                    .contains(&name_val.to_lowercase())
                            })
                            .unwrap())
                    && (control.is_none() || control.map(|ctrl| session.control == ctrl).unwrap())
            })
            .collect();

        let total_count = filtered_sessions.len();

        // Apply pagination
        let start = (page.unwrap_or(1) - 1) * per_page.unwrap_or(6);
        let _end = start + per_page.unwrap_or(6);
        let paginated_sessions = filtered_sessions
            .into_iter()
            .skip(start)
            .take(per_page.unwrap_or(6));

        // Map filtered sessions to RoomInfo
        let rooms = paginated_sessions
            .map(|(room, session)| {
                (
                    room.clone(),
                    RoomInfo {
                        server: session.server.clone(),
                        viewer_count: session.viewers.len(),
                        viewers: session.viewers.iter().cloned().collect(),
                        os: session.os.clone(),
                        version: session.version.clone(),
                        name: session.name.clone(),
                        control: session.control,
                    },
                )
            })
            .collect();

        (rooms, total_count)
    }

    pub fn subscribe_room_updates(&mut self, peer_id: String) {
        self.room_update_subscribers.insert(peer_id);
    }

    pub fn unsubscribe_room_updates(&mut self, peer_id: &str) {
        self.room_update_subscribers.remove(peer_id);
    }

    pub fn notify_room_update(&self, room: &str) {
        for subscriber in &self.room_update_subscribers {
            if let Some(peer) = self.peers.get(subscriber) {
                let _ = peer.sender.unbounded_send(Message::Text(
                    serde_json::to_string(&SignallerMessage::NewRoomNotification {
                        room: room.to_string(),
                    })
                    .unwrap(),
                ));
            }
        }
    }
}
