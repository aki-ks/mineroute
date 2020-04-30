use std::net::{IpAddr, SocketAddr, Ipv4Addr};
use std::rc::Rc;
use std::sync::Mutex;
use actix::prelude::*;
use actix::io::WriteHandler;
use tokio::net::TcpStream;
use futures::channel::oneshot;
use futures::FutureExt;
use crate::net::handshake::HandshakePacket;
use crate::net::*;
use crate::net::login::LoginStartPacket;
use crate::net::status::{StatusRequestPacket, StatusResponsePacket, PingPacket, PongPacket};
use crate::net::manager::{ProxyServerManager, HandlePacket, HandlerMessage, ConnectionManager, StatusServerManager};
use crate::net::status::server_status::ServerInfo;
use crate::net::play::RawPacket;

/// Manage a connection from a client to this server.
///
/// This server will act as a proxy server,
/// forwarding all packets to a defined upstream.
pub struct ProxyClientManager {
    connection: ClientConnection<Self>,
    handshake: Option<HandshakePacket>,
    upstream: Rc<Mutex<Option<Addr<ProxyServerManager<ProxyClientManager>>>>>,
}

impl ProxyClientManager {
    pub fn new(stream: TcpStream, ctx: &mut Context<Self>) -> ProxyClientManager {
        ProxyClientManager {
            connection: ClientConnection::new(stream, ctx),
            handshake: None,
            upstream: Rc::new(Mutex::new(None)),
        }
    }
}

impl ConnectionManager<Client> for ProxyClientManager {}

impl Actor for ProxyClientManager {
    type Context = Context<Self>;
}

impl StreamHandler<Result<PacketClientEnum, ()>> for ProxyClientManager {
    fn handle(&mut self, packet: Result<PacketClientEnum, ()>, ctx: &mut Self::Context) {
        let handle_result = packet.and_then(|packet| match packet {
            PacketClientEnum::Handshake(packet) => self.handle_packet(packet, ctx),

            PacketClientEnum::StatusRequest(packet) => self.handle_packet(packet, ctx),
            PacketClientEnum::Ping(packet) => self.handle_packet(packet, ctx),

            PacketClientEnum::LoginStart(packet) => self.handle_packet(packet, ctx),

            PacketClientEnum::Raw(packet) => self.handle_packet(packet, ctx),
        });

        if let Err(()) = handle_result {
            self.connection.disconnect();
        }
    }
}

impl Handler<HandlerMessage<Client>> for ProxyClientManager {
    type Result = Result<(), ()>;
    fn handle(&mut self, message: HandlerMessage<Client>, _ctx: &mut Self::Context) -> Self::Result {
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
            }
        }
    }
}

impl WriteHandler<()> for ProxyClientManager {}

impl HandlePacket<Client, HandshakePacket> for ProxyClientManager {
    fn handle_packet(&mut self, packet: HandshakePacket, _ctx: &mut Self::Context) -> Result<(), ()> {
        match packet.next_protocol {
            Protocol::Status | Protocol::Login => {
                self.connection.set_protocol(packet.next_protocol.clone());
                self.handshake = Some(packet);
                Ok(())
            }
            _ => Err(()),
        }
    }
}

impl HandlePacket<Client, StatusRequestPacket> for ProxyClientManager {
    fn handle_packet(&mut self, _packet: StatusRequestPacket, ctx: &mut Self::Context) -> Result<(), ()> {
        let server_info_future = async {
            let addr = {
                let localhost = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
                SocketAddr::new(localhost, 25566)
            };

            let stream = TcpStream::connect(addr).await.unwrap();
            let (sender, receiver) = oneshot::channel::<Result<ServerInfo, ()>>();
            StatusServerManager::create(|ctx| {
                StatusServerManager::new(stream, ctx, |server_info| {
                    sender.send(server_info).unwrap();
                })
            });
            receiver.await.unwrap_or(Err(()))
        };

        ctx.spawn(server_info_future.into_actor(self).map(|server_info, manager, ctx| {
            match server_info {
                Ok(status) => manager.connection.send_packet(PacketServerEnum::StatusResponse(StatusResponsePacket { status })).unwrap(),
                _ => ctx.stop(),
            }
        }));

        Ok(())
    }
}

impl HandlePacket<Client, PingPacket> for ProxyClientManager {
    fn handle_packet(&mut self, packet: PingPacket, _ctx: &mut Self::Context) -> Result<(), ()> {
        let packet = PacketServerEnum::Pong(PongPacket {
            payload: packet.payload,
        });

        self.connection.send_packet(packet)
    }
}

impl HandlePacket<Client, LoginStartPacket> for ProxyClientManager {
    fn handle_packet(&mut self, packet: LoginStartPacket, ctx: &mut Self::Context) -> Result<(), ()> {
        let handshake = self.handshake.take().unwrap();
        let downstream = ctx.address().clone();
        let self_upstream = self.upstream.clone();
        ctx.wait(async move {
            let addr = {
                let localhost = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
                SocketAddr::new(localhost, 25566)
            };

            let stream = TcpStream::connect(addr).await.unwrap();

            let upstream = ProxyServerManager::create(|ctx| {
                ProxyServerManager::new(downstream, stream, ctx)
            });

            upstream.send(HandlerMessage::SendPacket(PacketClientEnum::Handshake(handshake))).await.unwrap().unwrap();
            upstream.send(HandlerMessage::SetProtocol(Protocol::Login)).await.unwrap().unwrap();
            upstream.send(HandlerMessage::SendPacket(PacketClientEnum::LoginStart(packet))).await.unwrap().unwrap();

            *self_upstream.lock().unwrap() = Some(upstream);
        }.into_actor(self));
        Ok(())
    }
}

impl HandlePacket<Client, RawPacket> for ProxyClientManager {
    fn handle_packet(&mut self, packet: RawPacket, ctx: &mut Self::Context) -> Result<(), ()> {
        let upstream = self.upstream.lock().unwrap();
        let upstream = upstream.as_ref().unwrap();
        upstream.send(HandlerMessage::SendPacket(PacketClientEnum::Raw(packet)))
            .map(|_| ()).into_actor(self).wait(ctx);
        Ok(())
    }
}

