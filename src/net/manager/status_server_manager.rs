use std::net::SocketAddr;
use actix::prelude::*;
use actix::io::WriteHandler;
use tokio::net::TcpStream;
use futures::channel::oneshot::{channel, Sender};
use crate::net::{Connection, PacketServerEnum, PacketClientEnum, Protocol, Server};
use crate::net::status::{StatusResponsePacket, PongPacket, StatusRequestPacket, server_status};
use crate::net::status::server_status::ServerInfo;
use crate::net::handshake::HandshakePacket;
use crate::net::manager::{HandlerMessage, HandlePacket, ConnectionManager};

/// Manage a connection to a remote server where we act as a client.
///
/// The handler operates on connections of the [Protocol::Status].
/// It requests the [ServerInfo] from the server and
/// once recieved, passes it to the callback function.
pub struct StatusServerManager {
    connection: Connection<Server>,
    sender: Option<Sender<Result<ServerInfo, ()>>>,
}

impl StatusServerManager {
    /// Connect to a minecraft server at the provided address,
    /// fetch its server state and return it asynchronously.
    pub async fn fetch_status(addr: SocketAddr) -> Result<server_status::ServerInfo, ()> {
        let stream = TcpStream::connect(addr).await.map_err(|_| ())?;
        let (sender, receiver) = channel::<Result<ServerInfo, ()>>();
        StatusServerManager::create(|ctx| {
            StatusServerManager {
                connection: Connection::new::<Self>(stream, ctx),
                sender: Some(sender),
            }
        });
        receiver.await.map_err(|_| ())?
    }
}

impl ConnectionManager<Server> for StatusServerManager {}

impl Actor for StatusServerManager {
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

impl StreamHandler<Result<PacketServerEnum, ()>> for StatusServerManager {
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

impl Handler<HandlerMessage<Server>> for StatusServerManager {
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

impl WriteHandler<()> for StatusServerManager {}

impl HandlePacket<Server, StatusResponsePacket> for StatusServerManager {
    fn handle_packet(&mut self, packet: StatusResponsePacket, _ctx: &mut Self::Context) -> Result<(), ()> {
        if let Some(sender) = self.sender.take() {
            sender.send(Ok(packet.status)).map_err(|_| ())?;
        }

        self.connection.disconnect();
        Ok(())
    }
}

impl HandlePacket<Server, PongPacket> for StatusServerManager {
    fn handle_packet(&mut self, _packet: PongPacket, _ctx: &mut Self::Context) -> Result<(), ()> {
        // Accept ping packets without reacting to them
        Ok(())
    }
}
