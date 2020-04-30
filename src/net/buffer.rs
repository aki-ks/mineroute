use std::str::FromStr;
use bytes::{Buf, BufMut, BytesMut, Bytes};
use uuid::Uuid;

/// Calculate the size (in bytes) of a number when encoded as var-int
pub fn var_int_size(mut int: i32) -> usize {
    let mut size = 1;
    while (int & !127) != 0 {
        int >>= 7;
        size += 1;
    }
    size
}

pub trait Buffer {
    fn read_u8(&mut self) -> u8;
    fn read_u16(&mut self) -> u16;
    fn read_u64(&mut self) -> u64;
    fn read_var_int(&mut self) -> Result<i32, ()>;
    fn read_byte_array(&mut self) -> Result<Vec<u8>, ()>;
    fn read_string(&mut self) -> Result<String, ()>;
    fn read_uuid(&mut self) -> Result<Uuid, ()>;
    fn remaining_bytes(&mut self) -> Bytes;
}

impl Buffer for BytesMut {
    fn read_u8(&mut self) -> u8 {
        self.get_u8()
    }

    fn read_u16(&mut self) -> u16 {
        self.get_u16()
    }

    fn read_u64(&mut self) -> u64 {
        self.get_u64()
    }

    fn read_var_int(&mut self) -> Result<i32, ()> {
        let mut result = 0;
        for i in 0..5 {
            let byte = self.get_u8();
            result |= ((byte & 127) as i32) << (i * 7);
            if byte & 128 == 0 {
                return Ok(result);
            }
        }

        Err(())
    }

    fn read_byte_array(&mut self) -> Result<Vec<u8>, ()> {
        let size = self.read_var_int()? as usize;
        Ok((0..size).map(|_| self.get_u8()).collect())
    }

    fn read_string(&mut self) -> Result<String, ()> {
        String::from_utf8(self.read_byte_array()?).map_err(|_|())
    }

    fn read_uuid(&mut self) -> Result<Uuid, ()> {
        Uuid::from_str(&self.read_string()?).map_err(|_| ())
    }

    fn remaining_bytes(&mut self) -> Bytes {
        self.to_bytes()
    }
}

pub trait BufferMut {
    fn write_u8(&mut self, byte: u8);
    fn write_u16(&mut self, short: u16);
    fn write_u64(&mut self, long: u64);
    fn write_var_int(&mut self, int: i32);
    fn write_byte_array(&mut self, array: &[u8]);
    fn write_string(&mut self, string: &str);
    fn write_uuid(&mut self, uuid: &Uuid);

    // Write raw bytes, not prepending the slice size
    fn write_raw_bytes(&mut self, bytes: &[u8]);
}

impl BufferMut for BytesMut {
    fn write_u8(&mut self, byte: u8) {
        self.reserve(1);
        self.put_u8(byte);
    }

    fn write_u16(&mut self, short: u16) {
        self.reserve(2);
        self.put_u16(short);
    }

    fn write_u64(&mut self, long: u64) {
        self.reserve(4);
        self.put_u64(long);
    }

    fn write_var_int(&mut self, mut int: i32) {
        self.reserve(var_int_size(int));

        while (int & !127) != 0 {
            self.put_u8((int as u8) & 127 | 128);
            int >>= 7;
        }
        self.put_u8((int as u8) & 127)
    }

    fn write_byte_array(&mut self, array: &[u8]) {
        let array_size = array.len() as i32;
        self.reserve(var_int_size(array_size) + array.len());
        self.write_var_int(array_size);
        self.put_slice(array)
    }

    fn write_string(&mut self, string: &str) {
        self.write_byte_array(string.as_bytes())
    }

    fn write_uuid(&mut self, uuid: &Uuid) {
        self.write_string(&uuid.to_string())
    }

    fn write_raw_bytes(&mut self, bytes: &[u8]) {
        self.put_slice(bytes);
    }
}

