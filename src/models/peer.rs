use axum::extract::ws::Message;
#[allow(unused_imports)]
#[allow(unused_variables)]
#[allow(dead_code)]
use futures_channel::mpsc::UnboundedSender;
use serde::{Deserialize, Serialize};

type Tx = UnboundedSender<Message>;

pub struct Peer {
    pub room: String,
    pub sender: Tx,
    #[allow(dead_code)]
    pub peer_type: PeerType,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PeerType {
    Server {},
    Viewer {},
}
