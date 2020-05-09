use std::net::SocketAddr;
use std::rc::Rc;
use std::sync::{Mutex, RwLock};
use actix::prelude::*;
use actix::io::WriteHandler;
use tokio::net::TcpStream;
use futures::FutureExt;
use crate::net::handshake::HandshakePacket;
use crate::net::*;
use crate::net::login::LoginStartPacket;
use crate::net::status::{StatusRequestPacket, StatusResponsePacket, PingPacket, PongPacket};
use crate::net::manager::{ProxyServerManager, PacketHandler, HandlerMessage, ConnectionManager, StatusServerManager};
use crate::net::play::RawPacket;
use crate::server_state::Configuration;

/// Manage a client connection to this server.
///
/// This server will act as a proxy server,
/// forwarding all packets to a defined upstream.
pub struct ProxyClientManager {
    config: Rc<RwLock<Configuration>>,
    connection: Connection<Client>,
    handshake: Option<HandshakePacket>,
    upstream: Rc<Mutex<Option<Addr<ProxyServerManager<ProxyClientManager>>>>>,

    /// The name of the user if already connected
    name: Option<String>,

    /// The hostname used to connect to the server
    connection_host: Option<String>,

    /// The hostname that this connection should get proxied to
    upstream_host: Option<SocketAddr>,
}

impl ProxyClientManager {
    pub fn new(config: Rc<RwLock<Configuration>>, stream: TcpStream, ctx: &mut Context<Self>) -> ProxyClientManager {
        ProxyClientManager {
            config,
            connection: Connection::new::<Self>(stream, ctx),
            handshake: None,
            upstream: Rc::new(Mutex::new(None)),
            name: None,
            connection_host: None,
            upstream_host: None,
        }
    }
}

impl ConnectionManager<Client> for ProxyClientManager {}

impl Actor for ProxyClientManager {
    type Context = Context<Self>;
}

impl StreamHandler<Result<PacketClientEnum, ()>> for ProxyClientManager {
    /// Handle incoming packets by delegating to the corresponding [[PacketHandler]].
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

    /// Handle a disconnection of a player from the mineroute server.
    ///
    /// If the player was connected to an upstream server,
    /// he should get removed from its player list.
    fn finished(&mut self, _ctx: &mut Self::Context) {
        if let Some(ref upstream) = *self.upstream.lock().unwrap() {
            upstream.do_send(HandlerMessage::Disconnect())
        }

        if let Some(ref upstream_host) = self.connection_host {
            if let Some(ref name) = self.name {
                let mut config = self.config.write().unwrap();
                if let Some(server) = config.get_server_mut(upstream_host) {
                    server.remove_player(name);
                }
            }
        }
    }
}

/// Handle connection control messages that this actor may
/// receive from a linked [[ProxyServerManager]] actor
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
            HandlerMessage::Disconnect() => {
                self.connection.disconnect();
                Ok(())
            }
        }
    }
}

impl WriteHandler<()> for ProxyClientManager {}

// Handle the initial handshake packet by determining
// the upstream server requested by the client.
impl PacketHandler<Client, HandshakePacket> for ProxyClientManager {
    fn handle_packet(&mut self, packet: HandshakePacket, _ctx: &mut Self::Context) -> Result<(), ()> {
        if let Protocol::Status | Protocol::Login = packet.next_protocol {
            let address = packet.server_address.clone();
            let config = self.config.write().unwrap();

            if let Some(upstream) = config.get_server(&address) {
                self.connection.set_protocol(packet.next_protocol.clone());
                self.connection_host = Some(address);
                self.upstream_host = Some(upstream.upstream.clone());
                self.handshake = Some(packet);
                return Ok(())
            }
        }

        self.connection.disconnect();
        Err(())
    }
}

/// Handle status requests by connecting to the upstream server as a client,
/// asking it for its status and forwarding that response to the client
impl PacketHandler<Client, StatusRequestPacket> for ProxyClientManager {
    fn handle_packet(&mut self, _packet: StatusRequestPacket, ctx: &mut Self::Context) -> Result<(), ()> {
        let upstream_addr = self.upstream_host.unwrap();

        let server_info = StatusServerManager::fetch_status(upstream_addr)
            .into_actor(self)
            .map(|server_info, manager, ctx| {
                match server_info {
                    Ok(status) => manager.connection.send_packet(PacketServerEnum::StatusResponse(StatusResponsePacket { status })).unwrap(),
                    _ => ctx.stop(),
                }
            });

        ctx.spawn(server_info);
        Ok(())
    }
}

/// Immediately respond to incoming ping packets with a PongPacket
impl PacketHandler<Client, PingPacket> for ProxyClientManager {
    fn handle_packet(&mut self, packet: PingPacket, _ctx: &mut Self::Context) -> Result<(), ()> {
        let packet = PacketServerEnum::Pong(PongPacket {
            payload: packet.payload,
        });

        self.connection.send_packet(packet)
    }
}

impl PacketHandler<Client, LoginStartPacket> for ProxyClientManager {
    fn handle_packet(&mut self, packet: LoginStartPacket, ctx: &mut Self::Context) -> Result<(), ()> {
        let handshake = self.handshake.clone().unwrap();
        let downstream = ctx.address().clone();
        let self_upstream = self.upstream.clone();

        let name = packet.name.clone();
        self.name = Some(name.clone());

        let mut config = self.config.write().unwrap();
        let server = config.get_server_mut(self.connection_host.as_ref().unwrap()).unwrap();
        server.add_player(name);
        drop(config); // config lock is no longer required

        let upstream = self.upstream_host.clone().unwrap();
        let future = async move {
            let stream = TcpStream::connect(upstream).await.map_err(|_| ())?;

            let upstream = ProxyServerManager::create(|ctx| {
                ProxyServerManager::new(downstream, stream, ctx)
            });

            upstream.send(HandlerMessage::SendPacket(PacketClientEnum::Handshake(handshake))).await.unwrap_or_else(|_| Err(()))?;
            upstream.send(HandlerMessage::SetProtocol(Protocol::Login)).await.unwrap_or_else(|_| Err(()))?;
            upstream.send(HandlerMessage::SendPacket(PacketClientEnum::LoginStart(packet))).await.unwrap_or_else(|_| Err(()))?;

            Ok(upstream)
        }.into_actor(self).map(move |upstream_result: Result<Addr<ProxyServerManager<ProxyClientManager>>, ()>, actor, _ctx| {
            match upstream_result {
                Ok(upstream) => *self_upstream.lock().unwrap() = Some(upstream),
                Err(_) => actor.connection.disconnect(),
            }
        });
        ctx.wait(future);
        Ok(())
    }
}

impl PacketHandler<Client, RawPacket> for ProxyClientManager {
    fn handle_packet(&mut self, packet: RawPacket, ctx: &mut Self::Context) -> Result<(), ()> {
        let upstream = self.upstream.lock().unwrap();
        let upstream = upstream.as_ref().unwrap();
        upstream.send(HandlerMessage::SendPacket(PacketClientEnum::Raw(packet)))
            .map(|_| ()).into_actor(self).wait(ctx);
        Ok(())
    }
}

