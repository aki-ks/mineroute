use std::ops::Deref;
use deflate::deflate_bytes_zlib;
use inflate::inflate_bytes_zlib;
use bytes::{BytesMut, Buf, BufMut};
use crate::net::buffer::{Buffer, BufferMut, var_int_size};

/// An additional step in the pipeline compressing (deflate)
/// packets exceeding a defined byte size.
pub struct Compressor {
    pub size_limit: usize,
}

impl Compressor {
    pub fn encode(&self, payload: BytesMut) -> Result<BytesMut, ()> {
        fn to_packet(data_len: i32, data: &[u8]) -> Result<BytesMut, ()> {
            let mut buffer = BytesMut::with_capacity(var_int_size(data_len) + data.len());
            buffer.write_var_int(data_len);
            buffer.put_slice(data);
            Ok(buffer)
        }

        if payload.remaining() < self.size_limit {
            to_packet(0, &payload)
        } else {
            let compressed = deflate_bytes_zlib(payload.bytes());
            to_packet(payload.len() as i32, &compressed)
        }
    }

    pub fn decode(&self, mut buffer: BytesMut) -> Result<BytesMut, ()> {
        match buffer.read_var_int()? {
            0 => Ok(buffer),
            uncompressed_size => {
                let decompressed = inflate_bytes_zlib(&buffer).map_err(|_| ())?;
                debug_assert_eq!(decompressed.len(), uncompressed_size as usize);
                Ok(BytesMut::from(decompressed.deref()))
            }
        }
    }
}
