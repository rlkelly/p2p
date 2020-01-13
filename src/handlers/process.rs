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

    {
        let sstate = state.lock().await;
        peer.send_message(MessageEvent::Ping(sstate.my_contact.clone())).await.unwrap();
    }

    while let Some(result) = peer.next().await {
        match result {
            Ok(MessageEvent::Ping(peer_data)) => {
                let mut state = state.lock().await;
                peer.send_message(MessageEvent::Pong(state.my_contact.clone())).await.expect("FAILED PONG");
                state.add_peer(peer_data.clone(), Collection::new(vec![]), &addr);
                peer.send_message(MessageEvent::ArtistsRequest).await.expect("FAILED ARTIST REQUEST");
            },
            Ok(MessageEvent::Pong(peer_data)) => {
                // TODO: how to handle connection different from server???
                //       when a user connects to their initial peer, their ip address
                //       will be different than their server address

                let mut state = state.lock().await;
                state.add_peer(peer_data.clone(), Collection::new(vec![]), &addr);
                peer.send_message(MessageEvent::ArtistsRequest).await.expect("FAILED ARTIST REQUEST");
            },
            Ok(MessageEvent::ArtistsRequest) => {
                let state = state.lock().await;
                let collection = state.get_collection(true, None, None);
                peer.send_message(MessageEvent::ArtistsResponse(
                    collection
                )).await.expect("Artists Response Send Fail");
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
                ).await.expect("Artists Response Send Fail");
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
                ).await.expect("Peers Request Fail");
            },
            Ok(MessageEvent::PeersResponse(peers_list)) => {
                let mut state = state.lock().await;
                state.add_peers(peers_list);
            },
            Ok(MessageEvent::DownloadRequest(_album)) => {
                // TODO: get album files
                //       send chunks
                //       other user assembles chunks
            },
            Ok(MessageEvent::Ok) => (),
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
