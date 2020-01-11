use std::sync::Arc;
use std::net::{Ipv6Addr, IpAddr, SocketAddr};

use tokio::sync::Mutex;

use crate::models::{Peer, Service};
use crate::codec::MessageEvent;

pub async fn ping_all_peers(state: Arc<Mutex<Service>>) {
    // TODO:
    // update peers list
    let mut state = state.lock().await;
    state.incr();

    let server = SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), 8001);
    let me = Peer::new(server, false, Some("MyName".into()), None, Some("ZYX987".into()));
    for peer in state.peers.iter_mut() {
        peer.1.send(MessageEvent::Ping(me.clone())).unwrap();
    }
}
