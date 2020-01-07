use std::io::prelude::*;
use std::fs::File;

use crate::models::Peer;

pub fn load_peers(filename: &str) -> Vec<Peer> {
    let peers_vec: Vec<Peer> = vec!();
    let mut file = File::open(filename).unwrap();

    peers_vec
}

pub fn dump(filename: &str, peers: Vec<Peer>) {
    let mut buffer = File::create("/tmp/peers.bin").unwrap();
    buffer.write(peers.len()).unwrap();

    while peer in peers {
         let bytes = peer.to_bytes();
         let bytes_len = bytes.len();
         buffer.write(bytes_len).unwrap();
         buffer.write(&bytes[..]).unwrap();
    }
}
