use tokio_util::codec::{Decoder, Encoder, Framed};
use bytes::{BytesMut, BufMut};
use byteorder::{BigEndian, ReadBytesExt};

use std::io::{self, Cursor};
use std::str;
use serde_json;
use std::net::{
    SocketAddr,
    IpAddr,
    Ipv4Addr, Ipv6Addr,
};
use serde::{Deserialize, Serialize};

use crate::consts::{
    PING, PONG,
};

const LENGTH_FIELD_LEN: usize = 4;


#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum MessageEvent {
    Ping(SocketAddr),
    Pong(SocketAddr),
    Payload(String),
    Broadcast(String),
    Received(String),
    AddrVec(Vec<SocketAddr>),
    ArtistRequest(String),
    AlbumRequest(String),
    ArtistResponse(String),
    AlbumResponse(String),
    PeersRequest(String),
    PeersResponse(String),
    Ok,
}

#[derive(Debug)]
pub enum CodecError {
    SerializationError,
    DataLengthMismatch,
    IO(std::io::Error),
}

impl From<std::io::Error> for CodecError {
  fn from(err: std::io::Error) -> CodecError {
    CodecError::IO(err)
  }
}

impl PartialEq for CodecError {
  fn eq(&self, other: &Self) -> bool {
    match (&self, &other) {
      (CodecError::IO(a), CodecError::IO(b)) => a.kind() == b.kind(),
      _ => false
    }
  }
}

pub struct MessageCodec {}

impl Encoder for MessageCodec {
  type Item = MessageEvent;
  type Error = CodecError;

  fn encode(&mut self, event: Self::Item, buf: &mut BytesMut) ->
    Result<(), Self::Error> {
        match event {
            MessageEvent::Ping(addr) => {
                let ip_bytes = match addr.ip() {
                    IpAddr::V4(ip) => ip.octets().to_vec(),
                    IpAddr::V6(ip) => ip.octets().to_vec(),
                };
                buf.reserve(16);
                buf.put_u8(PING);
                let len = ip_bytes.len();
                buf.put_uint(len as u64, LENGTH_FIELD_LEN);
                buf.put(&ip_bytes[..]); // send the option
            },
            MessageEvent::Pong(addr) => {
                let ip_bytes = match addr.ip() {
                    IpAddr::V4(ip) => ip.octets().to_vec(),
                    IpAddr::V6(ip) => ip.octets().to_vec(),
                };
                buf.reserve(16);
                buf.put_u8(PONG);
                let len = ip_bytes.len();
                buf.put_uint(len as u64, LENGTH_FIELD_LEN);
                buf.put(&ip_bytes[..]); // send the option
            },
            _ => (),
        }
        Ok(())
    }
}

fn bytes_to_ip_addr(src: &mut BytesMut) -> SocketAddr {
    let addr_slice = src.split_to(16);
    let mut addr = [0u8; 16];
    for (x, y) in addr_slice.iter().zip(addr.iter_mut()) {
        *y = *x;
    }
    let ip_addr: IpAddr = addr.into();
    let mut port_slice: &[u8] = &src.split_to(2)[..];
    let port = port_slice.read_u16::<BigEndian>().unwrap() as u16;
    SocketAddr::new(ip_addr, port)
}

impl Decoder for MessageCodec {
    type Item = MessageEvent;
    type Error = CodecError;

    fn decode(&mut self, src: &mut BytesMut) ->
        Result<Option<Self::Item>, Self::Error> {
            let len = src.len();
            let mut byte = src.split_to(1)[0];
            let mut buf: &[u8] = &src.split_to(8)[..];
            let data_len = buf.read_u64::<BigEndian>().unwrap() as usize;

            if len == 0 {
                return Ok(None);
            }

            if src.len() != data_len {
                return Err(Self::Error::DataLengthMismatch);
            }

            match byte {
                PING => {
                    let ip = bytes_to_ip_addr(src);
                    return Ok(Some(MessageEvent::Ping(ip)));
                },
                PONG => {
                    let ip = bytes_to_ip_addr(src);
                    return Ok(Some(MessageEvent::Pong(ip)));
                },
                _ => {

                }
            }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_ip() {
        let localhost_v6 = SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), 8080);
        if let IpAddr::V6(ip) = localhost_v6.clone().ip() {
            let ip = ip.octets();
            let addr: IpAddr = ip.into();
            assert_eq!(SocketAddr::new(addr, 8080), localhost_v6);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_sign() {
        let localhost_v6 = SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), 8000);
        let mut b = BytesMut::new();
        b.put_u8(PING);
        b.put_u64(18);
        b.put_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
        b.put_u16(8000);
        // b.put_slice(&[0, 0, 0, 0]);
        assert_eq!(MessageCodec{}.decode(&mut b).unwrap(), Some(MessageEvent::Ping(localhost_v6)));
    }

}
