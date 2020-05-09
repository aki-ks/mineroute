use std::rc::Rc;
use std::sync::RwLock;
use tokio::io::split;
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

/// This struct encapsulates
pub struct HandlerPipeline<C: ConnectionType> {
    protocol: Rc<RwLock<Protocol>>,
    compressor: Option<Compressor>,
    codec: PacketCodec<C>,
}

impl<C: ConnectionType> HandlerPipeline<C> {
    pub fn new(stream: TcpStream) -> (Rc<RwLock<HandlerPipeline<C>>>, PipelineSink<C>, PipelineStream<C>) {
        let (r, w) = split(stream);
        let protocol = Rc::new(RwLock::new(Protocol::Handshake));
        let pipeline = Rc::new(RwLock::new(HandlerPipeline {
            protocol: protocol.clone(),
            compressor: None,
            codec: PacketCodec::new(protocol.clone()),
        }));

        let stream = PipelineStream::new(r, pipeline.clone());
        let sink = PipelineSink::new(w, pipeline.clone());

        (pipeline, sink, stream)
    }

    pub fn set_protocol(&mut self, proto: Protocol) {
        *self.protocol.write().unwrap() = proto;
    }

    pub fn enable_compression(&mut self, size_limit: Option<usize>) {
        self.compressor = size_limit.map(|size_limit| Compressor { size_limit });
    }
}
