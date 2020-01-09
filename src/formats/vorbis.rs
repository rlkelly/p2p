/// https://github.com/RustAudio/lewton

extern crate lewton;

use std::fs::File;
use lewton::inside_ogg::OggStreamReader;

#[allow(dead_code)]
pub fn validate(file_path: &str) -> bool {
    let f = File::open(file_path).expect("Can't open file");

    let srr = OggStreamReader::new(f);
    match srr {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[allow(dead_code)]
pub fn bitrate(file_path: &str) -> u32 {
    let f = File::open(file_path).expect("Can't open file");
	let srr = OggStreamReader::new(f).unwrap();

    srr.ident_hdr.audio_sample_rate
}
