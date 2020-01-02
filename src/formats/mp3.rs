use std::path::Path;
use std::fmt;
use serde::{Deserialize, Serialize};

extern crate mp3_metadata;


#[derive(Debug)]
pub enum FileError {
    InvalidFormat,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MusicFileData {
    path: String,
    version: String,
    layer: String,
    bitrate: u16,
    sampling_freq: u16,
    channel_type: String,
    title: String,
    artist: String,
    album: String,
    year: u16,
    genre: String,
    hash: String,
}

impl MusicFileData {
    pub fn new() -> MusicFileData {
        MusicFileData {
            path: "a".to_string(),
            version: "a".to_string(),
            layer: "a".to_string(),
            bitrate: 12600,
            sampling_freq: 12600,
            channel_type: "a".to_string(),
            title: "a".to_string(),
            artist: "a".to_string(),
            album: "a".to_string(),
            year: 2019,
            genre: "a".to_string(),
            hash: "".to_string(),
        }
    }
}


pub fn get_mp3_data(file: &Path) -> Result<MusicFileData, FileError> {
    let meta = match mp3_metadata::read_from_file(file) {
        Ok(data) => data,
        Err(_) => return Err(FileError::InvalidFormat),
    };

    let tag = match meta.tag {
        Some(t) => t,
        _ => {
            mp3_metadata::AudioTag{
                title: "".into(),
                artist: "".into(),
                album: "".into(),
                year: 0,
                comment: "".into(),
                genre: mp3_metadata::Genre::Unknown,
            }
        },
    };
    let frame = &meta.frames[0];

    let version = fmt::format(format_args!("{:?}", frame.version));
    let chan_type = fmt::format(format_args!("{:?}", frame.chan_type));
    let layer = fmt::format(format_args!("{:?}", frame.layer));
    let genre = fmt::format(format_args!("{:?}", tag.genre));
    let title = tag.title.trim_matches(char::from(0));
    let artist = tag.artist.trim_matches(char::from(0));
    let album = tag.album.trim_matches(char::from(0));

    let mp3_data = MusicFileData {
        path: file.to_str().unwrap().to_string(),
        version: version,
        layer: layer,
        bitrate: frame.bitrate,
        sampling_freq: frame.sampling_freq,
        channel_type: chan_type,
        title: title.to_string(),
        artist: artist.to_string(),
        album: album.to_string(),
        year: tag.year,
        genre: genre,
        hash: "".to_string(),
    };
    Ok(mp3_data)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mp3() {
        const MP3_FILE: &str = "./static/01 - mirror.mp3";
        let _valid = get_mp3_data(Path::new(MP3_FILE)).unwrap();
    }
}
