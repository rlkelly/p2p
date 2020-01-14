use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::sync::mpsc;

use crate::storage::Db;
use crate::models::{ArtistData, Collection, Peer};
use crate::codec::MessageEvent;
use crate::organizer::get_collection;
use crate::args::Config;

type Tx = mpsc::UnboundedSender<MessageEvent>;
pub type Rx = mpsc::UnboundedReceiver<MessageEvent>;

pub struct Service {
    pub peers: HashMap<SocketAddr, Tx>,
    pub my_contact: Peer,
    pub database: Db,
    pub storage_dir: String,
    pub port: u16,
}

impl Service {
    pub fn new(config: Config) -> Service {
        // TODO: handle file errors
        Service {
            peers: HashMap::new(),
            my_contact: Peer::new(
                format!("127.0.0.1:{}", config.port).parse().unwrap(),
                true,
                Some(config.name),
                Some("".into()),
                Some("ZYX987".into()),
            ),
            database: Db::new_from_file(&config.config),
            storage_dir: config.music,
            port: config.port,
        }
    }

    pub fn get_collection(&self, track_data: bool, artist_filter: Option<&str>, album_filter: Option<&str>) -> Vec<ArtistData> {
        get_collection(&self.storage_dir, track_data, artist_filter, album_filter)
    }

    pub fn get_peers(&self) -> Vec<Peer> {
        self.database.all_peers()
    }

    pub fn add_peers(&mut self, peers: Vec<Peer>) {
        for peer in peers {
            if self.my_contact == peer {
                continue;
            }
            self.database.add_peer(peer, Collection::new(vec![]));
        }
    }

    pub fn add_peer(&mut self, p: Peer, c: Collection, addr: &SocketAddr) {
        if addr == &self.my_contact.address || p == self.my_contact {
            return;
        }
        self.database.add_peer(p.clone(), c);
        self.database.insert_address(&addr, p);
    }

    pub fn update_peer_key(&mut self, new_addr: SocketAddr, old_addr: &SocketAddr) {
        if new_addr == *old_addr {
            return;
        }
        if let Some(peer) = self.peers.get(old_addr) {
            self.peers.insert(new_addr, peer.clone());
            self.peers.remove(&old_addr);
        }
    }
}

impl Service {
    pub async fn broadcast(&mut self, message: &MessageEvent) {
        for peer in self.peers.iter_mut() {
            let _ = peer.1.send(message.clone());
        }
    }
}
