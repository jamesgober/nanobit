//! Binary deserialization implementation

#[cfg(not(feature = "std"))]
use alloc::{format, string::ToString};

#[cfg(feature = "std")]
use std::io::Read;

use serde::de::{
    Deserialize, DeserializeSeed, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor,
};

use crate::buffer::ReadBuffer;
use crate::error::{Error, Result};

/// High-performance binary deserializer
///
/// Nested values (options, sequences, maps, structs, enums and newtype
/// structs) are limited to [`DEFAULT_MAX_DEPTH`](crate::DEFAULT_MAX_DEPTH)
/// levels so that deeply nested input returns an error instead of
/// overflowing the stack. Use [`Deserializer::with_max_depth`] to change it.
pub struct Deserializer<'de> {
    reader: ReadBuffer<'de>,
    max_depth: usize,
    remaining_depth: usize,
}

impl<'de> Deserializer<'de> {
    /// Create a new deserializer from bytes
    pub fn new(data: &'de [u8]) -> Result<Self> {
        // Verify header
        if data.len() < 5 {
            return Err(Error::InvalidFormat(
                "Data too short for header".to_string(),
            ));
        }

        // Check magic bytes
        if &data[0..4] != crate::MAGIC {
            return Err(Error::InvalidFormat("Invalid magic bytes".to_string()));
        }

        // Check version
        let version = data[4];
        if version != crate::VERSION {
            return Err(Error::UnsupportedVersion(version));
        }

        // Create reader starting after header
        let reader = ReadBuffer::new(&data[5..]);

        Ok(Self {
            reader,
            max_depth: crate::DEFAULT_MAX_DEPTH,
            remaining_depth: crate::DEFAULT_MAX_DEPTH,
        })
    }

    /// Set how many levels of nesting the input may use.
    ///
    /// ```
    /// use serde::Deserialize;
    ///
    /// let bytes = nanobit::to_bytes(&vec![vec![1u8]]).unwrap();
    /// let mut de = nanobit::Deserializer::new(&bytes).unwrap().with_max_depth(1);
    /// assert!(Vec::<Vec<u8>>::deserialize(&mut de).is_err());
    /// ```
    #[must_use]
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self.remaining_depth = max_depth;
        self
    }

    /// The nesting limit in force for this deserializer.
    pub fn max_depth(&self) -> usize {
        self.max_depth
    }

    fn enter(&mut self) -> Result<()> {
        if self.remaining_depth == 0 {
            return Err(Error::InvalidFormat(format!(
                "Input nested deeper than {} levels",
                self.max_depth
            )));
        }
        self.remaining_depth -= 1;
        Ok(())
    }

    fn leave(&mut self) {
        self.remaining_depth += 1;
    }

    fn nested<T>(&mut self, f: impl FnOnce(&mut Self) -> Result<T>) -> Result<T> {
        self.enter()?;
        let result = f(self);
        self.leave();
        result
    }
}

impl<'de> serde::Deserializer<'de> for &mut Deserializer<'de> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::Serde("deserialize_any is not supported".to_string()))
    }

    #[inline]
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.reader.read_u8()?;
        visitor.visit_bool(value != 0)
    }

    #[inline]
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(self.reader.read_i8()?)
    }

    #[inline]
    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(self.reader.read_i16()?)
    }

    #[inline]
    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.reader.read_i32()?)
    }

    #[inline]
    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.reader.read_i64()?)
    }

    #[inline]
    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.reader.read_u8()?)
    }

    #[inline]
    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(self.reader.read_u16()?)
    }

    #[inline]
    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(self.reader.read_u32()?)
    }

    #[inline]
    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.reader.read_u64()?)
    }

    #[inline]
    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f32(self.reader.read_f32()?)
    }

    #[inline]
    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(self.reader.read_f64()?)
    }

    #[inline]
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.reader.read_u32()?;
        let ch = char::from_u32(value)
            .ok_or_else(|| Error::InvalidFormat("Invalid char value".to_string()))?;
        visitor.visit_char(ch)
    }

    #[inline]
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let s = self.reader.read_str()?;
        visitor.visit_borrowed_str(s)
    }

    #[inline]
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    #[inline]
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let bytes = self.reader.read_byte_slice()?;
        visitor.visit_borrowed_bytes(bytes)
    }

    #[inline]
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let tag = self.reader.read_u8()?;
        match tag {
            0 => visitor.visit_none(),
            1 => self.nested(|de| visitor.visit_some(de)),
            _ => Err(Error::InvalidFormat("Invalid option tag".to_string())),
        }
    }

    #[inline]
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    #[inline]
    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    #[inline]
    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.nested(|de| visitor.visit_newtype_struct(de))
    }

    #[inline]
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = self.reader.read_len()?;
        self.nested(|de| visitor.visit_seq(SeqDeserializer::new(de, len)))
    }

    #[inline]
    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let expected_len = self.reader.read_len()?;
        if expected_len != len {
            return Err(Error::InvalidFormat(format!(
                "Tuple length mismatch: expected {len}, got {expected_len}"
            )));
        }
        self.nested(|de| visitor.visit_seq(SeqDeserializer::new(de, len)))
    }

    #[inline]
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    #[inline]
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = self.reader.read_len()?;
        self.nested(|de| visitor.visit_map(MapDeserializer::new(de, len)))
    }

    #[inline]
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = self.reader.read_len()?;
        if len != fields.len() {
            return Err(Error::InvalidFormat(format!(
                "Struct field count mismatch: expected {}, got {}",
                fields.len(),
                len
            )));
        }
        self.nested(|de| visitor.visit_seq(SeqDeserializer::new(de, len)))
    }

    #[inline]
    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.nested(|de| visitor.visit_enum(EnumDeserializer::new(de)))
    }

    #[inline]
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    #[inline]
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

