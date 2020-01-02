use std::fs::metadata;

use crate::formats::mp3::{
    get_mp3_data,
    // MusicFileData,
};

use crate::models::{
    ArtistData,
    AlbumData,
    TrackData,
};


pub fn get_collection(dir_name: &str, track_data: bool, maybe_artist_filter: Option<&str>) -> Vec<ArtistData> {
    let mut artist_vec: Vec<ArtistData> = Vec::with_capacity(2048);
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
            let mut album_vec: Vec<AlbumData> = vec!();
            let albums = std::fs::read_dir(path.clone()).unwrap();
            for album in albums {
                let entry = album.unwrap();
                let path = entry.path();
                if metadata(path.clone()).unwrap().is_dir() {
                    let album_name = path.file_stem().unwrap().to_str().unwrap();
                    let mut track_vec: Vec<TrackData> = vec![];

                    let tracks = std::fs::read_dir(path.clone()).unwrap();
                    let mut track_count: u8 = 0;
                    for track in tracks {
                        let entry = track.unwrap();
                        let path = entry.path();
                        let mp3_data = match get_mp3_data(&path) {
                            Ok(data) => data,
                            _ => continue,
                        };
                        if track_data {
                            track_vec.push(TrackData::new(
                                mp3_data.title,
                                mp3_data.bitrate,
                                0,
                            ));
                            track_count += 1;
                        }
                    }
                    let tracks = if track_data {
                        Some(track_vec)
                    } else {
                        None
                    };
                    album_vec.push(AlbumData::new(
                        None,
                        album_name.to_string(),
                        track_count,
                        tracks,
                    ));
                }
            }
            let artist = ArtistData::new(artist_name.into(), Some(album_vec));
            artist_vec.push(artist);
        }
    }
    artist_vec
}


#[cfg(test)]
mod tests {
    use super::*;
    use dirs::home_dir;

    #[test]
    fn test_folder() {
        let home_dir = home_dir().unwrap().into_os_string().into_string().unwrap();
        get_collection(&format!("{}/{}", home_dir, "Documents/music"), true, None);
        get_collection(&format!("{}/{}", home_dir, "Documents/music"), false, None);
    }

    fn test_fake_arist() -> String {
        let artist = ArtistData::new(
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
        serde_json::to_string(&artist).unwrap()
    }
}
