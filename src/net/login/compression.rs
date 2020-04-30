use crate::net::{Packet, PacketCodec};
use crate::net::buffer::{Buffer, BufferMut};

/// Tell the client to enable or disable compression
#[derive(Debug, Clone)]
pub struct CompressionPacket {
    /// Compress packets exceeding this size in bytes.
    /// Disables compression if no limit is provided.
    pub size_limit: Option<usize>,
}

impl Packet for CompressionPacket {}

impl PacketCodec for CompressionPacket {
    fn decode<B: Buffer>(buf: &mut B) -> Result<Self, ()> {
        let limit: i32 = buf.read_var_int()?;
        Ok(CompressionPacket {
            size_limit: if limit < 0 { None } else { Some(limit as usize) },
        })
    }

    fn encode<B: BufferMut>(&self, buf: &mut B) -> Result<(), ()> {
        if let Some(size_limit) = self.size_limit {
            let size_limit= size_limit as i32;
            if size_limit < 0 {
                // The size limit is too large and would overflow
                // to a negative number on the client side.
                return Err(());
            }

            buf.write_var_int(size_limit);
        } else {
            buf.write_var_int(-1);
        }

        Ok(())
    }
}
