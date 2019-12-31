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
    ARTISTS_REQUEST, ALBUM_REQUEST,
    ARTISTS_RESPONSE, ALBUM_RESPONSE,
};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ArtistData {
    artist: Option<String>,
    album: Option<String>,
}

impl ArtistData {
    pub fn new(artist: Option<String>, album: Option<String>) -> ArtistData {
        ArtistData {
            artist,
            album,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct TrackData {
    track: String,
    bitrate: u32,
    length: u32,
}

impl TrackData {
    pub fn new(track: String, bitrate: u32, length: u32) -> TrackData {
        TrackData {
            track,
            bitrate,
            length,
        }
    }
}

const LENGTH_FIELD_LEN: usize = 4;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum MessageEvent {
    Ping(SocketAddr), // add user data
    Pong(SocketAddr), // add user data
    Payload(String),
    Broadcast(String),
    Received(String),
    RequestFile(ArtistData),
    ArtistsRequest,
    ArtistsResponse(Vec<ArtistData>),
    AlbumRequest(ArtistData),
    AlbumResponse(Vec<TrackData>),
    Ok,
    // PeersRequest,
    // PeersResponse(Vec<SocketAddr>),
    Err(MessageCodecError),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum MessageCodecError {
    SerializationError,
    DataLengthMismatch,
    IO,
}

impl From<std::io::Error> for MessageCodecError {
  fn from(err: std::io::Error) -> MessageCodecError {
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

fn get_nstring(src: &mut BytesMut, n: usize) -> Option<String> {
    let target = src.split_to(n);
    if target.len() == 0 {
        return None;
    }
    Some(String::from_utf8_lossy(&target).trim_matches(char::from(0)).to_string())
}

fn take_u64(src: &mut BytesMut) -> Result<u64, MessageCodecError> {
    if src.len() < 8 {
        return Err(MessageCodecError::SerializationError)
    }
    let mut buf: &[u8] = &src.split_to(8)[..];
    Ok(buf.read_u64::<BigEndian>().unwrap())
}

fn take_u32(src: &mut BytesMut) -> Result<u32, MessageCodecError> {
    if src.len() < 4 {
        return Err(MessageCodecError::SerializationError)
    }
    let mut buf: &[u8] = &src.split_to(4)[..];
    Ok(buf.read_u32::<BigEndian>().unwrap())
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
                if let Some(artist) = message.clone().artist {
                    buf.put_u64(artist.len() as u64);
                    buf.put(artist.as_bytes());
                } else {
                    buf.put_u64(0);
                }
                if let Some(album) = message.album {
                    buf.put_u64(album.len() as u64);
                    buf.put(album.as_bytes());
                } else {
                    buf.put_u64(0);
                }
            },
            MessageEvent::ArtistsRequest => {
                buf.put_u8(ARTISTS_REQUEST);
            },
            MessageEvent::ArtistsResponse(artists) => {
                buf.put_u8(ARTISTS_RESPONSE);
                buf.put_u64(artists.len() as u64);
                for artist in artists {
                    if let Some(artist) = artist.artist {
                        buf.put_u64(artist.len() as u64);
                        buf.put(artist.as_bytes());
                    } else {
                        buf.put_u64(0);
                    }

                    if let Some(album) = artist.album {
                        buf.put_u64(album.len() as u64);
                        buf.put(album.as_bytes());
                    } else {
                        buf.put_u64(0);
                    }
                }
            },
            MessageEvent::AlbumRequest(album) => {
                buf.put_u8(ALBUM_REQUEST);
                if let Some(artist) = album.artist {
                    buf.put_u64(artist.len() as u64);
                    buf.put(artist.as_bytes());
                } else {
                    buf.put_u64(0);
                }

                if let Some(album) = album.album {
                    buf.put_u64(album.len() as u64);
                    buf.put(album.as_bytes());
                } else {
                    buf.put_u64(0);
                }
            },
            MessageEvent::AlbumResponse(tracks) => {
                buf.put_u8(ALBUM_RESPONSE);
                buf.put_u64(tracks.len() as u64);
                for track in tracks {
                    buf.put_u64(track.track.len() as u64);
                    buf.put(track.track.as_bytes());
                    buf.put_u32(track.bitrate);
                    buf.put_u32(track.length);
                }
            },
            _ => println!("UNKNOWN!!!"),
        }
        Ok(())
    }
}

fn get_artist_data(src: &mut BytesMut) -> ArtistData {
    let artist_name_len = take_u64(src).unwrap() as usize;
    let artist = get_nstring(src, artist_name_len);
    let album_name_len = take_u64(src).unwrap() as usize;
    let album = get_nstring(src, album_name_len);
    ArtistData::new(
        artist,
        album,
    )
}

fn get_track_data(src: &mut BytesMut) -> TrackData {
    let track_name_len = take_u64(src).unwrap() as usize;
    let track = get_nstring(src, track_name_len).unwrap();
    let bitrate = take_u32(src).unwrap();
    let length = take_u32(src).unwrap();
    TrackData::new(
        track,
        bitrate,
        length,
    )
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

            // let data_len = take_u64(src).unwrap() as usize;
            // if src.len() != data_len {
            //     println!("{:?} {:?}", src.len(), data_len);
            //     return Err(Self::Error::DataLengthMismatch);
            // }
            // // drop trailing bytes
            // src.split_off(data_len);

            match byte {
                PING => {
                    let data_len = take_u64(src).unwrap() as usize;
                    let ip = bytes_to_ip_addr(src);
                    return Ok(Some(MessageEvent::Ping(ip)));
                },
                PONG => {
                    let data_len = take_u64(src).unwrap() as usize;
                    let ip = bytes_to_ip_addr(src);
                    return Ok(Some(MessageEvent::Pong(ip)));
                },
                PAYLOAD => {
                    let data_len = take_u64(src).unwrap() as usize;
                    let message = get_string(src);
                    return Ok(Some(MessageEvent::Payload(message)));
                },
                RECEIVED => {
                    let data_len = take_u64(src).unwrap() as usize;
                    let message = get_string(src);
                    return Ok(Some(MessageEvent::Received(message)));
                },
                REQUEST_FILE => {
                    return Ok(Some(MessageEvent::RequestFile(
                        get_artist_data(src)
                    )));
                },
                ARTISTS_REQUEST => {
                    return Ok(Some(MessageEvent::ArtistsRequest));
                },
                ARTISTS_RESPONSE => {
                    let mut artist_count = take_u64(src).unwrap() as usize;
                    let mut artist_vec: Vec<ArtistData> = vec![];
                    while artist_count > 0 {
                        let artist = get_artist_data(src);
                        artist_vec.push(artist);
                        artist_count -= 1;
                    }
                    return Ok(Some(MessageEvent::ArtistsResponse(artist_vec)));
                },
                ALBUM_REQUEST => {
                    return Ok(Some(MessageEvent::AlbumRequest(
                        get_artist_data(src)
                    )));
                },
                ALBUM_RESPONSE => {
                    let mut track_count = take_u64(src).unwrap() as usize;
                    let mut track_vec: Vec<TrackData> = vec![];
                    while track_count > 0 {
                        let track = get_track_data(src);
                        track_vec.push(track);
                        track_count -= 1;
                    }
                    return Ok(Some(MessageEvent::AlbumResponse(track_vec)));
                },
                _ => {}
            }
            Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_artist_request() {
        let artist_request = MessageEvent::AlbumRequest(ArtistData::new(
            Some(String::from("test1")),
            Some(String::from("test2")),
        ));
        let mut res = BytesMut::new();
        // u8 would be big enough
        MessageCodec{}.encode(
                artist_request.clone(), &mut res).unwrap();
        assert_eq!(MessageCodec{}.decode(&mut res).unwrap().unwrap(), artist_request);
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
