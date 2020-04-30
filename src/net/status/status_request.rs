use crate::net::{Packet, PacketCodec};
use crate::net::buffer::{BufferMut, Buffer};

/// Request the status (playercount, Motd, etc.) of the connected server
#[derive(Debug, Clone)]
pub struct StatusRequestPacket;

impl Packet for StatusRequestPacket {}

impl PacketCodec for StatusRequestPacket {
    fn decode<B: Buffer>(_: &mut B) -> Result<Self, ()> {
        Ok(StatusRequestPacket)
    }

    fn encode<B: BufferMut>(&self, _: &mut B) -> Result<(), ()> {
        Ok(())
    }
}
