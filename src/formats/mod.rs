pub mod mp3;
pub mod vorbis;

use std::path::Path;
use std::ffi::OsStr;


fn get_extension_from_filename(filename: &str) -> Option<&str> {
    Path::new(filename)
        .extension()
        .and_then(OsStr::to_str)
}

// pub fn get_bitrate(filename: &str) -> Option<u32> {
//     match Path::new(filename)
//         .extension()
//         .and_then(OsStr::to_str) {
//         Some("mp3") => Some(mp3::bitrate(filename)),
//         Some("vorbis") => Some(vorbis::bitrate(filename)),
//         _ => None,
//     }
// }
