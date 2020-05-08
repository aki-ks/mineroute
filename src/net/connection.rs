use std::rc::Rc;
use std::sync::RwLock;
use actix::AsyncContext;
use actix::io::SinkWrite;
use tokio::net::TcpStream;
use crate::net::{Protocol, ConnectionType};
use crate::net::pipeline::{HandlerPipeline, PipelineSink, PipelineStream};
use crate::net::manager::ConnectionManager;

/// A connection to a remote minecraft server or client.
///
/// This struct gets encapsulated by a corresponding manager
/// actor in whose asynchronity context it will run and notifies
/// it about all incoming packets.
///
/// It exposes an interface allowing the manager actor to send
/// packets to the server and execute protocol changes.
pub struct Connection<CT: ConnectionType> {
    sink: SinkWrite<CT::Out, PipelineSink<CT>>,
    pipeline: Rc<RwLock<HandlerPipeline<CT>>>,
}

impl<CT: ConnectionType> Connection<CT> {
    pub fn new<H: ConnectionManager<CT>>(stream: TcpStream, ctx: &mut H::Context) -> Connection<CT> {
        let pipeline = Rc::new(RwLock::new(HandlerPipeline::new(stream)));

        ctx.add_stream(PipelineStream::new(pipeline.clone()));

        let sink = PipelineSink::new(pipeline.clone());
        let sink = SinkWrite::new(sink, ctx);

        Connection {
            pipeline,
            sink,
        }
    }

    pub fn send_packet(&mut self, packet: CT::Out) -> Result<(), ()> {
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
