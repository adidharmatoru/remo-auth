use axum::extract::ws::{Message, WebSocket};
use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};
use log::info;
use std::net::{IpAddr, SocketAddr};

use crate::{args::Args, models::rtc::SignallerMessage, models::state::StateType};

type Tx = UnboundedSender<Message>;

pub async fn handle_connection(
    _args: Args,
    state: StateType,
    websocket: WebSocket,
    socket_addr: SocketAddr,
    real_ip: Option<&IpAddr>,
) {
    info!(
        "WebSocket connection established: {socket_addr}, real IP: {:?}",
        real_ip
    );

    let (tx, rx) = unbounded();
    let (outgoing, incoming) = websocket.split();

    let handle_incoming =
        incoming.try_for_each(|msg| process_message(msg, state.clone(), &tx, socket_addr));

    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(handle_incoming, receive_from_others);
    future::select(handle_incoming, receive_from_others).await;

    info!("{socket_addr} disconnected, real IP: {:?}", real_ip);
    state.lock().await.on_disconnect(&socket_addr);
}

pub async fn handle_message(
    state: &mut crate::models::state::State,
    tx: &Tx,
    raw_payload: &str,
    socket_addr: SocketAddr,
) -> Result<(), failure::Error> {
    let msg: SignallerMessage = serde_json::from_str(raw_payload)?;
    let forward_message =
        |state: &crate::models::state::State, to: String| -> Result<(), failure::Error> {
            let peer = state
                .peers
                .get(&to)
                .ok_or_else(|| failure::format_err!("Peer does not exist"))?;
            peer.sender
                .unbounded_send(Message::Text(raw_payload.to_string()))?;
            Ok(())
        };

    match msg {
        SignallerMessage::Start {
            room,
            name,
            os,
            version,
            control,
        } => {
            state.add_server(
                room.clone(),
                name,
                os,
                version,
                control,
                tx.clone(),
                socket_addr,
            )?;
            tx.unbounded_send(Message::Text(serde_json::to_string(
                &SignallerMessage::StartResponse { room: room.clone() },
            )?))?;
            state.notify_room_update(&room);
        }
        SignallerMessage::Join { from, room } => {
            info!("{} attempting to join room {}", from, room);
            match state.add_viewer(from.clone(), room.clone(), tx.clone()) {
                Ok(_) => {
                    info!("{} joined room {}", from, room);
                    forward_message(state, room)?;
                }
                Err(e) => {
                    info!("Error joining room: {}", e);
                    tx.unbounded_send(Message::Text(serde_json::to_string(
                        &SignallerMessage::JoinDeclined {
                            to: from,
                            reason: e.to_string(),
                        },
                    )?))?;
                }
            }
        }
        SignallerMessage::Leave { from } => {
            state.leave_session(from)?;
        }
        SignallerMessage::Offer { to, .. }
        | SignallerMessage::Answer { to, .. }
        | SignallerMessage::Ice { to, .. }
        | SignallerMessage::JoinDeclined { to, .. } => {
            forward_message(state, to)?;
        }
        SignallerMessage::IceServers {} => {
            let ice_servers = state.get_ice_servers().await;
            tx.unbounded_send(Message::Text(serde_json::to_string(
                &SignallerMessage::IceServersResponse { ice_servers },
            )?))?;
        }
        SignallerMessage::GetRoomList {
            os,
            name,
            version,
            server,
            sort,
            control,
            page,
            per_page,
        } => {
            let (rooms, total_count) = state.get_available_rooms(
                os.as_deref(),
                version.as_deref(),
                server.as_deref(),
                name.as_deref(),
                sort.as_deref(),
                control,
                page,
                per_page,
            );
            tx.unbounded_send(Message::Text(serde_json::to_string(
                &SignallerMessage::RoomListResponse {
                    rooms,
                    total_count,
                    page,
                    per_page,
                },
            )?))?;
        }
        SignallerMessage::KeepAlive {} => {}
        SignallerMessage::RoomListResponse { .. }
        | SignallerMessage::NewRoomNotification { .. } => {
            log::warn!("Received unexpected message: {:?}", msg);
        }
        SignallerMessage::SubscribeRoomUpdates {} => {
            if let Some(peer) = state.peers.values().find(|p| std::ptr::eq(&p.sender, tx)) {
                let room = peer.room.clone();
                state.subscribe_room_updates(room);
            }
        }
        SignallerMessage::UnsubscribeRoomUpdates {} => {
            if let Some(peer) = state.peers.values().find(|p| std::ptr::eq(&p.sender, tx)) {
                let room = peer.room.clone();
                state.unsubscribe_room_updates(&room);
            }
        }
        _ => {}
    }
    Ok(())
}

pub async fn process_message(
    msg: Message,
    state: StateType,
    tx: &Tx,
    socket_addr: SocketAddr,
) -> Result<(), axum::Error> {
    if let Message::Text(text) = msg {
        let mut locked_state = state.lock().await;
        if let Err(e) = handle_message(&mut locked_state, tx, &text, socket_addr).await {
            info!(
                "Error occurred when handling message: {}\nMessage: {}",
                e, text
            );
        }
    }
    Ok(())
}
