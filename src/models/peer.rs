use std::convert::TryInto;

use bytes::{BytesMut, BufMut};

use std::str;
use std::net::{
    SocketAddr,
    IpAddr,
};
use serde::{Deserialize, Serialize};
use super::{
    take_u64,
    get_nstring,
    bytes_to_ip_addr,
};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Peer {
    pub address: SocketAddr,
    accept_incoming: bool,
    pub name: Option<String>,
    public_key: Option<String>,
    signature: Option<String>, // sign to prove they have private key
}

impl Peer {
    pub fn new(address: SocketAddr, accept_incoming: bool, name: Option<String>, public_key: Option<String>, signature: Option<String>) -> Self {
        Peer {
            address,
            accept_incoming,
            name,
            public_key,
            signature,
        }
    }

    pub fn update_addr(&mut self, addr: SocketAddr) -> Self {
        self.address = addr;
        self.clone()
    }

    pub fn get_self(addr: SocketAddr) -> Self {
        // TODO: get ip address and port on init
        Peer::new(
            addr,
            true,
            Some("TEST".into()),
            Some("ABC123".into()),
            Some("ZYX987".into()))
    }

    pub fn to_bytes(&self) -> BytesMut {
        let mut buf = BytesMut::new();
        let ip_bytes = match self.address.ip() {
            IpAddr::V4(ip) => ip.octets().to_vec(),
            IpAddr::V6(ip) => ip.octets().to_vec(),
        };
        let len = ip_bytes.len();
        buf.put_u64(len as u64);
        buf.put(&ip_bytes[..]);
        buf.put_u16(self.address.port());

        if self.accept_incoming {
            buf.put_u8(1);
        } else {
            buf.put_u8(0);
        };

        // TODO: factor this out
        if let Some(name) = &self.name {
            let name_length: u8 = name.len().try_into().unwrap();
            buf.put_u8(name_length);
            buf.put(name.as_bytes());
        } else {
            buf.put_u8(0);
        };
        if let Some(public_key) = &self.public_key {
            let public_key_length: u8 = public_key.len().try_into().unwrap();
            buf.put_u8(public_key_length);
            buf.put(public_key.as_bytes());
        } else {
            buf.put_u8(0);
        };
        if let Some(signature) = &self.signature {
            let signature_length: u8 = signature.len().try_into().unwrap();
            buf.put_u8(signature_length);
            buf.put(signature.as_bytes());
        } else {
            buf.put_u8(0);
        };
        buf
    }

    pub fn from_bytes(buf: &mut BytesMut) -> Self {
        let ip_len = take_u64(buf).unwrap();
        let address = bytes_to_ip_addr(buf, ip_len as usize);
        let accept_incoming_byte = buf.split_to(1)[0] as u8;
        let accept_incoming = accept_incoming_byte == 1u8;
        let name_key = buf.split_to(1)[0] as usize;
        let name = get_nstring(buf, name_key);
        let pk_key = buf.split_to(1)[0] as usize;
        let public_key = get_nstring(buf, pk_key);
        let signature_key = buf.split_to(1)[0] as usize;
        let signature = get_nstring(buf, signature_key);
        Peer {
            address,
            accept_incoming,
            name,
            public_key,
            signature,
        }
    }
}
