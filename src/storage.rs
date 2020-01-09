use bytes::{BytesMut, BufMut};
use std::io::prelude::*;
use std::fs::File;

use specs::prelude::{Component, DenseVecStorage, FlaggedStorage, World};
// use specs::world::Builder;
use specs::WorldExt;
use specs::join::Join;
use specs::world::Builder;

use std::net::SocketAddr;
use crate::models::Peer;
use crate::ecs::{
    Node,
    NodeSystem,
    WorldState,
};


impl Component for Peer {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

impl Node for Peer {
    fn addr(&self) -> SocketAddr {
        self.address
    }
}

pub struct Db {
    world: World,
}

impl Db {
    pub fn new() -> Self {
        let mut world = World::new();
        world.register::<Peer>();
        let _system = NodeSystem::<Peer>::new(&mut world);
        let _reader_id = world.write_resource::<WorldState<Peer>>().track();
        Db {
            world,
        }
    }

    pub fn maintain(&mut self) {
        self.world.maintain();
    }

    pub fn all_peers(&self) -> Vec<Peer> {
        self.world.read_storage::<Peer>().join().map(|x| x.clone()).collect()
    }

    pub fn add_entity(&mut self, n: Peer) {
        self.world.create_entity().with(n).build();
        self.maintain()
    }

    pub fn new_from_file(filename: &str) -> Self {
        let mut db = Db::new();
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
            db.add_entity(peer);
            peers_length -= 1;
        }
        db
    }
}

pub fn dump(filename: &str, peers: Vec<Peer>) {
    let mut buffer = BytesMut::new();
    buffer.put_u8(peers.len() as u8);

    for peer in peers {
        let bytes = peer.to_bytes();
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
    fn test_dump_and_load() {
        let ip1 = SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), 8000);
        let ip2 = SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 2)), 8000);

        let p1 = Peer::new(ip1, false, None, None, None);
        let p2 = Peer::new(ip2, true, None, None, None);

        dump("/tmp/thing.bin", vec![p1.clone(), p2.clone()]);
        let db = Db::new_from_file("/tmp/thing.bin");
        let peers = db.all_peers();
        assert_eq!(peers, vec![p1, p2]);
    }
}
