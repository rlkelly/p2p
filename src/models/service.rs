use std::collections::HashMap;
use std::net::SocketAddr;

use tokio::sync::mpsc;

type Tx = mpsc::UnboundedSender<String>;
pub type Rx = mpsc::UnboundedReceiver<String>;

pub struct Service {
    pub peers: HashMap<SocketAddr, Tx>,
}

impl Service {
    pub fn new() -> Service {
        Service { peers: HashMap::new() }
    }
}

impl Service {
    pub async fn broadcast(&mut self, sender: SocketAddr, message: &str) {
        for peer in self.peers.iter_mut() {
            if *peer.0 != sender {
                let _ = peer.1.send(message.into());
            }
        }
    }
}
