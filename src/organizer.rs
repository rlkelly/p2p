use std::fs::metadata;
use serde::{Deserialize, Serialize};

use crate::formats::mp3::{
    get_mp3_data,
    MusicFileData,
};


#[derive(Debug, Serialize, Deserialize)]
pub struct Artist {
    name: String,
    albums: Vec<Album>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Album {
    name: String,
    tracks: Vec<MusicFileData>,
    track_count: usize,
}

impl Album {
    pub fn new(name: &str, tracks: Vec<MusicFileData>, track_count: usize) -> Album {
        Album {
            name: name.to_string(),
            tracks: tracks,
            track_count: track_count,
        }
    }
}

impl Artist {
    pub fn new(name: &str, albums: Vec<Album>) -> Artist {
        Artist {
            name: name.to_string(),
            albums: albums,
        }
    }
}

pub fn get_collection(dir_name: &str, track_data: bool, maybe_artist_filter: Option<&str>) -> Vec<Artist> {
    let mut artist_vec: Vec<Artist> = vec!();

    let artist: Vec<Artist> = Vec::with_capacity(1024);
    let artists = std::fs::read_dir(dir_name).unwrap();

    for artist in artists {
        let entry = artist.unwrap();
        let path = entry.path();
        let artist_name = path.file_stem().unwrap().to_str().unwrap();
        if let Some(artist_filter) = maybe_artist_filter {
            if artist_name != artist_filter {
                continue
            }
        }
        if metadata(path.clone()).unwrap().is_dir() {
            let mut album_vec: Vec<Album> = vec!();
            let albums = std::fs::read_dir(path.clone()).unwrap();
            for album in albums {
                let entry = album.unwrap();
                let path = entry.path();
                if metadata(path.clone()).unwrap().is_dir() {
                    let album_name = path.file_stem().unwrap().to_str().unwrap();
                    let mut track_vec: Vec<MusicFileData> = vec!();

                    let tracks = std::fs::read_dir(path.clone()).unwrap();
                    let mut track_count: usize = 0;
                    for track in tracks {
                        let entry = track.unwrap();
                        let path = entry.path();
                        let mp3_data = match get_mp3_data(&path) {
                            Ok(data) => data,
                            _ => continue,
                        };
                        if track_data {
                            track_vec.push(mp3_data);
                        }
                        track_count += 1;
                    }
                    album_vec.push(Album::new(
                        album_name,
                        track_vec,
                        track_count,
                    ));
                }
            }
            let artist = Artist::new(artist_name, album_vec);
            artist_vec.push(artist);
        }
    }
    artist_vec
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_folder() {
        let home_dir = env::home_dir().unwrap().into_os_string().into_string().unwrap();
        get_collection(&format!("{}/{}", home_dir, "/Documents/music"), true, None);
        get_collection(&format!("{}/{}", home_dir, "/Documents/music"), false, None);
    }

    fn fake_arist() -> String {
        let track1 = MusicFileData::new();
        let track2 = MusicFileData::new();
        let album1 = Album {
            name: "album".to_string(),
            tracks: vec!(track1, track2),
            track_count: 3,
        };
        let artist = Artist {
            name: "artist".to_string(),
            albums: vec!(album1),
        };
        serde_json::to_string(&artist).unwrap()
    }

    #[test]
    fn test_sign() {
        assert_eq!(
            fake_arist(),
            r#"{"name":"artist","albums":[{"name":"album","tracks":[{"path":"a","version":"a","layer":"a","bitrate":12600,"sampling_freq":12600,"channel_type":"a","title":"a","artist":"a","album":"a","year":2019,"genre":"a","hash":""},{"path":"a","version":"a","layer":"a","bitrate":12600,"sampling_freq":12600,"channel_type":"a","title":"a","artist":"a","album":"a","year":2019,"genre":"a","hash":""}],"track_count":3}]}"#
        )
    }
}
