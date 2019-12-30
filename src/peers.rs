/// https://github.com/libp2p/rust-libp2p
/// find peers
/// add peer
/// drop peer
/// get user's file list
/// ask if user has file

#[derive(Clone, PartialEq, Eq)]
pub struct Peer {
    id: String,
    addr: String,
}

impl Peer {
    pub fn new(id: String, addr: String) -> Peer {
        Peer {
            id: id,
            addr: addr,
        }
    }
}
