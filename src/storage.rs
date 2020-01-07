use bytes::{BytesMut, BufMut};
use std::io::prelude::*;
use std::fs::File;

use crate::models::Peer;

pub fn load_peers(filename: &str) -> Vec<Peer> {
    let mut peers_vec: Vec<Peer> = vec!();
    let mut f = File::open(filename).unwrap();
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).unwrap();
    let mut bytes = BytesMut::new();
    bytes.extend_from_slice(&buffer[..]);
    let mut peers_length = bytes.split_to(1)[0] as u8;
    while peers_length > 0 {
        let peer_length = bytes.split_to(1)[0] as usize;
        let mut peer_bytes = bytes.split_to(peer_length);
        let peer = Peer::from_bytes(&mut peer_bytes);
        peers_vec.push(peer);
        peers_length -= 1;
    }
    peers_vec
}

pub fn dump(filename: &str, peers: Vec<Peer>) {
    let mut buffer = BytesMut::new();
    buffer.put_u8(peers.len() as u8);

    for peer in peers {
        let bytes = peer.to_bytes();
        println!("{:?}", bytes);
        let bytes_len = bytes.len();
        buffer.put_u8(bytes_len as u8);
        buffer.extend_from_slice(&bytes[..]);
    }
    let mut file = File::create(filename).unwrap();
    file.write(&buffer[..]).expect("file write error");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv6Addr;
    use std::net::{
        SocketAddr,
        IpAddr,
    };

    #[test]
    fn test_dump() {
        let ip1 = SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), 8000);
        let ip2 = SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 2)), 8000);

        let p1 = Peer::new(ip1, false, None, None, None);
        let p2 = Peer::new(ip2, true, None, None, None);

        dump("/tmp/thing.bin", vec![p1.clone(), p2.clone()]);
        let peers = load_peers("/tmp/thing.bin");
        assert_eq!(peers, vec![p1, p2]);

    }
}
