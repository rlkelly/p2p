use std::sync::Arc;
// use std::net::{Ipv4Addr, IpAddr, SocketAddr};

use tokio::sync::Mutex;

use crate::models::Service;
use crate::codec::MessageEvent;
use super::process;

// TODO: drop peers after no response for some time
// TODO: figure how to add new connections

pub async fn run_scheduled_tasks(state: Arc<Mutex<Service>>) {
    let t = ping_all_peers(Arc::clone(&state));
    let t2 = peers_request(Arc::clone(&state));
    t.await;
    t2.await;
}

pub async fn ping_all_peers(state: Arc<Mutex<Service>>) {
    use std::net::SocketAddr;
    use tokio::net::TcpStream;
    let mut new_addr = Vec::new();

    {
        let mut state = state.lock().await;
        let peers = state.peers.clone();

        let peer_key = peers.keys();
        let peer_db = state.database.all_peers().iter().map(|x| x.address).collect::<Vec<SocketAddr>>();
        for peer in peers.clone().iter_mut() {
            let res = peer.1.send(MessageEvent::Ping(state.my_contact.clone()));

            if let Err(_e) = res {
                state.database.delete_entity(peer.0);
            }
        }
    }
    {
        let mut state = state.lock().await;
        let peers = state.peers.clone();
        let peer_key = peers.keys();
        let peer_db = state.database.all_peers().iter().map(|x| x.address).collect::<Vec<SocketAddr>>();
        for addr in peer_db {
            if !peers.contains_key(&addr) {
                new_addr.push(addr);
            }
        }
    }
    for addr in new_addr {
        let mut stream = TcpStream::connect(addr).await.expect("failed to connect");
        let state = Arc::clone(&state);
        tokio::spawn(async move {
            if let Err(e) = process(state, stream, addr).await {
                println!("an error occured; error = {:?}", e);
            }
        });
    }
}

pub async fn peers_request(state: Arc<Mutex<Service>>) {
    let mut state = state.lock().await;
    let mut peers = state.peers.clone();

    for peer in peers.iter_mut() {
        let res = peer.1.send(MessageEvent::PeersRequest);
        if let Err(_e) = res {
            state.database.delete_entity(peer.0);
        }
    }
}
