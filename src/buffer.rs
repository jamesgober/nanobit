//! High-performance buffer implementations for binary I/O

#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec};

use crate::error::{Error, Result};

/// A high-performance write buffer for binary serialization
#[derive(Debug)]
pub struct WriteBuffer {
    data: Vec<u8>,
    capacity: usize,
}

impl WriteBuffer {
    /// Create a new write buffer with default capacity
    pub fn new() -> Self {
        Self::with_capacity(crate::DEFAULT_BUFFER_SIZE)
    }

    /// Create a new write buffer with the specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            capacity,
        }
    }

    /// Write a single byte
    #[inline]
    pub fn write_u8(&mut self, value: u8) -> Result<()> {
        self.data.push(value);
        Ok(())
    }

    /// Write a u16 in little-endian format
    #[inline]
    pub fn write_u16(&mut self, value: u16) -> Result<()> {
        self.data.extend_from_slice(&value.to_le_bytes());
        Ok(())
    }

    /// Write a u32 in little-endian format
    #[inline]
    pub fn write_u32(&mut self, value: u32) -> Result<()> {
        self.data.extend_from_slice(&value.to_le_bytes());
        Ok(())
    }

    /// Write a u64 in little-endian format
    #[inline]
    pub fn write_u64(&mut self, value: u64) -> Result<()> {
        self.data.extend_from_slice(&value.to_le_bytes());
        Ok(())
    }

    /// Write an i8
    #[inline]
    pub fn write_i8(&mut self, value: i8) -> Result<()> {
        self.write_u8(value as u8)
    }

    /// Write an i16 in little-endian format
    #[inline]
    pub fn write_i16(&mut self, value: i16) -> Result<()> {
        self.write_u16(value as u16)
    }

    /// Write an i32 in little-endian format
    #[inline]
    pub fn write_i32(&mut self, value: i32) -> Result<()> {
        self.write_u32(value as u32)
    }

    /// Write an i64 in little-endian format
    #[inline]
    pub fn write_i64(&mut self, value: i64) -> Result<()> {
        self.write_u64(value as u64)
    }

    /// Write an f32 in IEEE 754 format
    #[inline]
    pub fn write_f32(&mut self, value: f32) -> Result<()> {
        self.write_u32(value.to_bits())
    }

    /// Write an f64 in IEEE 754 format
    #[inline]
    pub fn write_f64(&mut self, value: f64) -> Result<()> {
        self.write_u64(value.to_bits())
    }

    /// Write a variable-length unsigned integer (varint)
    pub fn write_varint(&mut self, mut value: u64) -> Result<()> {
        while value >= 0x80 {
            self.write_u8((value as u8) | 0x80)?;
            value >>= 7;
        }
        self.write_u8(value as u8)
    }

    /// Write raw bytes
    #[inline]
    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        self.data.extend_from_slice(bytes);
        Ok(())
    }

    /// Write a length-prefixed byte slice
    pub fn write_byte_slice(&mut self, bytes: &[u8]) -> Result<()> {
        self.write_varint(bytes.len() as u64)?;
        self.write_bytes(bytes)
    }

    /// Write a length-prefixed string
    pub fn write_str(&mut self, s: &str) -> Result<()> {
        self.write_byte_slice(s.as_bytes())
    }

    /// Get the current length of the buffer
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the buffer is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Reserve additional capacity
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional);
    }

    /// Get the initial capacity of the buffer
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get the buffer contents as a slice
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// Convert the buffer into a Vec<u8>
    pub fn into_vec(self) -> Vec<u8> {
        self.data
    }

    /// Clear the buffer, keeping the capacity
    pub fn clear(&mut self) {
        self.data.clear();
    }
}

