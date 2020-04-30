use uuid::Uuid;
use crate::net::{Packet, PacketCodec};
use crate::net::buffer::{Buffer, BufferMut};

/// Tell the client that the login procedure was successfully completed.
/// The client should switch to the `Protocol::Play` protocol.
#[derive(Debug, Clone)]
pub struct LoginSuccessPacket {
    pub uuid: Uuid,
    pub name: String,
}

impl Packet for LoginSuccessPacket {}

impl PacketCodec for LoginSuccessPacket {
    fn decode<B: Buffer>(buf: &mut B) -> Result<Self, ()> {
        Ok(LoginSuccessPacket {
            uuid: buf.read_uuid()?,
            name: buf.read_string()?,
        })
    }

    fn encode<B: BufferMut>(&self, buf: &mut B) -> Result<(), ()> {
        buf.write_uuid(&self.uuid);
        buf.write_string(&self.name);
        Ok(())
    }
}
