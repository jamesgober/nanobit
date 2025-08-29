//! Binary serialization implementation

#[cfg(not(feature = "std"))]
use alloc::{vec::Vec, string::String};

#[cfg(feature = "std")]
use std::io::Write;

use serde::ser::{
    Serialize, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
    SerializeTuple, SerializeTupleStruct, SerializeTupleVariant,
};

use crate::buffer::WriteBuffer;
use crate::error::{Error, Result};

/// High-performance binary serializer
pub struct Serializer {
    buffer: WriteBuffer,
}

impl Serializer {
    /// Create a new serializer with default capacity
    pub fn new() -> Self {
        Self {
            buffer: WriteBuffer::new(),
        }
    }

    /// Create a new serializer with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: WriteBuffer::with_capacity(capacity),
        }
    }

    /// Finalize serialization and return the bytes
    pub fn into_bytes(self) -> Vec<u8> {
        // Write header: magic bytes + version
        let mut result = Vec::with_capacity(self.buffer.len() + 5);
        result.extend_from_slice(crate::MAGIC);
        result.push(crate::VERSION);
        result.extend_from_slice(self.buffer.as_slice());
        result
    }
}

impl Default for Serializer {
    fn default() -> Self {
        Self::new()
    }
}

impl serde::Serializer for &mut Serializer {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    #[inline]
    fn serialize_bool(self, v: bool) -> Result<()> {
        self.buffer.write_u8(if v { 1 } else { 0 })
    }

    #[inline]
    fn serialize_i8(self, v: i8) -> Result<()> {
        self.buffer.write_i8(v)
    }

    #[inline]
    fn serialize_i16(self, v: i16) -> Result<()> {
        self.buffer.write_i16(v)
    }

    #[inline]
    fn serialize_i32(self, v: i32) -> Result<()> {
        self.buffer.write_i32(v)
    }

    #[inline]
    fn serialize_i64(self, v: i64) -> Result<()> {
        self.buffer.write_i64(v)
    }

    #[inline]
    fn serialize_u8(self, v: u8) -> Result<()> {
        self.buffer.write_u8(v)
    }

    #[inline]
    fn serialize_u16(self, v: u16) -> Result<()> {
        self.buffer.write_u16(v)
    }

    #[inline]
    fn serialize_u32(self, v: u32) -> Result<()> {
        self.buffer.write_u32(v)
    }

    #[inline]
    fn serialize_u64(self, v: u64) -> Result<()> {
        self.buffer.write_u64(v)
    }

    #[inline]
    fn serialize_f32(self, v: f32) -> Result<()> {
        self.buffer.write_f32(v)
    }

    #[inline]
    fn serialize_f64(self, v: f64) -> Result<()> {
        self.buffer.write_f64(v)
    }

    #[inline]
    fn serialize_char(self, v: char) -> Result<()> {
        // Encode char as u32
        self.serialize_u32(v as u32)
    }

    #[inline]
    fn serialize_str(self, v: &str) -> Result<()> {
        self.buffer.write_str(v)
    }

    #[inline]
    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.buffer.write_byte_slice(v)
    }

    #[inline]
    fn serialize_none(self) -> Result<()> {
        self.serialize_u8(0)
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: Serialize + ?Sized,
    {
        self.serialize_u8(1)?;
        value.serialize(self)
    }

    #[inline]
    fn serialize_unit(self) -> Result<()> {
        Ok(())
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        self.buffer.write_varint(variant_index as u64)
    }

    #[inline]
    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: Serialize + ?Sized,
    {
        self.buffer.write_varint(variant_index as u64)?;
        value.serialize(self)
    }

    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        match len {
            Some(len) => self.buffer.write_varint(len as u64)?,
            None => return Err(Error::Serde("Sequences must have known length".to_string())),
        }
        Ok(self)
    }

    #[inline]
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.buffer.write_varint(len as u64)?;
        Ok(self)
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_tuple(len)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.buffer.write_varint(variant_index as u64)?;
        self.buffer.write_varint(len as u64)?;
        Ok(self)
    }

    #[inline]
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        match len {
            Some(len) => self.buffer.write_varint(len as u64)?,
            None => return Err(Error::Serde("Maps must have known length".to_string())),
        }
        Ok(self)
    }

    #[inline]
    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct> {
        self.buffer.write_varint(len as u64)?;
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.buffer.write_varint(variant_index as u64)?;
        self.buffer.write_varint(len as u64)?;
        Ok(self)
    }
}

// Implementations for compound serialization types
impl SerializeSeq for &mut Serializer {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl SerializeTuple for &mut Serializer {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl SerializeTupleStruct for &mut Serializer {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl SerializeTupleVariant for &mut Serializer {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl SerializeMap for &mut Serializer {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: Serialize + ?Sized,
    {
        key.serialize(&mut **self)
    }

    #[inline]
    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl SerializeStruct for &mut Serializer {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl SerializeStructVariant for &mut Serializer {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

/// Serialize a value to bytes
pub fn to_bytes<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    let mut serializer = Serializer::new();
    value.serialize(&mut serializer)?;
    Ok(serializer.into_bytes())
}

/// Serialize a value to a writer
#[cfg(feature = "std")]
pub fn to_writer<W, T>(writer: W, value: &T) -> Result<()>
where
    W: Write,
    T: Serialize,
{
    let bytes = to_bytes(value)?;
    let mut writer = writer;
    writer.write_all(&bytes).map_err(Error::from)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    struct TestStruct {
        name: String,
        age: u32,
        active: bool,
        scores: Vec<f64>,
    }

    #[test]
    fn test_primitive_serialization() {
        assert!(to_bytes(&42u32).is_ok());
        assert!(to_bytes(&-100i64).is_ok());
        assert!(to_bytes(&3.14f64).is_ok());
        assert!(to_bytes(&true).is_ok());
        assert!(to_bytes(&"hello").is_ok());
    }

    #[test]
    fn test_struct_serialization() {
        let test_data = TestStruct {
            name: "Alice".to_string(),
            age: 30,
            active: true,
            scores: vec![95.5, 87.2, 92.1],
        };

        let result = to_bytes(&test_data);
        assert!(result.is_ok());

        let bytes = result.unwrap();
        
        // Should start with magic bytes and version
        assert_eq!(&bytes[0..4], crate::MAGIC);
        assert_eq!(bytes[4], crate::VERSION);
    }

    #[test]
    fn test_collections() {
        let vec_data = vec![1u32, 2, 3, 4, 5];
        let result = to_bytes(&vec_data);
        assert!(result.is_ok());

        let map_data = std::collections::HashMap::from([
            ("key1", "value1"),
            ("key2", "value2"),
        ]);
        let result = to_bytes(&map_data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_option_serialization() {
        let some_value: Option<u32> = Some(42);
        let none_value: Option<u32> = None;

        assert!(to_bytes(&some_value).is_ok());
        assert!(to_bytes(&none_value).is_ok());
    }

    #[test]
    fn test_enum_serialization() {
        #[derive(Serialize)]
        enum TestEnum {
            Variant1,
            Variant2(u32),
            Variant3 { field: String },
        }

        assert!(to_bytes(&TestEnum::Variant1).is_ok());
        assert!(to_bytes(&TestEnum::Variant2(42)).is_ok());
        assert!(to_bytes(&TestEnum::Variant3 { field: "test".to_string() }).is_ok());
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_writer_serialization() {
        let data = vec![1u32, 2, 3, 4, 5];
        let mut buffer = Vec::new();
        
        let result = to_writer(&mut buffer, &data);
        assert!(result.is_ok());
        assert!(!buffer.is_empty());
    }
}
