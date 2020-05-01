use crate::net::{Packet, PacketCodec};
use crate::net::buffer::{Buffer, BufferMut};

/// Request to login with a certain username
#[derive(Debug, Clone)]
pub struct LoginStartPacket {
    pub name: String,
}

impl Packet for LoginStartPacket {}

impl PacketCodec for LoginStartPacket {
    fn decode<B: Buffer>(buf: &mut B) -> Result<Self, ()> {
        Ok(LoginStartPacket {
            name: buf.read_string()?,
        })
    }

    fn encode<B: BufferMut>(&self, buf: &mut B) -> Result<(), ()> {
        buf.write_string(&self.name);
        Ok(())
    }
}
