// /// https://docs.rs/puremp3/0.1.0/puremp3/
// extern crate puremp3;
//
//
// pub fn validate(filename: &str) -> bool {
//     let data = std::fs::read(filename).expect("Could not open file");
//     let song_file = puremp3::read_mp3(&data[..]);
//     match song_file {
//         Ok(_)  => true,
//         Err(_) => false,
//     }
// }
//
// pub fn bitrate(filename: &str) -> u32 {
//     let data = std::fs::read(filename).expect("Could not open file");
//     let (header, _) = puremp3::read_mp3(&data[..]).expect("invalid mp3");
//     println!("{:?}", header);
//     header.bitrate.bps()
// }
//
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_mp3() {
//         assert_eq!(validate("/Users/user2/Documents/music/Jeromes Dream/Completed 1997-2001/07 His Life Is My Denim Paradise All Day, Every Day.mp3"), true);
//         assert_eq!(validate("/Users/user2/development/airlink/assets/beep.mp3"), true);
//         assert_eq!(validate("/Users/user2/development/first.txt"), false);
//         assert_eq!(bitrate("/Users/user2/development/airlink/assets/beep.mp3"), 128000);
//     }
// }
