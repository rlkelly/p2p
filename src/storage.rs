use bytes::{BytesMut, BufMut};
use std::io::prelude::*;
use std::fs::File;

use specs::prelude::{Component, DenseVecStorage, FlaggedStorage, World};
use specs::{RunNow, WorldExt};
use specs::join::Join;
use specs::world::Builder;

use std::net::SocketAddr;
use crate::models::{AlbumData, ArtistData, Collection, Peer};
use crate::ecs::{
    Node,
    // NodeEvent,
    NodeSystem,
    WorldState,
};

impl Component for Collection {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

impl Component for Peer {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

impl Node<SocketAddr> for Peer {
    fn index(&self) -> SocketAddr {
        self.address
    }
}

pub struct Db {
    world: World,
    system: NodeSystem<Peer, SocketAddr>,
}

impl Db {
    pub fn new() -> Self {
        let mut world = World::new();
        world.register::<Peer>();
        world.register::<Collection>();
        let system = NodeSystem::<Peer, SocketAddr>::new(&mut world);
        let _reader_id = world.write_resource::<WorldState<Peer, SocketAddr>>().track();
        Db {
            world,
            system,
        }
    }

    pub fn maintain(&mut self) {
        self.system.run_now(&mut self.world);
        self.world.maintain();
    }

    pub fn insert_address(&mut self, addr: &SocketAddr, peer: Peer) {
        let entity = self.world.fetch::<WorldState<Peer, SocketAddr>>().get_entity(&peer.address).expect("FAILED GET ENTITY");
        self.world.fetch_mut::<WorldState<Peer, SocketAddr>>().insert_address(&addr, entity);
        self.maintain();
    }

    pub fn all_peers(&self) -> Vec<Peer> {
        self.world.read_storage::<Peer>().join().map(|x| x.clone()).collect()
    }

    pub fn add_peer(&mut self, p: Peer, c: Collection) {
        // // todo: better comparison
        if None == self.world.fetch::<WorldState<Peer, SocketAddr>>().get_entity(&p.address) {
            self.world.create_entity().with(p).with(c).build();
            self.maintain();
        }
    }

    pub fn add_peers(&mut self, peers: Vec<Peer>) {
        for peer in peers {
            let collection = self.get_collection(&peer.index());
            self.add_peer(peer, collection);
        }
        self.maintain()
    }

    pub fn add_tracks(&mut self, addr: &SocketAddr, album_data: AlbumData) {
        let mut collection = self.get_collection(addr);
        let artist_name = album_data.clone().artist.unwrap();
        let album_title = album_data.clone().album_title;

        // TODO: change to hashmap?
        // TODO: insert if not found
        for artist in &mut collection.artists {
            if artist.artist == artist_name {
                for album in artist.albums.as_mut().unwrap() {
                    if album.album_title == album_title {
                        *album = album_data;
                        self.update_collection(addr, collection);
                        return;
                    }
                }
                if artist.albums == None {
                    artist.albums = Some(vec![album_data]);
                } else {
                    artist.albums.as_mut().unwrap().push(album_data);
                }
                self.update_collection(addr, collection);
                return;
            }
        }
        collection.artists.push(ArtistData::new(album_data.clone().artist.unwrap(), Some(vec![album_data])));
        self.update_collection(addr, collection);
    }

    pub fn update_collection(&mut self, addr: &SocketAddr, c: Collection) {
        match self.world.fetch::<WorldState<Peer, SocketAddr>>().get_entity(addr) {
            Some(entity) => {
                self.world.write_storage::<Collection>()
                    .insert(entity, c)
                    .unwrap();
            }
            None => {
                panic!("PEER DOESNT EXIST!")
            }
        };
    }

    pub fn delete_entity(&mut self, addr: &SocketAddr) {
        let entity_result = self.world.fetch::<WorldState<Peer, SocketAddr>>().get_entity(addr);
        if let Some(entity) = entity_result {
            self.world.delete_entity(entity).expect("entity deletion failed");
        }
        self.world.fetch_mut::<WorldState<Peer, SocketAddr>>().remove_address(&addr);
        self.maintain();
    }

    pub fn get_collection(&mut self, addr: &SocketAddr) -> Collection {
        let entity = self.world.fetch::<WorldState<Peer, SocketAddr>>().get_entity(addr);
        if entity == None {
            return Collection::new(vec![]);
        }
        // use Box?
        self.world.read_storage::<Collection>()
            .get(entity.unwrap())
            .map_or(Collection::new(vec![]), |v| v.clone())
    }

