use bytes::{BytesMut, BufMut};
use std::str;
use serde::{Deserialize, Serialize};
use tokio_util::codec::{Decoder, Encoder};

use crate::models::{
    ArtistData,
    AlbumData,
    DownloadChunk,
    Peer,
    take_u64,
    get_nstring,
};
use crate::consts::*;

#[derive(Clone, Debug, PartialEq)]
pub enum MessageEvent {
    Ping(Peer), // add user data
    Pong(Peer), // add user data
    Payload(String),
    Broadcast(String),
    RequestFile(ArtistData),
    ArtistsRequest,
    ArtistsResponse(Vec<ArtistData>),
    AlbumRequest(AlbumData),
    AlbumResponse(AlbumData),
    PeersRequest,
    PeersResponse(Vec<Peer>),
    DownloadRequest(AlbumData),
    DownloadChunk(DownloadChunk),
    Err(MessageCodecError),
    Ok,
}

// // TODO: I think this will make it clearer who is sending
// pub struct MessageBody {
//     message: MessageEvent,
//     // name: String,
//     pub_key: String,
//     signature: String,
// }

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

  fn encode(&mut self, event: Self::Item, src: &mut BytesMut) ->
    Result<(), Self::Error> {
        let mut buf = BytesMut::new();
        match event {
            MessageEvent::Ping(peer) => {
                buf.put_u8(PING);
                buf.extend_from_slice(&peer.to_bytes()[..]);
            },
            MessageEvent::Pong(peer) => {
                buf.put_u8(PONG);
                buf.extend_from_slice(&peer.to_bytes()[..]);
            },
            MessageEvent::Payload(message) => {
                buf.put_u8(PAYLOAD);
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
            MessageEvent::AlbumResponse(album) => {
                buf.put_u8(ALBUM_RESPONSE);
                buf.extend_from_slice(&mut album.to_bytes()[..]);
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
            MessageEvent::DownloadRequest(album) => {
                buf.put_u8(DOWNLOAD_REQUEST);
                buf.extend_from_slice(&mut album.to_bytes()[..]);
            },
            MessageEvent::Ok => {
                buf.put_u8(OK);
            },
            _ => println!("UNKNOWN!!!"),
        }
        src.put_u64(buf.len() as u64);
        src.extend_from_slice(&buf[..]);
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
            let bytes_len = take_u64(src).expect("invalid message");
            let data = &mut src.split_to(bytes_len as usize);

            // TODO: validate data length
            let byte = data.split_to(1)[0];

            match byte {
                PING => {
                    let peer = Peer::from_bytes(data);
                    return Ok(Some(MessageEvent::Ping(peer)));
                },
                PONG => {
                    let peer = Peer::from_bytes(data);
                    return Ok(Some(MessageEvent::Pong(peer)));
                },
                PAYLOAD => {
                    let data_len = take_u64(data).unwrap() as usize;
                    let message = get_nstring(data, data_len).unwrap();
                    return Ok(Some(MessageEvent::Payload(message)));
                },
                REQUEST_FILE => {
                    return Ok(Some(MessageEvent::RequestFile(
                        ArtistData::from_bytes(data)
                    )));
                },
                ARTISTS_REQUEST => {
                    return Ok(Some(MessageEvent::ArtistsRequest));
                },
                ARTISTS_RESPONSE => {
                    let mut artist_count = take_u64(data).unwrap() as usize;
                    let mut artist_vec: Vec<ArtistData> = vec![];
                    while artist_count > 0 {
                        let artist = ArtistData::from_bytes(data);
                        artist_vec.push(artist);
                        artist_count -= 1;
                    }
                    return Ok(Some(MessageEvent::ArtistsResponse(artist_vec)));
                },
                ALBUM_REQUEST => {
                    return Ok(Some(MessageEvent::AlbumRequest(
                        AlbumData::from_bytes(data)
                    )));
                },
                ALBUM_RESPONSE => {
                    return Ok(Some(MessageEvent::AlbumRequest(
                        AlbumData::from_bytes(data)
                    )));
                },
                PEERS_REQUEST => {
                    return Ok(Some(MessageEvent::PeersRequest));
                },
                PEERS_RESPONSE => {
                    // TODO: parse vector into bytes
                    let mut peer_count = take_u64(data).unwrap() as usize;
                    let mut peer_vec: Vec<Peer> = vec![];
                    while peer_count > 0 {
                        // TODO: take len
                        let _len = take_u64(data);
                        let peer: Peer = Peer::from_bytes(data);
                        peer_vec.push(peer);
                        peer_count -= 1;
                    }

                    return Ok(Some(MessageEvent::PeersResponse(peer_vec)));
                },
                DOWNLOAD_REQUEST => {
                    return Ok(Some(MessageEvent::DownloadRequest(
                        AlbumData::from_bytes(data)
                    )));
                },
                OK => {
                    println!("OK!");
                    return Ok(Some(MessageEvent::Ok))
                },
                _ => {}
            }
            Ok(None)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ArtistData, AlbumData, TrackData};
    use std::net::{IpAddr, Ipv6Addr, SocketAddr};

    #[test]
    fn test_artists_request() {
        let ad = ArtistData {
            artist: "my_artist".to_string(),
            albums: Some(vec![
                AlbumData::new(
                    Some("test1".to_string()),
                    "test1-1".to_string(),
                    1,
                    Some(vec![TrackData::new("test".to_string(), 12_000, 250)]),
                ),
                AlbumData::new(
                    Some("test4".to_string()),
                    "test4-4".to_string(),
                    1,
                    Some(vec![TrackData::new("test".to_string(), 12_000, 250)]),
                ),
            ]),
        };
        let mut res = BytesMut::new();
        let ad_vec = vec![ad.clone(), ad.clone()];
        let artist_request = MessageEvent::ArtistsResponse(ad_vec);
        MessageCodec{}.encode(artist_request.clone(), &mut res).unwrap();
        assert_eq!(MessageCodec{}.decode(&mut res).unwrap(), Some(artist_request));
    }

    #[test]
    fn test_serialize_album_request() {
        let mut res = BytesMut::new();
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
        b.put_u64(31);
        b.put_u8(PING);
        b.put_u64(16);
        b.put_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
        b.put_u16(8000);
        b.put_u8(0);
        b.put_u8(0);
        b.put_u8(0);
        b.put_u8(0);
        assert_eq!(MessageCodec{}.decode(&mut b).unwrap(), Some(MessageEvent::Ping(Peer::new(localhost_v6, false, None, None, None))));
    }

    #[test]
    fn test_encode_pong() {
        let localhost_v6 = SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), 8000);
        let mut b = BytesMut::new();
        b.put_u64(31);
        b.put_u8(PONG);
        b.put_u64(16);
        b.put_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
        b.put_u16(8000);
        b.put_u8(0);
        b.put_u8(0);
        b.put_u8(0);
        b.put_u8(0);
        assert_eq!(MessageCodec{}.decode(&mut b).unwrap(), Some(MessageEvent::Pong(Peer::new(localhost_v6, false, None, None, None))));
    }

    #[test]
    fn test_encode_payload() {
        let mut b = BytesMut::new();
        b.put_u64(21);
        b.put_u8(PAYLOAD);
        b.put_u64(12);
        b.put(&b"hello world\0"[..]);
        assert_eq!(MessageCodec{}.decode(&mut b).unwrap(), Some(MessageEvent::Payload(String::from("hello world"))));

        let mut res = BytesMut::new();
        MessageCodec{}.encode(MessageEvent::Payload(String::from("hello world\0")), &mut res).unwrap();

        let mut b = BytesMut::new();
        b.put_u64(21);
        b.put_u8(PAYLOAD);
        b.put_u64(12);
        b.put(&b"hello world\0"[..]);
        assert_eq!(res[..], b[..]);
    }
}
