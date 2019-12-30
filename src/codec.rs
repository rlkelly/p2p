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
    PING, PONG, PAYLOAD, RECEIVED, REQUEST_FILE,
};


const LENGTH_FIELD_LEN: usize = 4;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum MessageEvent {
    Ping(SocketAddr), // add user data
    Pong(SocketAddr), // add user data
    Payload(String),
    Broadcast(String),
    Received(String),
    RequestFile(String),
    // AddrVec(Vec<SocketAddr>),
    // ArtistRequest(String),
    // AlbumRequest(String),
    // ArtistResponse(String),
    // AlbumResponse(String),
    // PeersRequest(String),
    // PeersResponse(String),
    Ok,
}

#[derive(Debug)]
pub enum MessageCodecError {
    SerializationError,
    DataLengthMismatch,
    IO(std::io::Error),
}

impl From<std::io::Error> for MessageCodecError {
  fn from(err: std::io::Error) -> MessageCodecError {
    MessageCodecError::IO(err)
  }
}

impl PartialEq for MessageCodecError {
  fn eq(&self, other: &Self) -> bool {
    match (&self, &other) {
      (MessageCodecError::IO(a), MessageCodecError::IO(b)) => a.kind() == b.kind(),
      _ => false
    }
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

fn get_string(src: &mut BytesMut) -> String {
    String::from_utf8_lossy(src).trim_matches(char::from(0)).to_string()
}

pub struct MessageCodec {}

impl MessageCodec {
    pub fn new() -> MessageCodec {
        MessageCodec {}
    }
}

impl Encoder for MessageCodec {
  type Item = MessageEvent;
  type Error = MessageCodecError;

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
                buf.put(&ip_bytes[..]);
            },
            MessageEvent::Payload(message) => {
                buf.put_u8(PAYLOAD);
                let bytes = message.as_bytes();
                buf.put_u64(bytes.clone().len() as u64);
                buf.put(bytes);
            },
            MessageEvent::Received(message) => {
                buf.put_u8(RECEIVED);
                let bytes = message.as_bytes();
                buf.put_u64(bytes.clone().len() as u64);
                buf.put(bytes);
            },
            MessageEvent::RequestFile(message) => {
                buf.put_u8(REQUEST_FILE);
                let bytes = message.as_bytes();
                buf.put_u64(bytes.clone().len() as u64);
                buf.put(bytes);
            },
            MessageEvent::Received(message) => {
                buf.put_u8(REQUEST_FILE);
                let bytes = message.as_bytes();
                buf.put_u64(bytes.clone().len() as u64);
                buf.put(bytes);
            },
            _ => println!("UNKNOWN!!!"),
        }
        Ok(())
    }
}

impl Decoder for MessageCodec {
    type Item = MessageEvent;
    type Error = MessageCodecError;

    fn decode(&mut self, src: &mut BytesMut) ->
        Result<Option<Self::Item>, Self::Error> {
            let len = src.len();
            if len == 0 {
                return Ok(None);
            }

            let mut byte = src.split_to(1)[0];
            if src.len() < 8 {
                src.clear();
                return Ok(None);
            }
            let mut buf: &[u8] = &src.split_to(8)[..];
            let data_len = buf.read_u64::<BigEndian>().unwrap() as usize;
            if src.len() != data_len {
                println!("{:?} {:?}", src.len(), data_len);
                return Err(Self::Error::DataLengthMismatch);
            }

            let data = src.split_to(data_len);

            match byte {
                PING => {
                    let ip = bytes_to_ip_addr(src);
                    return Ok(Some(MessageEvent::Ping(ip)));
                },
                PONG => {
                    let ip = bytes_to_ip_addr(src);
                    return Ok(Some(MessageEvent::Pong(ip)));
                },
                PAYLOAD => {
                    let message = get_string(src);
                    return Ok(Some(MessageEvent::Payload(message)));
                },
                RECEIVED => {
                    let message = get_string(src);
                    return Ok(Some(MessageEvent::Received(message)));
                },
                // REQUEST_FILE => {
                //     let message = get_string(src);
                //     return Ok(Some(MessageEvent::RequestFile(message)));
                // }
                _ => {}
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
    fn test_encode_ping() {
        let localhost_v6 = SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), 8000);
        let mut b = BytesMut::new();
        b.put_u8(PING);
        b.put_u64(18);
        b.put_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
        b.put_u16(8000);
        assert_eq!(MessageCodec{}.decode(&mut b).unwrap(), Some(MessageEvent::Ping(localhost_v6)));
    }

    #[test]
    fn test_encode_pong() {
        let localhost_v6 = SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), 8000);
        let mut b = BytesMut::new();
        b.put_u8(PONG);
        b.put_u64(18);
        b.put_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
        b.put_u16(8000);
        assert_eq!(MessageCodec{}.decode(&mut b).unwrap(), Some(MessageEvent::Pong(localhost_v6)));
    }

    #[test]
    fn test_encode_payload() {
        let mut b = BytesMut::new();
        b.put_u8(PAYLOAD);
        b.put_u64(12);
        b.put(&b"hello world\0"[..]);
        assert_eq!(MessageCodec{}.decode(&mut b).unwrap(), Some(MessageEvent::Payload(String::from("hello world"))));

        let mut res = BytesMut::new();
        MessageCodec{}.encode(MessageEvent::Payload(String::from("hello world\0")), &mut res);

        let mut b = BytesMut::new();
        b.put_u8(PAYLOAD);
        b.put_u64(12);
        b.put(&b"hello world\0"[..]);
        assert_eq!(res[..], b[..]);
    }
}
