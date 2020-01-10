use clap::{App, Arg};

#[derive(Debug)]
pub struct Config {
    pub port: u16,
    config: String,
    peers: String,
    music: String,
}

impl Config {
    pub fn new(port: u16, config: &str, peers: &str, music: &str) -> Self {
        Config {
            port,
            config: config.to_string(),
            peers: peers.to_string(),
            music: music.to_string(),
        }
    }
}

pub fn get_args() -> Config {
    let matches = App::new("Music Sharing")
        .version("0.1")
        .author("Robert Kelly <robert.l.kelly3@gmail.com>")
        .about("Share mp3s")
        .arg(Arg::with_name("port")
            .help("sets the port")
            .index(1))
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("FILE")
            .help("Sets a custom config file")
            .takes_value(true))
        .arg(Arg::with_name("peers")
            .short("p")
            .long("peer")
            .value_name("FILE")
            .help("Set the peers config file")
            .takes_value(true))
        .arg(Arg::with_name("music")
            .short("m")
            .long("music")
            .value_name("DIRECTORY")
            .help("where your music collection lives")
            .takes_value(true))
        .get_matches();

    Config::new(
        value_t!(matches, "port", u16).unwrap_or(8081u16),
        matches.value_of("config").unwrap_or("/tmp/config.yaml"),
        matches.value_of("peers").unwrap_or("/tmp/peers.yaml"),
        matches.value_of("music").unwrap_or("~/Documents/music"),
    )
}