use actix::prelude::*;
use actix::io::WriteHandler;
use tokio::net::TcpStream;
use crate::net::{ServerConnection, PacketServerEnum, PacketClientEnum, Protocol, Server};
use crate::net::status::{StatusResponsePacket, PongPacket, StatusRequestPacket};
use crate::net::status::server_status::ServerInfo;
use crate::net::handshake::HandshakePacket;
use crate::net::manager::{HandlerMessage, HandlePacket, ConnectionManager};

/// Manage a connection to a remote server where we act as a client.
///
/// The handler operates on connections of the [Protocol::Status].
/// It requests the [ServerInfo] from the server and
/// once recieved, passes it to the callback function.
pub struct StatusServerManager<F: FnOnce(Result<ServerInfo, ()>) + Unpin + 'static> {
    connection: ServerConnection<Self>,
    /// Callback for the retrieved server info
    callback: Option<F>,
}

impl<F: FnOnce(Result<ServerInfo, ()>) + Unpin + 'static> StatusServerManager<F> {
    pub fn new(stream: TcpStream, ctx: &mut Context<Self>, callback: F) -> StatusServerManager<F> {
        StatusServerManager {
            connection: ServerConnection::new(stream, ctx),
            callback: Some(callback),
        }
    }
}

impl<F: FnOnce(Result<ServerInfo, ()>) + Unpin + 'static> ConnectionManager<Server> for StatusServerManager<F> {}

impl<F: FnOnce(Result<ServerInfo, ()>) + Unpin + 'static> Actor for StatusServerManager<F> {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        let handshake = PacketClientEnum::Handshake(HandshakePacket {
            protocol_version: 57,
            server_address: "127.0.0.1".to_string(),
            server_port: 25565,
            next_protocol: Protocol::Status,
        });

        self.connection.send_packet(handshake)
            .unwrap_or_else(|_| self.connection.disconnect());

        self.connection.set_protocol(Protocol::Status);

        self.connection.send_packet(PacketClientEnum::StatusRequest(StatusRequestPacket))
            .unwrap_or_else(|_| self.connection.disconnect());
    }
}

impl<F: FnOnce(Result<ServerInfo, ()>) + Unpin + 'static> StreamHandler<Result<PacketServerEnum, ()>> for StatusServerManager<F> {
    fn handle(&mut self, packet: Result<PacketServerEnum, ()>, ctx: &mut Self::Context) {
        let handle_result = packet.and_then(|packet| match packet {
            PacketServerEnum::StatusResponse(packet) => self.handle_packet(packet, ctx),
            PacketServerEnum::Pong(packet) => self.handle_packet(packet, ctx),

            _ => Err(()),
        });
        if let Err(()) = handle_result {
            self.connection.disconnect();
        }
    }
}

impl<F: FnOnce(Result<ServerInfo, ()>) + Unpin + 'static> Handler<HandlerMessage<Server>> for StatusServerManager<F> {
    type Result = Result<(), ()>;
    fn handle(&mut self, message: HandlerMessage<Server>, _ctx: &mut Self::Context) -> Self::Result {
        match message {
            HandlerMessage::SendPacket(packet) => self.connection.send_packet(packet),
            HandlerMessage::SetProtocol(protocol) => {
                self.connection.set_protocol(protocol);
                Ok(())
            },
            HandlerMessage::EnableCompression(size_limit) => {
                self.connection.enable_compression(size_limit);
                Ok(())
            },
            HandlerMessage::Disconnect() => {
                self.connection.disconnect();
                Ok(())
            }
        }
    }
}

impl<F: FnOnce(Result<ServerInfo, ()>) + Unpin + 'static> WriteHandler<()> for StatusServerManager<F> {}

impl<F: FnOnce(Result<ServerInfo, ()>) + Unpin + 'static> HandlePacket<Server, StatusResponsePacket> for StatusServerManager<F> {
    fn handle_packet(&mut self, packet: StatusResponsePacket, _ctx: &mut Self::Context) -> Result<(), ()> {
        if let Some(callback) = self.callback.take() {
            callback(Ok(packet.status));
        }

        self.connection.disconnect();
        Ok(())
    }
}

impl<F: FnOnce(Result<ServerInfo, ()>) + Unpin + 'static> HandlePacket<Server, PongPacket> for StatusServerManager<F> {
    fn handle_packet(&mut self, _packet: PongPacket, _ctx: &mut Self::Context) -> Result<(), ()> {
        // Accept ping packets without reacting to them
        Ok(())
    }
}
