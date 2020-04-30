use crate::net::{Protocol, Packet, PacketCodec};
use crate::net::buffer::{Buffer, BufferMut};

/// The first packet that is ever send to the server.
/// It indicates whether the client wants to join or view the server status.
#[derive(Debug, Clone)]
pub struct HandshakePacket {
    pub protocol_version: i32,
    pub server_address: String,
    pub server_port: u16,
    pub next_protocol: Protocol,
}

impl Packet for HandshakePacket {}

impl PacketCodec for HandshakePacket {
    fn decode<B: Buffer>(buf: &mut B) -> Result<Self, ()> {
        Ok(HandshakePacket {
            protocol_version: buf.read_var_int()?,
            server_address: buf.read_string()?,
            server_port: buf.read_u16(),
            next_protocol: Protocol::from_int(buf.read_var_int()?).ok_or(())?
        })
    }

    fn encode<B: BufferMut>(&self, buf: &mut B) -> Result<(), ()> {
        buf.write_var_int(self.protocol_version);
        buf.write_string(&self.server_address);
        buf.write_u16(self.server_port);
        buf.write_var_int(self.next_protocol.to_int());
        Ok(())
    }
}
