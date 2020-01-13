use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;

use super::{DownloadChunk, SongData};

// type DownloadMemory<'a> = HashMap<u64<'a>, SongData<'a>>;
pub struct DownloadMemory {
    songs: HashMap<u64, SongData>,
    path: String,
}

impl DownloadMemory {
    pub fn new(path: String) -> Self {
        DownloadMemory {
            songs: HashMap::new(),
            path,
        }
    }

    pub fn add_song(&mut self, song_data: SongData) {
        self.songs.insert(song_data.id, song_data);
    }

    pub fn get_song(&self, song_id: u64) -> &SongData {
        self.songs.get(&song_id).unwrap()
    }

    pub fn add_chunk(&mut self, chunk: DownloadChunk) {
        let chunk_id = chunk.id.clone();
        // TODO: handle file deletion
        let song = self.songs.get_mut(&chunk_id).expect("Song doesnt exist in memory");
        let full = song.add_chunk(chunk);

        if full {
            self.assemble_song(chunk_id);
        }
    }

    pub fn assemble_song(&mut self, id: u64) {
        let song = self.songs.get(&id).unwrap();
        let directory = format!("./{}/{}/{}", self.path, song.artist, song.album);

        fs::create_dir_all(directory.clone()).unwrap();

        let mut buffer = File::create(format!("{}/{}", directory, song.filename)).expect("File create error");
        let mut index = 1;
        // TODO: this is a kludge
        for chunk in song.chunks.iter() {
            if index == song.total {
                let mut vec = chunk.unwrap().to_vec();
                vec = vec.into_iter().rev().skip_while(|&x| x == 0).collect();
                vec.reverse();
                buffer.write(&vec[..]).expect("write failed");
            } else {
                buffer.write(&chunk.expect("these should all be files")[..]).expect("write failed");
            };
            index += 1;
        }
        self.songs.remove(&id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use read_byte_slice::{ByteSliceIter, FallibleStreamingIterator};

    #[test]
    fn test_assemble_song() {
        let mut dm = DownloadMemory::new("static".to_string());
        let sd = SongData::new(1, 4, "artist".to_string(), "album".to_string(), "filename".to_string());
        dm.add_song(sd);

        for i in 0..4 {
            let dc = DownloadChunk::new(1, [0u8; 2048], i);
            dm.add_chunk(dc);
        }
        fs::remove_dir_all("./static/artist").unwrap();
    }

    #[test]
    fn test_buffer_write_song() {
        let mut dm = DownloadMemory::new("static".to_string());
        let sd = SongData::new(1, 1699, "artist".to_string(), "album".to_string(), "mirror.mp3".to_string());
        dm.add_song(sd);

        let file = File::open("./static/01 - mirror.mp3").unwrap();
        let mut iter = ByteSliceIter::new(file, 2048);
        let mut chunk_count = 0;
        while let Ok(Some(chunk)) = iter.next() {
            let mut c = [0u8; 2048];
            let len = chunk.len();
            c[..len].clone_from_slice(&chunk);
            let dc = DownloadChunk::new(1, c, chunk_count);
            dm.add_chunk(dc);
            chunk_count += 1;
        }
        fs::remove_dir_all("./static/artist").unwrap();
    }
}
