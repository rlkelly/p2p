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
use tokio_util::codec::{BytesCodec, FramedRead, Framed, LengthDelimitedCodec, LinesCodec, LinesCodecError};

use tokio::net::{
    TcpListener,
    TcpStream,
};

use music_snobster::organizer::get_collection;
use music_snobster::codec::{
    MessageEvent,
    MessageCodec,
    MessageCodecError,
};

type Tx = mpsc::UnboundedSender<String>;
type Rx = mpsc::UnboundedReceiver<String>;

struct Shared {
    peers: HashMap<SocketAddr, Tx>,
}

impl Shared {
    fn new() -> Self {
        Shared { peers: HashMap::new() }
    }
}

impl Shared {
    async fn broadcast(&mut self, sender: SocketAddr, message: &str) {
        for peer in self.peers.iter_mut() {
            if *peer.0 != sender {
                let _ = peer.1.send(message.into());
            }
        }
    }
}

struct Peer {
    messages: Framed<TcpStream, MessageCodec>,
    rx: Rx,
}

impl Peer {
    async fn new(
        state: Arc<Mutex<Shared>>,
        messages: Framed<TcpStream, MessageCodec>,
    ) -> io::Result<Peer> {
        let addr = messages.get_ref().peer_addr()?;
        let (tx, rx) = mpsc::unbounded_channel();
        state.lock().await.peers.insert(addr, tx);
        Ok( Peer { messages, rx })
    }
}

impl Stream for Peer {
    type Item = Result<MessageEvent, MessageCodecError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Poll::Ready(Some(v)) = Pin::new(&mut self.rx).poll_next(cx) {
            return Poll::Ready(Some(Ok(MessageEvent::Received(v))));
        }

        let result: Option<_> = futures::ready!(Pin::new(&mut self.messages).poll_next(cx));
        Poll::Ready(match result {
            Some(Ok(message)) => Some(Ok(message)),
            Some(Err(e)) => Some(Err(e)),
            None => None,
        })
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(Mutex::new(Shared::new()));
    let mut listener = TcpListener::bind("127.0.0.1:8080").await?;

    // listen for local commands
    tokio::spawn(async move {
        println!("start local listener");
        println!("connected");
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
            // let mut file = tokio::fs::File::open("/Users/user2/Documents/music/Jeromes Dream/Completed 1997-2001/12 How Staggering Is This Realization.mp3").await.unwrap();
            // file.read_buf(&mut buf).await.unwrap();
            // use std::io::SeekFrom;
            // file.seek(SeekFrom::Start(1)).await;
            // tokio::io::copy(&mut file_buf, &mut writer).await;
            // writer.write(b"end").await;

            if let Err(e) = process(state, stream, addr).await {
                println!("an error occured; error = {:?}", e);
            }
        });
    }
}

async fn process(
    state: Arc<Mutex<Shared>>,
    stream: TcpStream,
    addr: SocketAddr,
) -> Result<(), Box<dyn Error>> {
    let mut transport = Framed::new(stream, MessageCodec::new());
    let mut peer = Peer::new(state.clone(), transport).await?;
    {
        let mut state = state.lock().await;
        let msg = "broadcast test0";
        state.broadcast(addr, &msg).await;
    }

    while let Some(result) = peer.next().await {
        println!("result: {:?}", result);
        match result {
            Ok(MessageEvent::Broadcast(msg)) => {
                let mut state = state.lock().await;
                let msg = "broadcast test1";

                state.broadcast(addr, &msg).await;
            },
            Ok(MessageEvent::Payload(msg)) => {
                let mut state = state.lock().await;
                let msg = "broadcast test2";

                state.broadcast(addr, &msg).await;
            },
            Ok(MessageEvent::Received(msg)) => {
                peer.messages.send(MessageEvent::Payload(msg)).await.unwrap();
            },
            Err(e) => {
                println!(
                    "an error occured while processing messages; error = {:?}", e
                );
            },
            _ => println!("UNK"),
        }
    }
    {
        let mut state = state.lock().await;
        state.peers.remove(&addr);
        let msg = "broadcast test3";
        state.broadcast(addr, &msg).await;
    }
    Ok(())
}
