use bytes::{BytesMut, BufMut};
use std::convert::TryInto;

use std::str;
use serde::{Deserialize, Serialize};

use super::utils::{
    take_u16,
    take_u64,
    get_nstring,
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
    bitrate: u16,
    length: u16,
}

impl TrackData {
    pub fn new(title: String, bitrate: u16, length: u16) -> TrackData {
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
        buf.put_u16(self.bitrate);
        buf.put_u16(self.length);
        buf
    }
}

impl From<&mut BytesMut> for TrackData {
    fn from(buf: &mut BytesMut) -> TrackData {
        let track_name_len = take_u64(buf).unwrap() as usize;
        let track = get_nstring(buf, track_name_len).unwrap();
        let bitrate: u16 = take_u16(buf).unwrap();
        let length: u16 = take_u16(buf).unwrap();
        TrackData::new(
            track,
            bitrate,
            length,
        )
    }
}
