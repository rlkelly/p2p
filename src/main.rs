use std::sync::Arc;

use tokio::prelude::*;
use tokio::sync::Mutex;
use tokio::stream::StreamExt;
use tokio_util::codec::{BytesCodec, FramedRead};

use tokio::net::TcpListener;

use music_snobster::handlers::{process, Service};
use music_snobster::args::get_args;

// TODO: handle requests
//   - SEND FILE
//   - REQUEST FILE
//   - DOWNLOAD FILE
// Integrate World State into Service
//   - Remove Peer
// Allow for local client to query data

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = get_args();
    println!("{:?}", config);
    let state = Arc::new(Mutex::new(Service::new()));
    let mut listener = TcpListener::bind(format!("127.0.0.1:{}", config.port)).await?;

    // listen for local commands
    tokio::spawn(async move {
        println!("start local listener");
        loop {
            let mut stdin = FramedRead::new(io::stdin(), BytesCodec::new());
            while let Some(item) = stdin.next().await {
                println!("{:?}", item);
            }
        }
    });

    loop {
        println!("start server");
        let (stream, addr) = listener.accept().await?;
        let state = Arc::clone(&state);

        tokio::spawn(async move {
            if let Err(e) = process(state, stream, addr).await {
                println!("an error occured; error = {:?}", e);
            }
        });
    }
}
