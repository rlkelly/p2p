use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::sync::mpsc;
use crate::storage::Db;
use crate::models::{ArtistData, Peer};
use crate::organizer::get_collection;

type Tx = mpsc::UnboundedSender<String>;
pub type Rx = mpsc::UnboundedReceiver<String>;

pub struct Service {
    pub peers: HashMap<SocketAddr, Tx>,
    pub my_contact: Peer,
    pub database: Db,
    pub storage_dir: String,
}

impl Service {
    pub fn new() -> Service {
        // TODO: handle file errors
        Service {
            peers: HashMap::new(),
            my_contact: Peer::get_self(),
            database: Db::new_from_file("/tmp/thing.bin"),
            storage_dir: "~/Documents/music".into(),
        }
    }

    pub fn get_collection(&self, track_data: bool, artist_filter: Option<&str>, album_filter: Option<&str>) -> Vec<ArtistData> {
        get_collection(&self.storage_dir, track_data, artist_filter, album_filter)
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