// Sequence deserializer for arrays, tuples, etc.
struct SeqDeserializer<'a, 'de> {
    de: &'a mut Deserializer<'de>,
    remaining: usize,
}

impl<'a, 'de> SeqDeserializer<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, len: usize) -> Self {
        Self { de, remaining: len }
    }
}

impl<'de, 'a> SeqAccess<'de> for SeqDeserializer<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }
        self.remaining -= 1;
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.remaining)
    }
}

// Map deserializer for objects, dictionaries, etc.
struct MapDeserializer<'a, 'de> {
    de: &'a mut Deserializer<'de>,
    remaining: usize,
}

impl<'a, 'de> MapDeserializer<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, len: usize) -> Self {
        Self { de, remaining: len }
    }
}

impl<'de, 'a> MapAccess<'de> for MapDeserializer<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }
        self.remaining -= 1;
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.remaining)
    }
}

// Enum deserializer
struct EnumDeserializer<'a, 'de> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> EnumDeserializer<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Self { de }
    }
}

impl<'de, 'a> EnumAccess<'de> for EnumDeserializer<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        let variant_index = self.de.reader.read_varint()?;
        let val = seed.deserialize(variant_index.into_deserializer())?;
        Ok((val, self))
    }
}

impl<'de, 'a> VariantAccess<'de> for EnumDeserializer<'a, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let actual_len = self.de.reader.read_len()?;
        if actual_len != len {
            return Err(Error::InvalidFormat(format!(
                "Tuple variant length mismatch: expected {len}, got {actual_len}"
            )));
        }
        serde::Deserializer::deserialize_tuple(self.de, len, visitor)
    }

    fn struct_variant<V>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = self.de.reader.read_len()?;
        if len != fields.len() {
            return Err(Error::InvalidFormat(format!(
                "Struct variant field count mismatch: expected {}, got {}",
                fields.len(),
                len
            )));
        }
        visitor.visit_seq(SeqDeserializer::new(self.de, len))
    }
}

// Helper for creating deserializer from primitive values
struct PrimitiveDeserializer<T> {
    value: T,
}

impl<T> PrimitiveDeserializer<T> {
    fn new(value: T) -> Self {
        Self { value }
    }
}

impl<'de> serde::Deserializer<'de> for PrimitiveDeserializer<u64> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.value)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

trait IntoDeserializer<'de> {
    type Deserializer: serde::Deserializer<'de>;

    fn into_deserializer(self) -> Self::Deserializer;
}

impl<'de> IntoDeserializer<'de> for u64 {
    type Deserializer = PrimitiveDeserializer<u64>;

    fn into_deserializer(self) -> Self::Deserializer {
        PrimitiveDeserializer::new(self)
    }
}

/// Deserialize from bytes
pub fn from_bytes<'de, T>(bytes: &'de [u8]) -> Result<T>
where
    T: Deserialize<'de>,
{
    let mut deserializer = Deserializer::new(bytes)?;
    T::deserialize(&mut deserializer)
}

