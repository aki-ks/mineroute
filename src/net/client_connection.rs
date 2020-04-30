use std::rc::Rc;
use std::sync::RwLock;
use actix::{Addr, AsyncContext};
use actix::io::SinkWrite;
use tokio::net::TcpStream;
use crate::net::{PacketServerEnum, Client, Protocol};
use crate::net::pipeline::{HandlerPipeline, PipelineStream, PipelineSink};
use crate::net::manager::ConnectionManager;

/// A connection to a minecraft client held by a server
pub struct ClientConnection<H: ConnectionManager<Client>> {
    pipeline: Rc<RwLock<HandlerPipeline<Client>>>,
    sink: SinkWrite<PacketServerEnum, PipelineSink<Client>>,
    #[allow(unused)]
    addr: Addr<H>,
}

impl<H: ConnectionManager<Client>> ClientConnection<H> {
    pub fn new(stream: TcpStream, ctx: &mut H::Context) -> ClientConnection<H> {
        let pipeline = Rc::new(RwLock::new(HandlerPipeline::new(stream)));

        ctx.add_stream(PipelineStream::new(pipeline.clone()));

        let sink = PipelineSink::new(pipeline.clone());
        let sink = SinkWrite::new(sink, ctx);

        return ClientConnection {
            pipeline: pipeline.clone(),
            sink,
            addr: ctx.address().clone(),
        };
    }

    pub fn send_packet(&mut self, packet: PacketServerEnum) -> Result<(), ()> {
        self.sink.write(packet)
    }

    pub fn set_protocol(&self, protocol: Protocol) {
        self.pipeline.write().unwrap().set_protocol(protocol);
    }

    pub fn enable_compression(&mut self, size_limit: Option<usize>) {
        self.pipeline.write().unwrap().enable_compression(size_limit)
    }

    pub fn disconnect(&mut self) {
        self.sink.close()
    }
}
