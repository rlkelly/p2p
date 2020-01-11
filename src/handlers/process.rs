use std::sync::Arc;
use std::error::Error;
use std::net::SocketAddr;

use tokio::sync::Mutex;
use tokio::stream::StreamExt;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

pub use crate::models::Service;
use crate::models::{Collection, PeerConnection};
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
                state.database.add_peer(peer_data, Collection::new(vec![]));
                peer.send_message(MessageEvent::ArtistsRequest).await.unwrap();
            },
            Ok(MessageEvent::Pong(peer_data)) => {
                let mut state = state.lock().await;
                state.database.add_peer(peer_data, Collection::new(vec![]));
                peer.send_message(MessageEvent::ArtistsRequest).await.unwrap();
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
            Ok(MessageEvent::Received(mut msg)) => {
                use tokio_util::codec::{Decoder, Encoder};
                let res = MessageCodec{}.decode(&mut msg).unwrap().unwrap();
                peer.send_message(res).await.unwrap();
            },
            Ok(MessageEvent::Broadcast(msg)) => {
                let mut state = state.lock().await;
                state.broadcast(addr, &msg).await;
            },
            Ok(MessageEvent::Payload(msg)) => {
                let mut state = state.lock().await;
                state.broadcast(addr, &msg).await;
            },
            Ok(MessageEvent::Ok) => {
                let ss = state.lock().await;
                println!("COUNTER: {:?}", ss.counter);
            },
            Err(e) => {
                println!(
                    "an error occured while processing messages; error = {:?}", e
                );
            },
            _ => println!("UNK"), // do nothing?
        }
    }
    Ok(())
}
