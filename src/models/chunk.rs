use std::fmt::{self, Debug};
use serde::{Deserialize, Serialize};


#[derive(Clone)]
pub struct DownloadChunk {
    pub id: u64,
    pub content: [u8; 2048],
    pub index: u16,
}

impl DownloadChunk {
    pub fn new(id: u64, content: [u8; 2048], index: u16) -> Self {
        DownloadChunk {
            id,
            content,
            index,
        }
    }
}

impl Debug for DownloadChunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:?}, {:?})", self.id, self.index)
    }
}

impl PartialEq for DownloadChunk {
    fn eq(&self, other: &Self) -> bool {
        if self.id == other.id {
            true
        } else {
            false
        }
    }
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
pub struct DownloadInfo {
    pub path: String,
    pub size: u16,
}

#[derive(Clone)]
pub struct SongData {
    pub id: u64,
    pub chunks: Vec<Option<[u8; 2048]>>,
    pub filled: u16,
    pub total: u16,
    pub artist: String,
    pub album: String,
    pub filename: String,
}

impl SongData {
    pub fn new(id: u64, total: u16, artist: String, album: String, filename: String) -> Self {
        let mut v = Vec::with_capacity(total as usize);
        for _ in 0..total {
            v.push(None);
        };
        SongData {
            id,
            chunks: v,
            filled: 0u16,
            total,
            artist,
            album,
            filename,
        }
    }

    pub fn add_chunk(&mut self, chunk: DownloadChunk) -> bool {
        if !self.chunks[chunk.index as usize].is_some() {
            self.filled += 1
        };
        self.chunks[chunk.index as usize] = Some(chunk.content);
        if self.filled == self.total {
            true
        } else {
            false
        }
    }
}
