use actix::prelude::*;
use actix::io::WriteHandler;
use tokio::net::TcpStream;
use crate::net::{ServerConnection, PacketServerEnum, Protocol, Server, Client};
use crate::net::manager::{HandlerMessage, HandlePacket, ConnectionManager};
use crate::net::login::{CompressionPacket, LoginSuccessPacket};
use futures::FutureExt;

/// Manage a connection to a remote server in which we act as client.
/// The received packets are proxied to some other client.
pub struct ProxyServerManager<C: ConnectionManager<Client>> {
    /// proxy packets received from the server to this client
    downstream: Addr<C>,

    /// The connection to the remove server
    connection: ServerConnection<Self>,
}

impl<C: ConnectionManager<Client>> ProxyServerManager<C> {
    pub fn new(downstream: Addr<C>, stream: TcpStream, ctx: &mut Context<Self>) -> ProxyServerManager<C> {
        ProxyServerManager {
            downstream,
            connection: ServerConnection::new(stream, ctx),
        }
    }
}

impl<C: ConnectionManager<Client>> ConnectionManager<Server> for ProxyServerManager<C> {}

impl<C: ConnectionManager<Client>> Actor for ProxyServerManager<C> {
    type Context = Context<Self>;
}

impl<C: ConnectionManager<Client>> StreamHandler<Result<PacketServerEnum, ()>> for ProxyServerManager<C> {
    fn handle(&mut self, packet: Result<PacketServerEnum, ()>, ctx: &mut Self::Context) {
        let handle_result = packet.and_then(|packet| {
            self.downstream.send(HandlerMessage::SendPacket(packet.clone()))
                .map(|_| ()).into_actor(self).wait(ctx);

            match packet {
                PacketServerEnum::StatusResponse(_) => Ok(()),
                PacketServerEnum::Pong(_) => Ok(()),

                PacketServerEnum::Disconnect(_) => Ok(()),
                PacketServerEnum::Compression(packet) => self.handle_packet(packet, ctx),
                PacketServerEnum::LoginSuccess(packet) => self.handle_packet(packet, ctx),

                PacketServerEnum::Raw(_) => Ok(()),
            }
        });

        if let Err(()) = handle_result {
            self.connection.disconnect();
        }
    }
}

impl<C: ConnectionManager<Client>> Handler<HandlerMessage<Server>> for ProxyServerManager<C> {
    type Result = Result<(), ()>;
    fn handle(&mut self, message: HandlerMessage<Server>, _ctx: &mut Self::Context) -> Self::Result {
        match message {
            HandlerMessage::SendPacket(packet) => {
                self.connection.send_packet(packet)
            },
            HandlerMessage::SetProtocol(protocol) => {
                self.connection.set_protocol(protocol);
                Ok(())
            },
            HandlerMessage::EnableCompression(size_limit) => {
                self.connection.enable_compression(size_limit);
                Ok(())
            },
        }
    }
}

impl<C: ConnectionManager<Client>> WriteHandler<()> for ProxyServerManager<C> {}

impl<C: ConnectionManager<Client>> HandlePacket<Server, CompressionPacket> for ProxyServerManager<C> {
    fn handle_packet(&mut self, packet: CompressionPacket, ctx: &mut Self::Context) -> Result<(), ()> {
        self.connection.enable_compression(packet.size_limit);
        self.downstream.send(HandlerMessage::EnableCompression(packet.size_limit))
            .map(|_| ()).into_actor(self).wait(ctx);

        Ok(())
    }
}

impl<C: ConnectionManager<Client>> HandlePacket<Server, LoginSuccessPacket> for ProxyServerManager<C> {
    fn handle_packet(&mut self, _packet: LoginSuccessPacket, ctx: &mut Self::Context) -> Result<(), ()> {
        self.connection.set_protocol(Protocol::Play);
        self.downstream.send(HandlerMessage::SetProtocol(Protocol::Play))
            .map(|_| ()).into_actor(self).wait(ctx);

        Ok(())
    }
}
