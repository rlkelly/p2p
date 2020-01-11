use bytes::BytesMut;
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::sync::mpsc;

use crate::storage::Db;
use crate::models::{ArtistData, Peer};
use crate::organizer::get_collection;
use crate::args::Config;

type Tx = mpsc::UnboundedSender<BytesMut>;
pub type Rx = mpsc::UnboundedReceiver<BytesMut>;

pub struct Service {
    pub peers: HashMap<SocketAddr, Tx>,
    pub my_contact: Peer,
    pub database: Db,
    pub storage_dir: String,
    pub counter: u8,
    pub port: u16,
}

impl Service {
    pub fn new(config: Config) -> Service {
        // TODO: handle file errors
        Service {
            peers: HashMap::new(),
            my_contact: Peer::get_self(),
            database: Db::new_from_file(&config.config),
            storage_dir: config.music,
            counter: 0,
            port: config.port,
        }
    }

    pub fn get_collection(&self, track_data: bool, artist_filter: Option<&str>, album_filter: Option<&str>) -> Vec<ArtistData> {
        get_collection(&self.storage_dir, track_data, artist_filter, album_filter)
    }

    pub fn get_peers(&self) -> Vec<Peer> {
        self.database.all_peers()
    }

    pub fn incr(&mut self) {
        self.counter += 1;
    }
}

impl Service {
    pub async fn broadcast(&mut self, sender: SocketAddr, message: &str) {
        for peer in self.peers.iter_mut() {
            // TODO: this is prob not necessary anymore
            // if *peer.0 != sender {
            //     let _ = peer.1.send(message.as_bytes());
            // }
        }
    }
}
