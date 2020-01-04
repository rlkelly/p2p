use std::convert::TryInto;

use tokio_util::codec::{Decoder, Encoder};
use bytes::{BytesMut, BufMut};
use byteorder::{BigEndian, ReadBytesExt};

use std::str;
use std::net::{
    SocketAddr,
    IpAddr,
};
use serde::{Deserialize, Serialize};
use crate::models::{
    ArtistData,
    AlbumData,
    TrackData,
    take_u64,
    get_nstring,
};
use crate::consts::*;

const LENGTH_FIELD_LEN: usize = 4;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Peer {
    addr: SocketAddr,
    name: Option<String>,
    public_key: Option<String>,
    signature: Option<String>, // sign to prove they have private key
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

impl Peer {
    pub fn new(addr: SocketAddr, name: Option<String>, public_key: Option<String>, signature: Option<String>) -> Self {
        Peer {
            addr,
            name,
            public_key,
            signature,
        }
    }

    pub fn to_bytes(&self) -> BytesMut {
        let mut buf = BytesMut::new();
        let ip_bytes = match self.addr.ip() {
            IpAddr::V4(ip) => ip.octets().to_vec(),
            IpAddr::V6(ip) => ip.octets().to_vec(),
        };
        let len = ip_bytes.len();
        buf.put_uint(len as u64, LENGTH_FIELD_LEN);
        buf.put(&ip_bytes[..]); // send the option

        // factor this out
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
        let _ip_len = take_u64(buf).unwrap();
        let addr = bytes_to_ip_addr(buf);
        let name_key = buf.split_to(1)[0] as usize;
        let name = get_nstring(buf, name_key);
        let pk_key = buf.split_to(1)[0] as usize;
        let public_key = get_nstring(buf, pk_key);
        let signature_key = buf.split_to(1)[0] as usize;
        let signature = get_nstring(buf, signature_key);
        Peer {
            addr,
            name,
            public_key,
            signature,
        }

    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum MessageEvent {
    Ping(Peer), // add user data
    Pong(Peer), // add user data
    Payload(String),
    Broadcast(String),
    Received(String),
    RequestFile(ArtistData),
    ArtistsRequest,
    ArtistsResponse(Vec<ArtistData>),
    AlbumRequest(AlbumData),
    AlbumResponse(Vec<TrackData>),
    Ok,
    PeersRequest,
    PeersResponse(Vec<Peer>),
    Err(MessageCodecError),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum MessageCodecError {
    SerializationError,
    DataLengthMismatch,
    IO,
}

impl From<std::io::Error> for MessageCodecError {
  fn from(_err: std::io::Error) -> MessageCodecError {
    MessageCodecError::IO
  }
}

impl PartialEq for MessageCodecError {
  fn eq(&self, other: &Self) -> bool {
    match (&self, &other) {
      (MessageCodecError::IO, MessageCodecError::IO) => true,
      _ => false
    }
  }
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
            MessageEvent::Ping(peer) => {
                buf.extend_from_slice(&peer.to_bytes()[..])
            },
            MessageEvent::Pong(peer) => {
                buf.extend_from_slice(&peer.to_bytes()[..])
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
            MessageEvent::RequestFile(artist_data) => {
                buf.put_u8(REQUEST_FILE);
                buf.extend_from_slice(&artist_data.to_bytes()[..])
            },
            MessageEvent::ArtistsRequest => {
                buf.put_u8(ARTISTS_REQUEST);
            },
            MessageEvent::ArtistsResponse(artists) => {
                buf.put_u8(ARTISTS_RESPONSE);
                buf.put_u64(artists.len() as u64);
                for artist in artists {
                    buf.extend_from_slice(&artist.to_bytes()[..]);
                }
            },
            MessageEvent::AlbumRequest(album) => {
                buf.put_u8(ALBUM_REQUEST);
                buf.extend_from_slice(&mut album.to_bytes()[..]);
            },
            MessageEvent::AlbumResponse(tracks) => {
                buf.put_u8(ALBUM_RESPONSE);
                for track in tracks {
                    buf.extend_from_slice(&track.to_bytes()[..]);
                };
            },
            MessageEvent::PeersRequest => {
                buf.put_u8(PEERS_REQUEST);
            },
            MessageEvent::PeersResponse(peers) => {
                buf.put_u8(PEERS_RESPONSE);
                buf.put_u64(peers.len() as u64);
                for peer in peers {
                    let bytes = peer.to_bytes();
                    buf.put_u64(bytes.len() as u64);
                    buf.extend_from_slice(&bytes[..]);
                };
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

            let byte = src.split_to(1)[0];

            // let data_len = take_u64(src).unwrap() as usize;
            // if src.len() != data_len {
            //     println!("{:?} {:?}", src.len(), data_len);
            //     return Err(Self::Error::DataLengthMismatch);
            // }
            // // drop trailing bytes
            // src.split_off(data_len);

            match byte {
                PING => {
                    let _data_len = take_u64(src).unwrap() as usize;
                    let ip = bytes_to_ip_addr(src);
                    return Ok(Some(MessageEvent::Ping(Peer::new(
                        ip,
                        None,
                        None,
                        None,
                    ))));
                },
                PONG => {
                    let _data_len = take_u64(src).unwrap() as usize;
                    let ip = bytes_to_ip_addr(src);
                    return Ok(Some(MessageEvent::Pong(Peer::new(
                        ip,
                        None,
                        None,
                        None,
                    ))));
                },
                PAYLOAD => {
                    let data_len = take_u64(src).unwrap() as usize;
                    let message = get_nstring(src, data_len).unwrap();
                    return Ok(Some(MessageEvent::Payload(message)));
                },
                RECEIVED => {
                    let data_len = take_u64(src).unwrap() as usize;
                    let message = get_nstring(src, data_len).unwrap();
                    return Ok(Some(MessageEvent::Received(message)));
                },
                REQUEST_FILE => {
                    return Ok(Some(MessageEvent::RequestFile(
                        ArtistData::from_bytes(src)
                    )));
                },
                ARTISTS_REQUEST => {
                    return Ok(Some(MessageEvent::ArtistsRequest));
                },
                ARTISTS_RESPONSE => {
                    // ArtistsResponse(Vec<ArtistData>),
                    let mut artist_count = take_u64(src).unwrap() as usize;
                    let mut artist_vec: Vec<ArtistData> = vec![];
                    while artist_count > 0 {
                        let artist = ArtistData::from_bytes(src);
                        artist_vec.push(artist);
                        artist_count -= 1;
                    }
                    return Ok(Some(MessageEvent::ArtistsResponse(artist_vec)));
                },
                ALBUM_REQUEST => {
                    // AlbumRequest(ArtistData),
                    return Ok(Some(MessageEvent::AlbumRequest(
                        AlbumData::from_bytes(src)
                    )));
                },
                ALBUM_RESPONSE => {
                    // AlbumResponse(Vec<TrackData>),
                    let mut track_count = take_u64(src).unwrap() as usize;
                    let mut track_vec: Vec<TrackData> = vec![];
                    while track_count > 0 {
                        let track: TrackData = src.into();
                        track_vec.push(track);
                        track_count -= 1;
                    }
                    return Ok(Some(MessageEvent::AlbumResponse(track_vec)));
                },
                PEERS_REQUEST => {
                    return Ok(Some(MessageEvent::PeersRequest));
                },
                PEERS_RESPONSE => {
                    // TODO: parse vector into bytes
                    let mut peer_count = take_u64(src).unwrap() as usize;
                    let mut peer_vec: Vec<Peer> = vec![];
                    while peer_count > 0 {
                        let peer: Peer = Peer::from_bytes(src);
                        peer_vec.push(peer);
                        peer_count -= 1;
                    }

                    return Ok(Some(MessageEvent::PeersResponse(vec![])));
                },
                _ => {}
            }
            Ok(None)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv6Addr;

    #[test]
    fn test_serialize_album_request() {
        let mut res = BytesMut::new();
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
        let album_request = MessageEvent::AlbumRequest(AlbumData::new(
            Some("test2".to_string()),
            "test3".to_string(),
            0,
            Some(vec![TrackData::new("test".to_string(), 12_000, 250)]),
        ));
        MessageCodec{}.encode(album_request.clone(), &mut res).unwrap();
        let left = MessageCodec{}.decode(&mut res).unwrap().unwrap();
        assert_eq!(left, album_request);
    }

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
        assert_eq!(MessageCodec{}.decode(&mut b).unwrap(), Some(MessageEvent::Ping(Peer::new(localhost_v6, None, None, None))));
    }

    #[test]
    fn test_encode_pong() {
        let localhost_v6 = SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), 8000);
        let mut b = BytesMut::new();
        b.put_u8(PONG);
        b.put_u64(18);
        b.put_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
        b.put_u16(8000);
        b.put_u8(0);
        b.put_u8(0);
        b.put_u8(0);
        assert_eq!(MessageCodec{}.decode(&mut b).unwrap(), Some(MessageEvent::Pong(Peer::new(localhost_v6, None, None, None))));
    }

    #[test]
    fn test_encode_payload() {
        let mut b = BytesMut::new();
        b.put_u8(PAYLOAD);
        b.put_u64(12);
        b.put(&b"hello world\0"[..]);
        assert_eq!(MessageCodec{}.decode(&mut b).unwrap(), Some(MessageEvent::Payload(String::from("hello world"))));

        let mut res = BytesMut::new();
        MessageCodec{}.encode(MessageEvent::Payload(String::from("hello world\0")), &mut res).unwrap();

        let mut b = BytesMut::new();
        b.put_u8(PAYLOAD);
        b.put_u64(12);
        b.put(&b"hello world\0"[..]);
        assert_eq!(res[..], b[..]);
    }
}
