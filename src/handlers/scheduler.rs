use std::sync::Arc;
use std::net::{Ipv6Addr, IpAddr, SocketAddr};

use tokio::sync::Mutex;

use crate::models::{Peer, Service};
use crate::codec::MessageEvent;

// TODO: drop peers after no response for some time
// TODO: figure how to add new connections

pub async fn run_scheduled_tasks(state: Arc<Mutex<Service>>) {
    // let t = ping_all_peers(Arc::clone(&state));
    // let t2 = peers_request(Arc::clone(&state));
    // t.await;
    // t2.await;
}

pub async fn ping_all_peers(state: Arc<Mutex<Service>>) {
    let mut state = state.lock().await;
    state.incr();

    let server = SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), 8001);
    let me = Peer::new(server, false, Some("MyName".into()), None, Some("ZYX987".into()));
    for peer in state.peers.iter_mut() {
        peer.1.send(MessageEvent::Ping(me.clone())).unwrap();
    }
}

pub async fn peers_request(state: Arc<Mutex<Service>>) {
    let mut state = state.lock().await;
    for peer in state.peers.iter_mut() {
        peer.1.send(MessageEvent::PeersRequest).unwrap();
    }
}
