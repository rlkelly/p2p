use bytes::Bytes;
use std::sync::Arc;
use std::error::Error;
use std::collections::HashMap;
use std::pin::Pin;
use std::net::SocketAddr;
use std::task::{Context, Poll};

use futures::SinkExt;
use tokio::prelude::*;
use tokio::sync::{mpsc, Mutex};
use tokio::stream::{self, Stream, StreamExt};
use tokio_util::codec::{Framed, LengthDelimitedCodec, LinesCodec, LinesCodecError};

use tokio::net::{
    TcpListener,
    TcpStream,
};

use music_snobster::organizer::get_collection;
use music_snobster::codec::MessageEvent;

type Tx = mpsc::UnboundedSender<String>;
type Rx = mpsc::UnboundedReceiver<String>;

struct Shared {
    peers: HashMap<SocketAddr, Tx>,
}

impl Shared {
    fn new() -> Self {
        Shared { peers: HashMap::new() }
    }
    async fn broadcast(&mut self, sender: SocketAddr, message: &str) {
        for peer in self.peers.iter_mut() {
            if *peer.0 != sender {
                let _ = peer.1.send(message.into());
            }
        }
    }
}

struct Peer {
    lines: Framed<TcpStream, LinesCodec>,
    rx: Rx,
}

impl Peer {
    async fn new(
        state: Arc<Mutex<Shared>>,
        lines: Framed<TcpStream, LinesCodec>,
    ) -> io::Result<Peer> {
        let addr = lines.get_ref().peer_addr()?;
        let (tx, rx) = mpsc::unbounded_channel();

        state.lock().await.peers.insert(addr, tx);
        Ok(Peer { lines, rx })
    }
}

impl Stream for Peer {
    type Item = Result<MessageEvent, LinesCodecError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Poll::Ready(Some(v)) = Pin::new(&mut self.rx).poll_next(cx) {
            return Poll::Ready(Some(Ok(MessageEvent::Received(v))));
        }

        let result: Option<_> = futures::ready!(Pin::new(&mut self.lines).poll_next(cx));

        // publish to all peers
        Poll::Ready(match result {
            Some(Ok(message)) => Some(Ok(MessageEvent::Broadcast(message))),
            Some(Err(e)) => Some(Err(e)),
            None => None,
        })
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(Mutex::new(Shared::new()));
    let mut listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (stream, addr) = listener.accept().await?;
        let state = Arc::clone(&state);
        // let db = db.clone();
        tokio::spawn(async move {
            // let (reader, mut writer) = stream.split();
            let mut buf = bytes::BytesMut::with_capacity(10);

            let mut file = tokio::fs::File::open("/Users/user2/Documents/music/Jeromes Dream/Completed 1997-2001/12 How Staggering Is This Realization.mp3").await.unwrap();
            let mut transport = Framed::new(stream, LengthDelimitedCodec::new());
            file.read_buf(&mut buf).await.unwrap();
            transport.send(buf.clone().freeze()).await.unwrap();
            println!("The bytes: {:?}", &buf[..]);
            buf.clear();

            use std::io::SeekFrom;
            file.seek(SeekFrom::Start(1)).await;

            file.read_buf(&mut buf).await.unwrap();
            transport.send(buf.clone().freeze()).await.unwrap();
            println!("The bytes: {:?}", &buf[..]);
            // tokio::io::copy(&mut file_buf, &mut writer).await;
            // writer.write(b"end").await;


            // if let Err(e) = process(state, stream, addr).await {
            //     println!("an error occured; error = {:?}", e);
            // }
        });
    }
}

async fn process(
    state: Arc<Mutex<Shared>>,
    stream: TcpStream,
    addr: SocketAddr,
) -> Result<(), Box<dyn Error>> {
    let mut lines = Framed::new(stream, LinesCodec::new());
    lines
        .send(String::from("Please enter your username:"))
        .await?;

    let username = match lines.next().await {
        Some(Ok(line)) => line,
        _ => {
            println!("Failed to get username from {}. Client disconnected.", addr);
            return Ok(());
        }
    };

    let mut peer = Peer::new(state.clone(), lines).await?;
    {
        let mut state = state.lock().await;
        let msg = format!("{} has joined the chat", username);
        println!("{} {}", msg, addr);
        state.broadcast(addr, &msg).await;
    }

    while let Some(result) = peer.next().await {
        match result {
            Ok(MessageEvent::Broadcast(msg)) => {
                let mut state = state.lock().await;
                let msg = format!("{}: {}", username, msg);

                state.broadcast(addr, &msg).await;
            }
            Ok(MessageEvent::Received(msg)) => {
                peer.lines.send(msg).await?;
            },
            Err(e) => {
                println!(
                    "an error occured while processing messages for {}; error = {:?}",
                    username, e
                );
            },
            _ => println!("unknown"),
        }
    }
    {
        let mut state = state.lock().await;
        state.peers.remove(&addr);

        let msg = format!("{} has left the chat", username);
        println!("{}", msg);
        state.broadcast(addr, &msg).await;
    }
    Ok(())
}

// format!("{:?}", get_collection("/Users/user2/Documents/music", false, filter)).as_bytes()
// socket.write_all("pong".as_bytes()).await;
