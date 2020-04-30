use serde_json::{de, ser};
use crate::net::{Packet, PacketCodec};
use crate::net::buffer::{BufferMut, Buffer};
use crate::net::status::server_status;

/// Response to the [StatusRequestPacket] containing the status information of the server
#[derive(Debug, Clone)]
pub struct StatusResponsePacket {
    pub status: server_status::ServerInfo,
}

impl Packet for StatusResponsePacket {}

impl PacketCodec for StatusResponsePacket {
    fn decode<B: Buffer>(buf: &mut B) -> Result<Self, ()> {
        let payload = buf.read_string()?;
        Ok(StatusResponsePacket {
            status: de::from_str::<server_status::ServerInfo>(&payload).map_err(|_| ())?,
        })
    }

    fn encode<B: BufferMut>(&self, buf: &mut B) -> Result<(), ()> {
        buf.write_string(&ser::to_string(&self.status).map_err(|_| ())?);
        Ok(())
    }
}
