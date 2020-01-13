use std::net::SocketAddr;
use clap::{App, Arg};

// TODO: pass initial peers list comma separated

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub config: String,
    pub peers: String,
    pub music: String,
    pub name: String,
    pub initial_peer: Option<Vec<SocketAddr>>,
    pub tui: bool,
}

impl Config {
    pub fn new(
        port: u16,
        config: &str,
        peers: &str,
        music: &str,
        name: &str,
        initial_peer: Option<Vec<SocketAddr>>,
        tui: bool,
    ) -> Self {
        Config {
            port,
            config: config.to_string(),
            peers: peers.to_string(),
            music: music.to_string(),
            name: name.to_string(),
            initial_peer: initial_peer,
            tui,
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
        .arg(Arg::with_name("initial_peers")
            .short("f")
            .long("friends")
            .help("initial peers")
            .takes_value(true))
        .arg(Arg::with_name("name")
            .short("n")
            .long("name")
            .help("username")
            .takes_value(true))
        .arg(Arg::with_name("tui")
            .short("t")
            .long("text_interface")
            .help("text interface boolean")
            .takes_value(false))
        .get_matches();

    let initial_peers = matches.value_of("initial_peers");
    let peers: Option<Vec<SocketAddr>> = if let Some(peers) = initial_peers {
        let peer_vec: Vec<&str> = peers.split(",").collect();
        Some(peer_vec.into_iter().map(|x| x.parse().unwrap()).collect::<Vec<SocketAddr>>())
    } else {
        None
    };

    Config::new(
        value_t!(matches, "port", u16).unwrap_or(8081u16),
        matches.value_of("config").unwrap_or("/tmp/thing.bin"),
        matches.value_of("peers").unwrap_or("/tmp/peers.bin"),
        matches.value_of("music").unwrap_or("/Users/user2/Documents/music"),
        matches.value_of("name").unwrap_or("UNKNOWN_USER"),
        peers,
        matches.is_present("tui"),
    )
}
