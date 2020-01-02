use std::io::Read;
use std::fs::File;

use crypto::sha2::Sha256;

use crate::tree_utils::{
    Stack,
};

pub fn chunk_file(filename: &str) -> Vec<Vec<u8>> {
    let mut file = std::fs::File::open(filename).unwrap();
    let mut list_of_chunks = Vec::new();
    let chunk_size = 0x4000;

    loop {
        let mut chunk = Vec::with_capacity(chunk_size);
        let n = file.by_ref().take(chunk_size as u64).read_to_end(&mut chunk).unwrap();
        if n == 0 { break; }
        list_of_chunks.push(chunk);
        if n < chunk_size { break; }
    }
    list_of_chunks
}

pub fn get_root(filename: &str) -> String {
    let mut file = match File::open(filename) {
        Ok(file) => file,
        Err(..)  => panic!("couldn't open file"),
    };

    let mut s = Stack::new(Sha256::new());
    s.read_from(&mut file);

    s.root()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign() {
        assert_ne!(chunk_file("./static/01 - mirror.mp3"), vec!([1]));
    }

    #[test]
    fn test_root() {
        use std::env;
        println!("{:?}", env::current_dir().unwrap());
        let chunks = get_root("./static/01 - mirror.mp3");
        let chunks2 = get_root("./static/01 - mirror.mp3");
        assert_eq!(chunks, chunks2);
    }

    #[test]
    fn test_root_change() {
        assert_ne!(
            get_root("./static/first.txt"),
            get_root("./static/second.txt"),
        );
    }
}