impl Default for WriteBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// A high-performance read buffer for binary deserialization
#[derive(Debug)]
pub struct ReadBuffer<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> ReadBuffer<'a> {
    /// Create a new read buffer from a byte slice
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, position: 0 }
    }

    /// Read a single byte
    #[inline]
    pub fn read_u8(&mut self) -> Result<u8> {
        if self.position >= self.data.len() {
            return Err(Error::UnexpectedEof);
        }
        let value = self.data[self.position];
        self.position += 1;
        Ok(value)
    }

    /// Read a u16 in little-endian format
    #[inline]
    pub fn read_u16(&mut self) -> Result<u16> {
        let bytes = self.read_bytes(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    /// Read a u32 in little-endian format
    #[inline]
    pub fn read_u32(&mut self) -> Result<u32> {
        let bytes = self.read_bytes(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// Read a u64 in little-endian format
    #[inline]
    pub fn read_u64(&mut self) -> Result<u64> {
        let bytes = self.read_bytes(8)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    /// Read an i8
    #[inline]
    pub fn read_i8(&mut self) -> Result<i8> {
        Ok(self.read_u8()? as i8)
    }

    /// Read an i16 in little-endian format
    #[inline]
    pub fn read_i16(&mut self) -> Result<i16> {
        Ok(self.read_u16()? as i16)
    }

    /// Read an i32 in little-endian format
    #[inline]
    pub fn read_i32(&mut self) -> Result<i32> {
        Ok(self.read_u32()? as i32)
    }

    /// Read an i64 in little-endian format
    #[inline]
    pub fn read_i64(&mut self) -> Result<i64> {
        Ok(self.read_u64()? as i64)
    }

    /// Read an f32 in IEEE 754 format
    #[inline]
    pub fn read_f32(&mut self) -> Result<f32> {
        let bits = self.read_u32()?;
        Ok(f32::from_bits(bits))
    }

    /// Read an f64 in IEEE 754 format
    #[inline]
    pub fn read_f64(&mut self) -> Result<f64> {
        let bits = self.read_u64()?;
        Ok(f64::from_bits(bits))
    }

    /// Read a variable-length unsigned integer (varint)
    pub fn read_varint(&mut self) -> Result<u64> {
        let mut result = 0u64;
        let mut shift = 0;

        loop {
            if shift >= 64 {
                return Err(Error::InvalidFormat("Varint too long".to_string()));
            }

            let byte = self.read_u8()?;
            result |= ((byte & 0x7F) as u64) << shift;

            if byte & 0x80 == 0 {
                break;
            }

            shift += 7;
        }

        Ok(result)
    }

    /// Read a specific number of bytes
    #[inline]
    pub fn read_bytes(&mut self, len: usize) -> Result<&'a [u8]> {
        if self.position + len > self.data.len() {
            return Err(Error::UnexpectedEof);
        }
        let bytes = &self.data[self.position..self.position + len];
        self.position += len;
        Ok(bytes)
    }

    /// Read a length-prefixed byte slice
    pub fn read_byte_slice(&mut self) -> Result<&'a [u8]> {
        let len = self.read_varint()? as usize;
        self.read_bytes(len)
    }

    /// Read a length-prefixed string
    pub fn read_str(&mut self) -> Result<&'a str> {
        let bytes = self.read_byte_slice()?;
        core::str::from_utf8(bytes).map_err(|_| {
            Error::InvalidFormat("Invalid UTF-8 string".to_string())
        })
    }

    /// Get the current position in the buffer
    #[inline]
    pub fn position(&self) -> usize {
        self.position
    }

    /// Get the remaining bytes in the buffer
    #[inline]
    pub fn remaining(&self) -> usize {
        self.data.len() - self.position
    }

    /// Check if there are more bytes to read
    #[inline]
    pub fn has_remaining(&self) -> bool {
        self.position < self.data.len()
    }

    /// Peek at the next byte without advancing the position
    #[inline]
    pub fn peek_u8(&self) -> Result<u8> {
        if self.position >= self.data.len() {
            return Err(Error::UnexpectedEof);
        }
        Ok(self.data[self.position])
    }

    /// Skip a number of bytes
    #[inline]
    pub fn skip(&mut self, count: usize) -> Result<()> {
        if self.position + count > self.data.len() {
            return Err(Error::UnexpectedEof);
        }
        self.position += count;
        Ok(())
    }

    /// Get the underlying data slice
    #[inline]
    pub fn as_slice(&self) -> &'a [u8] {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_read_roundtrip() {
        let mut buf = WriteBuffer::new();

        // Write various types
        buf.write_u8(0x42).unwrap();
        buf.write_u16(0x1234).unwrap();
        buf.write_u32(0x12345678).unwrap();
        buf.write_u64(0x123456789ABCDEF0).unwrap();
        buf.write_i8(-42).unwrap();
        buf.write_i16(-1234).unwrap();
        buf.write_i32(-123456789).unwrap();
        buf.write_i64(-123456789012345).unwrap();
        buf.write_f32(3.14159).unwrap();
        buf.write_f64(2.718281828459045).unwrap();
        buf.write_str("Hello, NanoBit!").unwrap();

        // Read them back
        let mut reader = ReadBuffer::new(buf.as_slice());

        assert_eq!(reader.read_u8().unwrap(), 0x42);
        assert_eq!(reader.read_u16().unwrap(), 0x1234);
        assert_eq!(reader.read_u32().unwrap(), 0x12345678);
        assert_eq!(reader.read_u64().unwrap(), 0x123456789ABCDEF0);
        assert_eq!(reader.read_i8().unwrap(), -42);
        assert_eq!(reader.read_i16().unwrap(), -1234);
        assert_eq!(reader.read_i32().unwrap(), -123456789);
        assert_eq!(reader.read_i64().unwrap(), -123456789012345);
        assert!((reader.read_f32().unwrap() - 3.14159).abs() < f32::EPSILON);
        assert!((reader.read_f64().unwrap() - 2.718281828459045).abs() < f64::EPSILON);
        assert_eq!(reader.read_str().unwrap(), "Hello, NanoBit!");
    }

    #[test]
    fn test_varint_encoding() {
        let mut buf = WriteBuffer::new();
        
        // Test various varint values
        let values = [0, 127, 128, 255, 256, 16384, 2097151, 268435456, u64::MAX];
        
        for &value in &values {
            buf.write_varint(value).unwrap();
        }
        
        let mut reader = ReadBuffer::new(buf.as_slice());
        
        for &expected in &values {
            assert_eq!(reader.read_varint().unwrap(), expected);
        }
    }

    #[test]
    fn test_buffer_overflow_detection() {
        let data = [1, 2, 3];
        let mut reader = ReadBuffer::new(&data);

        // Should be able to read 3 bytes
        assert_eq!(reader.read_u8().unwrap(), 1);
        assert_eq!(reader.read_u8().unwrap(), 2);
        assert_eq!(reader.read_u8().unwrap(), 3);

        // Should fail on 4th byte
        assert!(reader.read_u8().is_err());
    }

    #[test]
    fn test_string_encoding() {
        let mut buf = WriteBuffer::new();
        
        let test_strings = ["", "hello", "世界", "🚀 NanoBit"];
        
        for s in &test_strings {
            buf.write_str(s).unwrap();
        }
        
        let mut reader = ReadBuffer::new(buf.as_slice());
        
        for expected in &test_strings {
            assert_eq!(reader.read_str().unwrap(), *expected);
        }
    }
}
