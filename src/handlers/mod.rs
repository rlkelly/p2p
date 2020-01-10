use crate::models::PeerConnection;
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

use crate::models::Collection;
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
                state.database.add_peer(peer_data, Collection::new(vec![]))
            },
            Ok(MessageEvent::Pong(peer_data)) => {
                let mut state = state.lock().await;
                state.database.add_peer(peer_data, Collection::new(vec![]))
            },
            Ok(MessageEvent::ArtistsRequest) => {
                let state = state.lock().await;
                peer.send_message(MessageEvent::ArtistsResponse(
                    state.get_collection(false, None, None))
                ).await.unwrap();
            },
            Ok(MessageEvent::ArtistsResponse(artist_data)) => {
                let mut state = state.lock().await;
                state.database.update_collection(&addr, Collection::new(artist_data));
            },
            Ok(MessageEvent::AlbumRequest(album)) => {
                let state = state.lock().await;
                peer.send_message(
                    MessageEvent::ArtistsResponse(
                        state.get_collection(
                            false, album.artist.as_deref(),
                            Some(&album.album_title),
                    ))
                ).await.unwrap();
            },
            Ok(MessageEvent::AlbumResponse(album_data)) => {
                let mut state = state.lock().await;
                state.database.add_tracks(&addr, album_data);
            },
            Ok(MessageEvent::PeersRequest) => {
                let state = state.lock().await;
                peer.send_message(
                    MessageEvent::PeersResponse(
                        state.database.all_peers()
                    )
                ).await.unwrap();
            },
            Ok(MessageEvent::PeersResponse(peers_list)) => {
                let mut state = state.lock().await;
                state.database.add_peers(peers_list);
            },
            Ok(MessageEvent::Received(msg)) => {
                peer.send_message(MessageEvent::Payload(msg)).await.unwrap();
            },
            Ok(MessageEvent::Broadcast(msg)) => {
                let mut state = state.lock().await;
                state.broadcast(addr, &msg).await;
            },
            Ok(MessageEvent::Payload(msg)) => {
                let mut state = state.lock().await;
                state.broadcast(addr, &msg).await;
            },
            Ok(MessageEvent::Ok) => {},
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
