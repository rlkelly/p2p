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
        let mut state = state.lock().await;
        let msg = "broadcast test";
        state.broadcast(addr, &msg).await;
    }

    while let Some(result) = peer.next().await {
        match result {
            Ok(MessageEvent::Ping(_peer_data)) => {
                // TODO: add to ecs
                peer.send_message(MessageEvent::Pong).await.unwrap();
            },
            Ok(MessageEvent::Pong) => {
                // add to peers
            },
            Ok(MessageEvent::Broadcast(msg)) => {
                let mut state = state.lock().await;
                state.broadcast(addr, &msg).await;
            },
            Ok(MessageEvent::Payload(msg)) => {
                let mut state = state.lock().await;
                state.broadcast(addr, &msg).await;
            },
            Ok(MessageEvent::Received(msg)) => {
                peer.send_message(MessageEvent::Payload(msg)).await.unwrap();
            },
            Err(e) => {
                println!(
                    "an error occured while processing messages; error = {:?}", e
                );
            },
            _ => println!("UNK"),
        }
    }
    {
        let mut state = state.lock().await;
        state.peers.remove(&addr);
        let msg = "broadcast test3";
        state.broadcast(addr, &msg).await;
    }
    Ok(())
}
