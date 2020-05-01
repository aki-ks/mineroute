mod proxy_client_manager;
mod status_server_manager;
mod proxy_server_manager;

pub use status_server_manager::StatusServerManager;
pub use proxy_client_manager::ProxyClientManager;
pub use proxy_server_manager::ProxyServerManager;

use actix::{Actor, Context, StreamHandler, Handler, Message};
use actix::io::WriteHandler;
use crate::net::{Packet, Protocol, ConnectionType};

/// Manage a connection between a server/client setup
pub trait ConnectionManager<CT: ConnectionType> where
    Self: Actor<Context = Context<Self>>,
    Self: StreamHandler<Result<CT::In, ()>>,
    Self: Handler<HandlerMessage<CT>>,
    Self: WriteHandler<()> {}

/// Handle an incoming packet retrieved from a server/client.
pub trait HandlePacket<CT: ConnectionType, P: Packet>: ConnectionManager<CT> {
    fn handle_packet(&mut self, packet: P, ctx: &mut Self::Context) -> Result<(), ()>;
}

/// Messages supported by [ConnectionManager] actors
pub enum HandlerMessage<CT: ConnectionType> {
    SendPacket(CT::Out),
    SetProtocol(Protocol),
    EnableCompression(Option<usize>),
    Disconnect(),
}
impl<CT: ConnectionType> Message for HandlerMessage<CT> {
    type Result = Result<(), ()>;
}
