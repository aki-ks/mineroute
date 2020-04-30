use std::rc::Rc;
use std::sync::RwLock;
use actix::{Addr, AsyncContext};
use actix::io::SinkWrite;
use tokio::net::TcpStream;
use crate::net::{Server, PacketClientEnum, Protocol};
use crate::net::pipeline::{HandlerPipeline, PipelineSink, PipelineStream};
use crate::net::manager::ConnectionManager;

/// A connection to a remote minecraft server held when connected to a server as client.
pub struct ServerConnection<H: ConnectionManager<Server>> {
    sink: SinkWrite<PacketClientEnum, PipelineSink<Server>>,
    pipeline: Rc<RwLock<HandlerPipeline<Server>>>,
    #[allow(unused)]
    addr: Addr<H>,
}

impl<H: ConnectionManager<Server>> ServerConnection<H> {
    pub fn new(stream: TcpStream, ctx: &mut H::Context) -> ServerConnection<H> {
        let pipeline = Rc::new(RwLock::new(HandlerPipeline::new(stream)));

        ctx.add_stream(PipelineStream::new(pipeline.clone()));

        let sink = PipelineSink::new(pipeline.clone());
        let sink = SinkWrite::new(sink, ctx);

        ServerConnection {
            pipeline,
            sink,
            addr: ctx.address().clone(),
        }
    }

    pub fn send_packet(&mut self, packet: PacketClientEnum) -> Result<(), ()> {
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
