use std::rc::Rc;
use std::sync::RwLock;
use std::marker::PhantomData;
use bytes::{Buf, BytesMut};
use crate::net::*;

/// Encodes/Decodes packets to bytes
pub struct PacketCodec<C: ConnectionType> {
    protocol: Rc<RwLock<Protocol>>,
    phantom: PhantomData<C>,
}

impl<C: ConnectionType> PacketCodec<C> {
    pub fn new(protocol: Rc<RwLock<Protocol>>) -> PacketCodec<C> {
        PacketCodec {
            protocol,
            phantom: PhantomData,
        }
    }

    pub fn decode(&self, src: &mut BytesMut) -> Result<C::In, ()> {
        let packet_id = src.get_u8();
        let protocol = self.protocol.read().unwrap();
        C::WC::read_packet(&protocol, packet_id, src)
    }

    pub fn encode(&self, packet: &C::Out, dst: &mut BytesMut) -> Result<(), ()> {
        let protocol = self.protocol.write().unwrap();
        C::WC::write_packet(&protocol, packet, dst)
    }
}
