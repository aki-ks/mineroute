use bytes::Bytes;
use crate::net::Packet;

/// The id and data of a not decoded packet
#[derive(Debug, Clone)]
pub struct RawPacket {
    pub id: u8,
    pub data: Bytes,
}

impl Packet for RawPacket {}
