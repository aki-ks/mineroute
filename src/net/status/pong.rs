use crate::net::{Packet, PacketCodec};
use crate::net::buffer::{Buffer, BufferMut};

/// The echo response to the [PingPacket].
#[derive(Debug, Clone)]
pub struct PongPacket {
    pub payload: u64,
}

impl Packet for PongPacket {}

impl PacketCodec for PongPacket {
    fn decode<B: Buffer>(buf: &mut B) -> Result<Self, ()> {
        Ok(PongPacket {
            payload: buf.read_u64()
        })
    }

    fn encode<B: BufferMut>(&self, buf: &mut B) -> Result<(), ()> {
        buf.write_u64(self.payload);
        Ok(())
    }
}
