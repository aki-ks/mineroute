use crate::net::*;
use crate::net::buffer::{Buffer, BufferMut};
use crate::net::play::RawPacket;

/// A utility responsible for mapping packets to their ids.
pub trait WireCodec<C: ConnectionType> {
    fn read_packet<B: Buffer>(protocol: &Protocol, packet_id: u8, buf: &mut B) -> Result<C::In, ()>;
    fn write_packet<B: BufferMut>(protocol: &Protocol, packet: &C::Out, buf: &mut B) -> Result<(), ()>;
}

fn write<B: BufferMut, P: PacketCodec>(packet_id: u8, packet: &P, buf: &mut B) -> Result<(), ()> {
    buf.write_u8(packet_id);
    packet.encode(buf)
}

pub struct ClientWireCodec;
impl WireCodec<Client> for ClientWireCodec {
    fn read_packet<B: Buffer>(protocol: &Protocol, packet_id: u8, buf: &mut B) -> Result<PacketClientEnum, ()> {
        match protocol {
            Protocol::Handshake => match packet_id {
                0 => handshake::HandshakePacket::decode(buf).map(PacketClientEnum::Handshake),
                _ => Err(()),
            },
            Protocol::Status => match packet_id {
                0 => status::StatusRequestPacket::decode(buf).map(PacketClientEnum::StatusRequest),
                1 => status::PingPacket::decode(buf).map(PacketClientEnum::Ping),
                _ => Err(()),
            },
            Protocol::Login => match packet_id {
                0 => login::LoginStartPacket::decode(buf).map(PacketClientEnum::LoginStart),
                _ => Err(())
            }
            Protocol::Play => Ok(PacketClientEnum::Raw(RawPacket {
                id: packet_id,
                data: buf.remaining_bytes(),
            })),
        }
    }

    fn write_packet<B: BufferMut>(protocol: &Protocol, packet: &PacketServerEnum, buf: &mut B) -> Result<(), ()> {
        match protocol {
            Protocol::Handshake => Err(()),
            Protocol::Status => match packet {
                PacketServerEnum::StatusResponse(packet) => write(0, packet, buf),
                PacketServerEnum::Pong(packet) => write(1, packet, buf),
                _ => Err(()),
            },
            Protocol::Login => match packet {
                PacketServerEnum::Disconnect(packet) => write(0, packet, buf),
                PacketServerEnum::LoginSuccess(packet) => write(2, packet, buf),
                PacketServerEnum::Compression(packet) => write(3, packet, buf),
                _ => Err(()),
            },
            Protocol::Play => match packet {
                PacketServerEnum::Raw(packet) => {
                    buf.write_u8(packet.id);
                    buf.write_raw_bytes(&packet.data);
                    Ok(())
                },
                _ => Err(()),
            },
        }
    }
}

pub struct ServerWireCodec;
impl WireCodec<Server> for ServerWireCodec {
    fn read_packet<B: Buffer>(protocol: &Protocol, packet_id: u8, buf: &mut B) -> Result<PacketServerEnum, ()> {
        match protocol {
            Protocol::Handshake => Err(()),
            Protocol::Status => match packet_id {
                0 => status::StatusResponsePacket::decode(buf).map(PacketServerEnum::StatusResponse),
                1 => status::PongPacket::decode(buf).map(PacketServerEnum::Pong),
                _ => Err(()),
            },
            Protocol::Login => match packet_id {
                0 => login::DisconnectPacket::decode(buf).map(PacketServerEnum::Disconnect),
                2 => login::LoginSuccessPacket::decode(buf).map(PacketServerEnum::LoginSuccess),
                3 => login::CompressionPacket::decode(buf).map(PacketServerEnum::Compression),
                _ => Err(()),
            },
            Protocol::Play => Ok(PacketServerEnum::Raw(RawPacket {
                id: packet_id,
                data: buf.remaining_bytes(),
            })),
        }
    }

    fn write_packet<B: BufferMut>(protocol: &Protocol, packet: &PacketClientEnum, buf: &mut B) -> Result<(), ()> {
        match protocol {
            Protocol::Handshake => match packet {
                PacketClientEnum::Handshake(packet) => write(0, packet, buf),
                _ => Err(()),
            },
            Protocol::Status => match packet {
                PacketClientEnum::StatusRequest(packet) => write(0, packet, buf),
                PacketClientEnum::Ping(packet) => write(1, packet, buf),
                _ => Err(()),
            },
            Protocol::Login => match packet {
                PacketClientEnum::LoginStart(packet) => write(0, packet, buf),
                _ => Err(()),
            },
            Protocol::Play => match packet {
                PacketClientEnum::Raw(packet) => {
                    buf.write_u8(packet.id);
                    buf.write_raw_bytes(&packet.data);
                    Ok(())
                },
                _ => Err(()),
            },
        }
    }
}
