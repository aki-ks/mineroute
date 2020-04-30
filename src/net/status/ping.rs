use crate::net::{Packet, PacketCodec};
use crate::net::buffer::{BufferMut, Buffer};

/// Send a number to the server that gets immediately echoed.
/// This is usually the system time of the client
#[derive(Debug, Clone)]
pub struct PingPacket {
    pub payload: u64,
}

impl Packet for PingPacket {}

impl PacketCodec for PingPacket {
    fn decode<B: Buffer>(buf: &mut B) -> Result<Self, ()> {
        Ok(PingPacket {
            payload: buf.read_u64()
        })
    }

    fn encode<B: BufferMut>(&self, buf: &mut B) -> Result<(), ()> {
        buf.write_u64(self.payload);
        Ok(())
    }
}
