/// https://github.com/libp2p/rust-libp2p ???
/// find peers
/// add peer
/// drop peer
/// get user's file list
/// ask if user has file

use super::service::{Service, Rx};

use std::sync::Arc;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::SinkExt;
use futures::sink::Send;
use tokio::prelude::*;
use tokio::sync::{mpsc, Mutex};
use tokio::stream::Stream;
use tokio_util::codec::Framed;

use tokio::net::TcpStream;

use crate::codec::{
    MessageEvent,
    MessageCodec,
    MessageCodecError,
};

// TODO: rename this?
pub struct PeerConnection {
    messages: Framed<TcpStream, MessageCodec>,
    rx: Rx,
}

impl PeerConnection {
    pub async fn new(
        state: Arc<Mutex<Service>>,
        messages: Framed<TcpStream, MessageCodec>,
    ) -> io::Result<PeerConnection> {
        let addr = messages.get_ref().peer_addr()?;
        let (tx, rx) = mpsc::unbounded_channel();
        state.lock().await.peers.insert(addr, tx);
        Ok( PeerConnection { messages, rx })
    }

    pub fn send_message(&mut self, message: MessageEvent)
        -> Send<'_, Framed<TcpStream, MessageCodec>, MessageEvent>
    {
        self.messages.send(message)
    }
}

impl Stream for PeerConnection {
    type Item = Result<MessageEvent, MessageCodecError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Poll::Ready(Some(v)) = Pin::new(&mut self.rx).poll_next(cx) {
            return Poll::Ready(Some(Ok(v)));
        }

        let result: Option<_> = futures::ready!(Pin::new(&mut self.messages).poll_next(cx));
        Poll::Ready(match result {
            Some(Ok(message)) => Some(Ok(message)),
            Some(Err(e)) => Some(Err(e)),
            None => None,
        })
    }
}
