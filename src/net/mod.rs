pub mod manager;
pub mod buffer;
pub mod pipeline;
pub mod handshake;
pub mod login;
pub mod play;
pub mod status;

mod connection;
mod wire_codec;

pub use connection::Connection;

use actix::Message;
use crate::net::buffer::{Buffer, BufferMut};
use crate::net::wire_codec::{WireCodec, ServerWireCodec, ClientWireCodec};

pub trait ConnectionType: Sized + 'static {
    type In;
    type Out;
    type WC: WireCodec<Self>;
}

/// ConnectionType for connections from a client to a server
pub struct Server;
impl ConnectionType for Server {
    type In = PacketServerEnum;
    type Out = PacketClientEnum;
    type WC = ServerWireCodec;
}

/// Connectiontype for connections from a server to a client
pub struct Client;
impl ConnectionType for Client {
    type In = PacketClientEnum;
    type Out = PacketServerEnum;
    type WC = ClientWireCodec;
}

/// Packets send by the client to the server
#[allow(unused)]
#[derive(Debug, Clone)]
pub enum PacketClientEnum {
    Handshake(handshake::HandshakePacket),

    StatusRequest(status::StatusRequestPacket),
    Ping(status::PingPacket),

    LoginStart(login::LoginStartPacket),

    Raw(play::RawPacket),
}

/// Packets send by the server to the client
#[allow(unused)]
#[derive(Debug, Clone)]
pub enum PacketServerEnum {
    StatusResponse(status::StatusResponsePacket),
    Pong(status::PongPacket),

    Disconnect(login::DisconnectPacket),
    Compression(login::CompressionPacket),
    LoginSuccess(login::LoginSuccessPacket),

    Raw(play::RawPacket),
}

// Implementing Message for packets allows them to be send to actix actors
impl Message for PacketServerEnum { type Result = (); }
impl Message for PacketClientEnum { type Result = (); }

pub trait Packet: Sized {}
pub trait PacketCodec: Packet {
    fn decode<B: Buffer>(buf: &mut B) -> Result<Self, ()>;
    fn encode<B: BufferMut>(&self, buf: &mut B) -> Result<(), ()>;
}

/// The different sub-protocols of the minecraft protocol.
///
/// Each Protocol defines a set a Packets, that each have a packet id.
/// The meaning of a PacketId varies between the different Protocols and
/// whether the packet is directed to the server or client.
#[derive(Debug, Clone)]
pub enum Protocol {
    Handshake,
    Play,
    Status,
    Login,
}

impl Protocol {
    pub fn from_int(int: i32) -> Option<Protocol> {
        match int {
            0 => Some(Protocol::Play),
            1 => Some(Protocol::Status),
            2 => Some(Protocol::Login),
            _ => None
        }
    }

    pub fn to_int(&self) -> i32 {
        match self {
            Protocol::Play => 0,
            Protocol::Status => 1,
            Protocol::Login => 2,
            Protocol::Handshake => panic!("Protocol 'Handshake' cannot be converted into an int"),
        }
    }
}