/// Deserialize from a reader
///
/// This reads the whole stream into memory first, with no size limit. For a
/// stream you do not control, use [`from_reader_with_limit`].
#[cfg(feature = "std")]
pub fn from_reader<R, T>(reader: R) -> Result<T>
where
    R: Read,
    T: for<'de> Deserialize<'de>,
{
    let mut reader = reader;
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer).map_err(Error::from)?;

    // We need to work with owned data for the reader case
    let mut deserializer = Deserializer::new(&buffer)?;
    T::deserialize(&mut deserializer)
}

/// Deserialize from a reader, reading at most `max_bytes` bytes.
///
/// Returns [`Error::InvalidFormat`] if the stream holds more than
/// `max_bytes` bytes.
#[cfg(feature = "std")]
pub fn from_reader_with_limit<R, T>(reader: R, max_bytes: usize) -> Result<T>
where
    R: Read,
    T: for<'de> Deserialize<'de>,
{
    let mut buffer = Vec::new();
    let limit = u64::try_from(max_bytes)
        .unwrap_or(u64::MAX)
        .saturating_add(1);
    reader
        .take(limit)
        .read_to_end(&mut buffer)
        .map_err(Error::from)?;
    if buffer.len() > max_bytes {
        return Err(Error::InvalidFormat(format!(
            "Input is larger than the {max_bytes} byte limit"
        )));
    }

    let mut deserializer = Deserializer::new(&buffer)?;
    T::deserialize(&mut deserializer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ser::to_bytes;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestStruct {
        name: String,
        age: u32,
        active: bool,
        scores: Vec<f64>,
    }

    #[test]
    fn test_primitive_deserialization() {
        assert_eq!(
            from_bytes::<u32>(&to_bytes(&42u32).unwrap()).unwrap(),
            42u32
        );
        assert_eq!(
            from_bytes::<i64>(&to_bytes(&-100i64).unwrap()).unwrap(),
            -100i64
        );
        assert_eq!(
            from_bytes::<f64>(&to_bytes(&2.5f64).unwrap()).unwrap(),
            2.5f64
        );
        assert!(from_bytes::<bool>(&to_bytes(&true).unwrap()).unwrap());
        assert_eq!(
            from_bytes::<&str>(&to_bytes(&"hello").unwrap()).unwrap(),
            "hello"
        );
    }

    #[test]
    fn test_struct_roundtrip() {
        let original = TestStruct {
            name: "Alice".to_string(),
            age: 30,
            active: true,
            scores: vec![95.5, 87.2, 92.1],
        };

        let serialized = to_bytes(&original).unwrap();
        let deserialized: TestStruct = from_bytes(&serialized).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_collections() {
        let vec_data = vec![1u32, 2, 3, 4, 5];
        let serialized = to_bytes(&vec_data).unwrap();
        let deserialized: Vec<u32> = from_bytes(&serialized).unwrap();
        assert_eq!(vec_data, deserialized);

        let map_data = std::collections::HashMap::from([
            ("key1".to_string(), "value1".to_string()),
            ("key2".to_string(), "value2".to_string()),
        ]);
        let serialized = to_bytes(&map_data).unwrap();
        let deserialized: std::collections::HashMap<String, String> =
            from_bytes(&serialized).unwrap();
        assert_eq!(map_data, deserialized);
    }

    #[test]
    fn test_option_roundtrip() {
        let some_value: Option<u32> = Some(42);
        let none_value: Option<u32> = None;

        let serialized_some = to_bytes(&some_value).unwrap();
        let serialized_none = to_bytes(&none_value).unwrap();

        let deserialized_some: Option<u32> = from_bytes(&serialized_some).unwrap();
        let deserialized_none: Option<u32> = from_bytes(&serialized_none).unwrap();

        assert_eq!(some_value, deserialized_some);
        assert_eq!(none_value, deserialized_none);
    }

    #[test]
    fn test_enum_roundtrip() {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        enum TestEnum {
            Variant1,
            Variant2(u32),
            Variant3 { field: String },
        }

        let variants = vec![
            TestEnum::Variant1,
            TestEnum::Variant2(42),
            TestEnum::Variant3 {
                field: "test".to_string(),
            },
        ];

        for variant in variants {
            let serialized = to_bytes(&variant).unwrap();
            let deserialized: TestEnum = from_bytes(&serialized).unwrap();
            assert_eq!(variant, deserialized);
        }
    }

    #[test]
    fn test_invalid_data() {
        // Test with data too short
        let result: Result<u32> = from_bytes(&[]);
        assert!(result.is_err());

        // Test with wrong magic bytes
        let wrong_magic = [b'X', b'Y', b'Z', b'W', 1, 42, 0, 0, 0];
        let result: Result<u32> = from_bytes(&wrong_magic);
        assert!(result.is_err());

        // Test with wrong version
        let wrong_version = [b'N', b'A', b'N', b'O', 99, 42, 0, 0, 0];
        let result: Result<u32> = from_bytes(&wrong_version);
        assert!(result.is_err());
    }

    #[test]
    fn test_char_roundtrip() {
        let chars = ['a', '世', '🚀', '\0'];
        for ch in chars {
            let serialized = to_bytes(&ch).unwrap();
            let deserialized: char = from_bytes(&serialized).unwrap();
            assert_eq!(ch, deserialized);
        }
    }

    #[test]
    fn hostile_string_length_is_an_error() {
        let mut input = b"NANO".to_vec();
        input.extend_from_slice(&[0xff; 9]);
        input.push(0x01);
        assert_eq!(from_bytes::<String>(&input), Err(Error::UnexpectedEof));
        assert_eq!(from_bytes::<&str>(&input), Err(Error::UnexpectedEof));
        assert_eq!(from_bytes::<&[u8]>(&input), Err(Error::UnexpectedEof));
    }

    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    enum Nest {
        End,
        Next(Box<Nest>),
    }

    fn nested_input(levels: usize) -> Vec<u8> {
        let mut input = b"NANO".to_vec();
        input.extend_from_slice(&vec![1u8; levels]);
        input.push(0);
        input
    }

    #[test]
    fn deep_nesting_is_an_error_not_a_stack_overflow() {
        let result = from_bytes::<Nest>(&nested_input(1_000_000));
        assert!(matches!(result, Err(Error::InvalidFormat(_))), "{result:?}");
    }

    #[test]
    fn nesting_within_the_limit_still_works() {
        let mut value = Nest::End;
        for _ in 0..crate::DEFAULT_MAX_DEPTH - 1 {
            value = Nest::Next(Box::new(value));
        }
        let bytes = to_bytes(&value).unwrap();
        assert_eq!(from_bytes::<Nest>(&bytes).unwrap(), value);

        let too_deep = Nest::Next(Box::new(value));
        let bytes = to_bytes(&too_deep).unwrap();
        assert!(from_bytes::<Nest>(&bytes).is_err());

        let mut de = Deserializer::new(&bytes).unwrap().with_max_depth(200);
        assert_eq!(de.max_depth(), 200);
        assert_eq!(Nest::deserialize(&mut de).unwrap(), too_deep);
    }

    #[test]
    fn depth_counts_every_kind_of_nesting() {
        let bytes = to_bytes(&Some(Some(1u8))).unwrap();
        let mut de = Deserializer::new(&bytes).unwrap().with_max_depth(1);
        assert!(Option::<Option<u8>>::deserialize(&mut de).is_err());

        let bytes = to_bytes(&vec![vec![1u8]]).unwrap();
        let mut de = Deserializer::new(&bytes).unwrap().with_max_depth(2);
        assert_eq!(
            Vec::<Vec<u8>>::deserialize(&mut de).unwrap(),
            vec![vec![1u8]]
        );

        // Sibling values do not add up: depth is restored on the way out.
        let wide = vec![vec![1u8]; 1000];
        let bytes = to_bytes(&wide).unwrap();
        let mut de = Deserializer::new(&bytes).unwrap().with_max_depth(2);
        assert_eq!(Vec::<Vec<u8>>::deserialize(&mut de).unwrap(), wide);
    }

    #[cfg(feature = "std")]
    #[test]
    fn reader_limit_is_enforced() {
        let data = vec![7u8; 100];
        let serialized = to_bytes(&data).unwrap();
        let ok: Vec<u8> =
            from_reader_with_limit(std::io::Cursor::new(&serialized), serialized.len()).unwrap();
        assert_eq!(ok, data);
        let err = from_reader_with_limit::<_, Vec<u8>>(
            std::io::Cursor::new(&serialized),
            serialized.len() - 1,
        );
        assert!(matches!(err, Err(Error::InvalidFormat(_))));
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_reader_deserialization() {
        let data = vec![1u32, 2, 3, 4, 5];
        let serialized = to_bytes(&data).unwrap();
        let cursor = std::io::Cursor::new(serialized);

        let deserialized: Vec<u32> = from_reader(cursor).unwrap();
        assert_eq!(data, deserialized);
    }
}
