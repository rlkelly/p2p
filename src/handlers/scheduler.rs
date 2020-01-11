use std::sync::Arc;
use std::net::{Ipv6Addr, IpAddr, SocketAddr};

use bytes::{BytesMut, BufMut};
use tokio::sync::Mutex;
use tokio::net::TcpStream;
use tokio_util::codec::{Decoder, Encoder, Framed};

use crate::models::{Peer, Service};
use crate::codec::{
    MessageEvent,
    MessageCodec,
};

pub async fn scheduler(state: Arc<Mutex<Service>>) {
    // TODO:
    // update peers list
    let mut state = state.lock().await;
    state.incr();

    let server = SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), 8001);
    let me = Peer::new(server, false, Some("MyName".into()), None, Some("ZYX987".into()));
    for peer in state.peers.iter_mut() {
        let mut res = BytesMut::new();
        MessageCodec{}.encode(MessageEvent::Ping(me.clone()), &mut res).unwrap();
        peer.1.send(res);
    }
}
