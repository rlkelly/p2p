use tokio_util::codec::{Decoder, Encoder};
use bytes::{BytesMut, BufMut};
use byteorder::{BigEndian, ReadBytesExt};

use std::str;
use std::net::{
    SocketAddr,
    IpAddr,
};
use serde::{Deserialize, Serialize};

use crate::consts::{
    PING, PONG, PAYLOAD, RECEIVED, REQUEST_FILE,
    ARTISTS_REQUEST, ALBUM_REQUEST,
    ARTISTS_RESPONSE, ALBUM_RESPONSE,
};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ArtistData {
    artist: String,
    albums: Option<Vec<AlbumData>>,
}

impl ArtistData {
    pub fn new(artist: String, albums: Option<Vec<AlbumData>>) -> ArtistData {
        ArtistData {
            artist,
            albums,
        }
    }

    pub fn to_bytes(&self) -> BytesMut {
        let mut buf = BytesMut::new();
        buf.put_u64(self.artist.len() as u64);
        buf.put(self.artist.as_bytes());
        if let Some(albums) = &self.albums {
            buf.put_u64(albums.len() as u64);
            for album in albums {
                buf.extend_from_slice(&album.to_bytes()[..]);
            }
        } else {
            buf.put_u64(0);
        };
        buf
    }

    pub fn from_bytes(buf: &mut BytesMut) -> ArtistData {
        let artist_name_len = take_u64(buf).unwrap() as usize;
        let artist = get_nstring(buf, artist_name_len).unwrap();
        let mut album_count = take_u64(buf).unwrap();
        let mut album_vec: Vec<AlbumData> = vec![];

        let albums = if album_count > 0 {
            while album_count > 0 {
                let album = AlbumData::from_bytes(buf);
                album_vec.push(album);
                album_count -= 1;
            }
            Some(album_vec)
        } else {
            None
        };

        ArtistData::new(
            artist,
            albums,
        )
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct AlbumData {
    artist: Option<String>,
    album_title: String,
    track_count: u8,
    tracks: Option<Vec<TrackData>>,
}

impl AlbumData {
    pub fn new(artist: Option<String>, album_title: String, track_count: u8, tracks: Option<Vec<TrackData>>) -> AlbumData {
        AlbumData {
            artist,
            album_title,
            track_count,
            tracks,
        }
    }

    pub fn to_bytes(&self) -> BytesMut {
        let mut buf = BytesMut::new();
        if let Some(artist) = &self.artist {
            buf.put_u64(artist.len() as u64);
            buf.put(artist.as_bytes());
        } else {
            buf.put_u64(0);
        }
        buf.put_u64(self.album_title.len() as u64);
        buf.put(self.album_title.as_bytes());
        buf.put_u8(self.track_count);

        match &self.tracks {
            Some(tracks) => {
                buf.put_u8(1);
                buf.put_u64(tracks.len() as u64);
                for track in tracks {
                    buf.extend_from_slice(&track.to_bytes()[..]);
                }
            },
            None => {
                buf.put_u8(0);
            }
        };
        buf
    }

    pub fn from_bytes(buf: &mut BytesMut) -> AlbumData {
        let artist_name_len = take_u64(buf).unwrap() as usize;
        let artist = get_nstring(buf, artist_name_len);
        let album_name_len = take_u64(buf).unwrap() as usize;
        let album = get_nstring(buf, album_name_len).unwrap();
        let track_count = buf.split_to(1)[0];
        let get_tracks = buf.split_to(1)[0];

        let tracks = if get_tracks == 1 {
            let mut tracks = vec![];

            let mut track_count = take_u64(buf).unwrap();
            while track_count > 0 {
                let track: TrackData = buf.into();
                tracks.push(track);
                track_count -= 1;
            }
            Some(tracks)
        } else {
            None
        };

        AlbumData::new(
            artist,
            album,
            track_count,
            tracks,
        )

    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct TrackData {
    title: String,
    bitrate: u32,
    length: u32,
}

impl TrackData {
    pub fn new(title: String, bitrate: u32, length: u32) -> TrackData {
        TrackData {
            title,
            bitrate,
            length,
        }
    }

    pub fn to_bytes(&self) -> BytesMut {
        let mut buf = BytesMut::new();
        buf.put_u64(self.title.len() as u64);
        buf.put(self.title.as_bytes());
        buf.put_u32(self.bitrate);
        buf.put_u32(self.length);
        buf
    }
}

impl From<&mut BytesMut> for TrackData {
    fn from(buf: &mut BytesMut) -> TrackData {
        let track_name_len = take_u64(buf).unwrap() as usize;
        let track = get_nstring(buf, track_name_len).unwrap();
        let bitrate = take_u32(buf).unwrap();
        let length = take_u32(buf).unwrap();
        TrackData::new(
            track,
            bitrate,
            length,
        )
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

fn get_nstring(src: &mut BytesMut, n: usize) -> Option<String> {
    if n == 0 {
        return None;
    };
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
                }
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
                    return Ok(Some(MessageEvent::Ping(ip)));
                },
                PONG => {
                    let _data_len = take_u64(src).unwrap() as usize;
                    let ip = bytes_to_ip_addr(src);
                    return Ok(Some(MessageEvent::Pong(ip)));
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
                        ArtistData::from_bytes(src)
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
        let album_request = MessageEvent::AlbumRequest(ArtistData::new(
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
        MessageCodec{}.encode(MessageEvent::Payload(String::from("hello world\0")), &mut res).unwrap();

        let mut b = BytesMut::new();
        b.put_u8(PAYLOAD);
        b.put_u64(12);
        b.put(&b"hello world\0"[..]);
        assert_eq!(res[..], b[..]);
    }
}
