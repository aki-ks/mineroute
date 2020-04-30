use std::pin::Pin;
use std::rc::Rc;
use std::sync::RwLock;
use tokio::io::{split, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use crate::net::{Protocol, ConnectionType};
use crate::net::pipeline::packet_codec::PacketCodec;
use crate::net::pipeline::compressor::Compressor;

mod stream;
mod sink;
mod framing;
mod compressor;
mod packet_codec;

pub use self::sink::PipelineSink;
pub use self::stream::PipelineStream;

pub struct HandlerPipeline<C: ConnectionType> {
    r: Pin<Box<ReadHalf<TcpStream>>>,
    w: Pin<Box<WriteHalf<TcpStream>>>,
    protocol: Rc<RwLock<Protocol>>,
    compressor: Option<Compressor>,
    codec: PacketCodec<C>,
}

impl<C: ConnectionType> HandlerPipeline<C> {
    pub fn new(stream: TcpStream) -> HandlerPipeline<C> {
        let (r, w) = split(stream);
        let protocol = Rc::new(RwLock::new(Protocol::Handshake));
        HandlerPipeline {
            r: Box::pin(r),
            w: Box::pin(w),
            protocol: protocol.clone(),
            compressor: None,
            codec: PacketCodec::new(protocol.clone()),
        }
    }

    pub fn set_protocol(&mut self, proto: Protocol) {
        *self.protocol.write().unwrap() = proto;
    }

    pub fn enable_compression(&mut self, size_limit: Option<usize>) {
        self.compressor = size_limit.map(|size_limit| Compressor { size_limit });
    }
}
