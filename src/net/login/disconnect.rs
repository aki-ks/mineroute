use crate::net::{Packet, PacketCodec};
use crate::net::buffer::{Buffer, BufferMut};

/// This packet tells the client a disconnect reason, before closing the connection
#[derive(Debug, Clone)]
pub struct DisconnectPacket {
    reason: String,
}

impl Packet for DisconnectPacket {}

impl PacketCodec for DisconnectPacket {
    fn decode<B: Buffer>(buf: &mut B) -> Result<Self, ()> {
        Ok(DisconnectPacket {
            reason: buf.read_string()?,
        })
    }

    fn encode<B: BufferMut>(&self, buf: &mut B) -> Result<(), ()> {
        buf.write_string(&self.reason);
        Ok(())
    }
}