    pub fn new_from_file(filename: &str) -> Self {
        let mut db = Db::new();
        let mut f = File::open(filename).unwrap();
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer).unwrap();
        if buffer.len() == 0 {
            return db;
        }
        let mut bytes = BytesMut::new();
        bytes.extend_from_slice(&buffer[..]);
        let mut peers_length = bytes.split_to(1)[0] as u8;
        while peers_length > 0 {
            let peer_length = bytes.split_to(1)[0] as usize;
            let mut peer_bytes = bytes.split_to(peer_length);
            let peer = Peer::from_bytes(&mut peer_bytes);
            db.add_peer(peer, Collection::new(vec![]));
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
    use crate::models::{ArtistData, AlbumData, TrackData};

    #[test]
    fn test_dump_and_load() {
        let ip1 = SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), 8000);
        let ip2 = SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 2)), 8000);

        let p1 = Peer::new(ip1, false, Some("TEST".into()), None, Some("ZYX987".into()));
        let p2 = Peer::new(ip2, true, None, Some("ABC123".into()), None);

        dump("/tmp/thing1.bin", vec![p1.clone(), p2.clone()]);
        let db = Db::new_from_file("/tmp/thing1.bin");
        let peers = db.all_peers();
        assert_eq!(peers, vec![p1, p2]);
    }

    #[test]
    fn test_update_collection() {
        let ip1 = SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), 8000);
        let ip2 = SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 2)), 8000);

        let p1 = Peer::new(ip1, false, Some("TEST".into()), None, Some("ZYX987".into()));
        let p2 = Peer::new(ip2, true, None, Some("ABC123".into()), None);

        dump("/tmp/thing2.bin", vec![p1.clone(), p2.clone()]);
        let mut db = Db::new_from_file("/tmp/thing2.bin");
        assert_eq!(db.get_collection(&ip1), Collection::new(vec![]));

        let artist_data = ArtistData::new(
            "test1".to_string(),
            Some(
                vec![
                    AlbumData::new(
                        Some(
                            "test2".to_string()),
                            "test3".to_string(),
                            0,
                            Some(vec![TrackData::new("test".to_string(), 12_000, 250)]),
                        ),
                    AlbumData::new(Some("test2".to_string()), "test3".to_string(), 0, None),
                ]
            ),
        );
        let artists_vec = vec![artist_data.clone()];
        let collection = Collection::new(artists_vec);
        db.update_collection(&ip1, collection.clone());
        assert_eq!(db.all_peers().len(), 2);
        assert_eq!(db.get_collection(&ip1), collection);
    }

    #[test]
    fn test_add_tracks() {
        let ip1 = SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), 8000);
        let p1 = Peer::new(ip1, false, Some("TEST".into()), None, Some("ZYX987".into()));

        dump("/tmp/thing3.bin", vec![p1.clone(), ]);
        let mut db = Db::new_from_file("/tmp/thing3.bin");
        let artist_data = ArtistData::new(
            "first artist".to_string(),
            Some(
                vec![
                    AlbumData::new(
                        Some("first artist".to_string()),
                        "first album".to_string(),
                        0,
                        None,
                    ),
                    AlbumData::new(
                        Some("first artist".to_string()),
                        "second album".to_string(),
                        0,
                        None,
                    ),
                ]
            ),
        );
        let collection = Collection::new(vec![artist_data]);
        db.update_collection(&ip1, collection.clone());
        assert_eq!(None, db.get_collection(&ip1).artists[0].albums.as_ref().unwrap()[0].tracks);

        let album_data = AlbumData::new(
            Some("first artist".to_string()),
            "first album".to_string(),
            1,
            Some(vec![TrackData::new("test".to_string(), 12_000, 250)]),
        );
        db.add_tracks(&ip1, album_data);
        assert_eq!(1, db.get_collection(&ip1).artists[0].albums.as_ref().unwrap()[0].tracks.as_ref().unwrap().len());
    }

    #[test]
    fn test_add_peers() {
        let ip1 = SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), 8000);
        let p1 = Peer::new(ip1, false, Some("TEST".into()), None, Some("ZYX987".into()));

        dump("/tmp/thing3.bin", vec![]);
        let mut db = Db::new_from_file("/tmp/thing3.bin");
        db.add_peers(vec![p1.clone(), ]);
        let album_data = AlbumData::new(
            Some("first artist".to_string()),
            "first album".to_string(),
            1,
            Some(vec![TrackData::new("test".to_string(), 12_000, 250)]),
        );
        db.add_tracks(&ip1, album_data);
        db.add_peers(vec![p1, ]);
        assert_eq!(1, db.get_collection(&ip1).artists[0].albums.as_ref().unwrap()[0].tracks.as_ref().unwrap().len());
        let album_data = AlbumData::new(
            Some("first artist".to_string()),
            "second album".to_string(),
            1,
            Some(vec![TrackData::new("test".to_string(), 12_000, 250)]),
        );
        db.add_tracks(&ip1, album_data);
        assert_eq!(1, db.get_collection(&ip1).artists[0].albums.as_ref().unwrap()[1].tracks.as_ref().unwrap().len());
    }
}
