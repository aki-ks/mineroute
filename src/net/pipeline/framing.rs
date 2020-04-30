use bytes::{BytesMut, Buf, BufMut};
use crate::net::buffer::{BufferMut, var_int_size};

/// Utility for encoding and decoding frames.
///
/// A frame is the data of one packet preceded by its size.
/// The size is encoded as a var-int with a maximum of 21-bit.
pub struct FrameCodec;

impl FrameCodec {
    /// Try to decode a frame from a provided buffer.
    /// This may fail if the frame was not yet completely received.
    pub fn try_decode(src: &mut BytesMut) -> Result<Option<BytesMut>, ()> {
        if let Some(frame_size) = peek_var_int_21(src)? {
            let total_frame_size = frame_size.size + frame_size.value;
            if src.len() < total_frame_size {
                // Reserve enough space to read the missing remainder of this frame
                src.reserve(total_frame_size - src.len());
            } else {
                src.advance(frame_size.size);
                let frame = src.split_to(frame_size.value);
                return Ok(Some(frame));
            }
        }
        Ok(None)
    }

    pub fn encode(payload: BytesMut) -> Result<BytesMut, ()> {
        let payload_size = payload.len() as i32;
        let packet_size = var_int_size(payload_size) + payload.len();

        let mut buffer = BytesMut::with_capacity(packet_size);
        buffer.write_var_int(payload_size);
        buffer.put_slice(&payload);
        Ok(buffer)
    }
}

struct PeekedVarInt {
    value: usize,
    size: usize,
}

/// Read a var-int encoded 21-bit number from a buffer without consuming it.
///
/// Returns the peeked number and its size on the buffer, if it could already be read.
fn peek_var_int_21(src: &BytesMut) -> Result<Option<PeekedVarInt>, ()> {
    let mut result = 0;
    for i in 0..3 {
        if let Some(byte) = src.get(i) {
            result |= ((byte & 127) as usize) << (i * 7);

            if byte & 128 == 0 {
                return Ok(Some(PeekedVarInt { value: result, size: i + 1}));
            }
        } else {
            return Ok(None);
        }
    }
    Err(())
}
