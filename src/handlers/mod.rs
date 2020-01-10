use crate::models::{Peer, PeerConnection};
pub use crate::models::Service;

use std::sync::Arc;
use std::error::Error;
use std::net::SocketAddr;

use tokio::sync::Mutex;
use tokio::stream::StreamExt;
use tokio_util::codec::Framed;

use tokio::net::{
    TcpStream,
};

use crate::codec::{
    MessageEvent,
    MessageCodec,
};

pub async fn process(
    state: Arc<Mutex<Service>>,
    stream: TcpStream,
    addr: SocketAddr,
) -> Result<(), Box<dyn Error>> {
    let transport = Framed::new(stream, MessageCodec::new());
    let mut peer = PeerConnection::new(state.clone(), transport).await?;
    while let Some(result) = peer.next().await {
        match result {
            Ok(MessageEvent::Ping(peer_data)) => {
                let mut state = state.lock().await;
                peer.send_message(MessageEvent::Pong(state.my_contact.clone())).await.unwrap();
                state.database.add_peer(peer_data)
            },
            Ok(MessageEvent::Pong(peer_data)) => {
                let mut state = state.lock().await;
                state.database.add_peer(peer_data)
            },
            Ok(MessageEvent::ArtistsRequest) => {
                // send artists
            },
            Ok(MessageEvent::ArtistsResponse(artist_data)) => {
                // add artists to ecs for peer
            },
            Ok(MessageEvent::AlbumRequest(album)) => {
                // send albums
            },
            Ok(MessageEvent::AlbumResponse(track_data)) => {
                // display track data
            },
            Ok(MessageEvent::PeersRequest) => {
                // send peers
            },
            Ok(MessageEvent::PeersResponse(peers_list)) => {
                // load peers
            },
            Ok(MessageEvent::Received(msg)) => {
                peer.send_message(MessageEvent::Payload(msg)).await.unwrap();
            },
            Ok(MessageEvent::Ok) => {},
            Ok(MessageEvent::Broadcast(msg)) => {
                let mut state = state.lock().await;
                state.broadcast(addr, &msg).await;
            },
            Ok(MessageEvent::Payload(msg)) => {
                let mut state = state.lock().await;
                state.broadcast(addr, &msg).await;
            },
            Err(e) => {
                println!(
                    "an error occured while processing messages; error = {:?}", e
                );
            },
            _ => println!("UNK"), // do nothing?
        }
    }
    let mut state = state.lock().await;
    state.peers.remove(&addr);
    let msg = "broadcast test3";
    state.broadcast(addr, &msg).await;
    Ok(())
}
