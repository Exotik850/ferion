use std::{
    error::Error,
    fmt::{Debug, Display},
};

use serde::{de::Visitor, forward_to_deserialize_any};

use crate::{
    types::{LeadByte, NormalRionType, RionFieldType, ShortRionType},
    RionField,
};

impl serde::de::Error for DeserializeError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        let msg = msg.to_string();
        DeserializeError::Custom(msg)
    }
}

impl std::error::Error for DeserializeError {}
impl Debug for DeserializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}
impl Display for DeserializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeserializeError::Eod => write!(f, "end of available data!")?,
            DeserializeError::InvalidData(data) => write!(f, "invalid data! {data:?}")?,
            DeserializeError::Custom(msg) => write!(f, "{}", msg)?,
            DeserializeError::ExpectedNull => write!(f, "expected null")?,
            DeserializeError::DataLength(expected, actual) => {
                write!(f, "expected data length {expected}, but got {actual}")?
            }
            DeserializeError::InvalidType(expected, actual) => {
                write!(f, "expected type {expected:?}, but got {actual:?}")?
            }
        }
        Ok(())
    }
}

impl From<String> for DeserializeError {
    fn from(err: String) -> Self {
        DeserializeError::Custom(err)
    }
}

impl From<Box<dyn Error>> for DeserializeError {
    fn from(err: Box<dyn Error>) -> Self {
        DeserializeError::Custom(err.to_string())
    }
}

// impl<T: Display> From<T> for DeserializeError {
//     fn from(err: T) -> Self {
//         DeserializeError::Custom(err.to_string())
//     }
// }

pub fn from_bytes<'de, T>(data: &'de [u8]) -> Result<T, DeserializeError>
where
    T: serde::de::Deserialize<'de>,
{
    let mut deserializer = Deserializer::new(data);
    T::deserialize(&mut deserializer)
}

// #[derive(Debug)]
pub enum DeserializeError {
    Eod,
    DataLength(usize, usize),                  // Expected, Actual
    InvalidType(RionFieldType, RionFieldType), // Expected, Actual
    ExpectedNull,
    InvalidData(Vec<u8>),
    Custom(String),
}

pub struct Deserializer<'de> {
    data: &'de [u8],
}

impl<'de> Deserializer<'de> {
    pub fn new(data: &'de [u8]) -> Self {
        Self { data }
    }

    fn parse_next_field(&mut self) -> Result<RionField<'de>, DeserializeError> {
        let (field, rest) = RionField::parse(self.data)
            .map_err(|_| DeserializeError::InvalidData(self.data.to_vec()))?;
        self.data = rest;
        Ok(field)
    }

    fn parse_field<T>(&mut self) -> Result<T, DeserializeError>
    where
        T: TryFrom<RionField<'de>, Error: Display>,
    {
        let field = self.parse_next_field()?;
        println!("{:?}", field);
        Ok(field
            .try_into()
            .map_err(|e: T::Error| DeserializeError::Custom(e.to_string()))?)
    }

    fn visit_field<V>(&mut self, field: RionField, visitor: V) -> Result<V::Value, DeserializeError>
    where
        V: Visitor<'de>,
    {
        match field.field_type() {
            RionFieldType::Normal(NormalRionType::Bytes) => visitor.visit_bytes(field.as_bytes()),
            RionFieldType::Normal(NormalRionType::UTF8 | NormalRionType::Key)
            | RionFieldType::Short(ShortRionType::Key | ShortRionType::UTF8) => {
                visitor.visit_str(field.as_str().unwrap())
            }
            RionFieldType::Normal(NormalRionType::Array) => visitor.visit_seq(self),
            RionFieldType::Normal(NormalRionType::Object | NormalRionType::Table) => {
                visitor.visit_map(self) // TODO: Properly handle table
            }
            RionFieldType::Tiny(lead) => visitor.visit_bool(lead.length() == 2),
            RionFieldType::Short(ShortRionType::Int64Negative) => {
                visitor.visit_i64(field.try_into().unwrap())
            }
            RionFieldType::Short(ShortRionType::Int64Positive) => {
                visitor.visit_u64(field.try_into().unwrap())
            }
            RionFieldType::Short(ShortRionType::Float) => {
                let RionField::Short(short) = field else {
                    unreachable!()
                };
                match short.data_len {
                    ..=4 => visitor.visit_f32(short.as_f32().unwrap()),
                    ..=8 => visitor.visit_f64(short.as_f64().unwrap()),
                    _ => Err(DeserializeError::DataLength(8, short.data_len as usize)),
                }
            }
            RionFieldType::Short(ShortRionType::UTCDateTime) => {
                todo!()
            }
            _ => Err(DeserializeError::InvalidData(
                field.to_data().unwrap_or_default().to_vec(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_deserialize_uint() {
        let data = vec![0x21, 0x0A]; // 10
        let value: u64 = from_bytes(&data).unwrap();
        assert_eq!(value, 10);
    }

    #[test]
    fn test_deserialize_string() {
        let data = vec![0xD1, 0x05, b'A', b'l', b'i', b'c', b'e'];
        let name: String = from_bytes(&data).unwrap();
        assert_eq!(name, "Alice");
    }

    #[test]
    fn test_deserialize_map() {
        let data = vec![
            0xC1, 0x0A, 0xE3, b'K', b'e', b'y', 0x65, b'V', b'a', b'l', b'u', b'e',
        ];
        let map: std::collections::HashMap<String, String> = from_bytes(&data).unwrap();
        println!("{:?}", map);
        assert_eq!(map.get("Key").unwrap(), "Value");
    }

    #[test]
    fn test_deserialize_integers() {
        let data = vec![0x21, 0x7F]; // 127 (i8 max)
        let value: u8 = from_bytes(&data).unwrap();
        assert_eq!(value, 127);

        let data = vec![0x31, 0x7F]; // -128 (i8 min)
        let value: i8 = from_bytes(&data).unwrap();
        assert_eq!(value, i8::MIN);

        let data = vec![0x22, 0x7F, 0xFF]; // 32767 (i16 max)
        let value: i16 = from_bytes(&data).unwrap();
        assert_eq!(value, 32767);

        let data = vec![0x24, 0x7F, 0xFF, 0xFF, 0xFF]; // 2147483647 (i32 max)
        let value: i32 = from_bytes(&data).unwrap();
        assert_eq!(value, 2147483647);
    }

    #[test]
    fn test_deserialize_float() {
        let data = vec![0x44, 0x40, 0x48, 0xF5, 0xC3]; // 3.14 (f32)
        let value: f32 = from_bytes(&data).unwrap();
        assert!((value - 3.14).abs() < f32::EPSILON);

        let data = vec![0x48, 0x40, 0x09, 0x21, 0xFB, 0x54, 0x44, 0x2D, 0x11]; // 3.14159265358979 (f64)
        let value: f64 = from_bytes(&data).unwrap();
        assert!((value - 3.14159265358979).abs() < f64::EPSILON);
    }

    #[test]
    fn test_deserialize_char() {
        let data = vec![0x61, b'A'];
        let value: char = from_bytes(&data).unwrap();
        assert_eq!(value, 'A');
    }

    #[test]
    fn test_deserialize_option() {
        let data = vec![0x00]; // null
        let value: Option<u32> = from_bytes(&data).unwrap();
        assert_eq!(value, None);

        let data = vec![0x21, 0x0A]; // Some(10)
        let value: Option<u32> = from_bytes(&data).unwrap();
        assert_eq!(value, Some(10));
    }

    use serde::Deserialize;
    #[derive(Deserialize)]
    struct Test {
        name: String,
        #[allow(dead_code)]
        age: u32,
    }

    #[test]
    fn test_deserialize_struct() {
        let data = vec![
            0xC1, 0x0F, // Start of object
            0xE4, b'n', b'a', b'm', b'e', 0x65, b'A', b'l', b'i', b'c', b'e', // name: "Alice"
            0xE3, b'a', b'g', b'e', 0x21, 0x1E, // age: 30
        ];
        let value: Test = from_bytes(&data).unwrap();
        assert_eq!(value.name, "Alice");
    }

    // Nested structs
    #[derive(Deserialize, Debug, PartialEq)]
    struct Address {
        street: String,
        city: String,
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct User {
        name: String,
        age: u32,
        address: Address,
    }

    #[test]
    fn test_deserialize_nested_struct() {
        let data = vec![
            0xC1, 0x35, // Start of object
            0xE4, b'n', b'a', b'm', b'e', 0x65, b'A', b'l', b'i', b'c', b'e', // name: "Alice"
            0xE3, b'a', b'g', b'e', 0x21, 0x1E, // age: 30
            0xE7, b'a', b'd', b'd', b'r', b'e', b's', b's', 0xC1, 0x1A, // address: { ... }
            0xE6, b's', b't', b'r', b'e', b'e', b't', 0x68, b'1', b'2', b'3', b' ', b'M', b'a',
            b'i', b'n', // street: "123 Main"
            0xE4, b'c', b'i', b't', b'y', 0x64, b'S', b'o', b'm', b'e', // city: "Some"
        ];
        println!("{:?}", data.len());
        let value: User = from_bytes(&data).unwrap();
        assert_eq!(value.name, "Alice");
        assert_eq!(value.age, 30);
        assert_eq!(value.address.street, "123 Main");
        assert_eq!(value.address.city, "Some");
    }

    #[test]
    fn test_deserialize_tuple() {
        let data = vec![0xA1, 0x04, 0x21, 0x0A, 0x61, b'A']; // (10, 'A')
        let value: (u8, char) = from_bytes(&data).unwrap();
        assert_eq!(value, (10, 'A'));
    }

    // TODO Make deserialization of Vec<u8> accept Normal::Bytes as well, wasting a lot of space
    #[test]
    fn test_deserialize_bytes() {
        let data = vec![0xA1, 0x0A, 0x21, 0x01, 0x21, 0x02, 0x21, 0x03, 0x21, 0x04, 0x21, 0x05];
        let value: Vec<u8> = from_bytes(&data).unwrap();
        assert_eq!(value, vec![1, 2, 3, 4, 5]);
    }
}

impl<'de, 'a> serde::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = DeserializeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let field = self.parse_next_field()?;
        if field.is_null() {
            return visitor.visit_none();
        }
        self.visit_field(field, visitor)
    }

    forward_to_deserialize_any! {
      bool i64 u64 bytes f32 f64 str
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let value: _ = self.parse_field()?;
        visitor.visit_i8(value)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let value: _ = self.parse_field()?;
        visitor.visit_i16(value)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let value: _ = self.parse_field()?;
        visitor.visit_i32(value)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let value: _ = self.parse_field()?;
        visitor.visit_u8(value)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let value: _ = self.parse_field()?;
        visitor.visit_u16(value)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let value: _ = self.parse_field()?;
        visitor.visit_u32(value)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_char(self.parse_field()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_string(self.parse_field()?)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let field = self.parse_next_field()?;
        let field_type = field.field_type();
        let RionFieldType::Normal(NormalRionType::Bytes) = field_type else {
            return Err(DeserializeError::InvalidType(
                RionFieldType::Normal(NormalRionType::Bytes),
                field_type,
            ));
        };
        visitor.visit_byte_buf(field.as_bytes().to_vec())
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let Some(first) = self.data.get(0) else {
            return Err(DeserializeError::Eod);
        };
        let lead = LeadByte::try_from(*first)?;
        if lead.is_null() {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let field = self.parse_next_field()?;
        if field.is_null() {
            visitor.visit_unit()
        } else {
            Err(DeserializeError::ExpectedNull)
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let (lead, length, rest) =
            crate::get_normal_header(self.data).map_err(|e| e.to_string())?;
        let field_type = lead.field_type();
        let RionFieldType::Normal(NormalRionType::Array) = field_type else {
            return Err(DeserializeError::InvalidType(
                RionFieldType::Normal(NormalRionType::Array),
                field_type,
            ));
        };
        self.data = rest;
        if self.data.len() < length {
            return Err(DeserializeError::DataLength(length, self.data.len()));
        }
        visitor.visit_seq(self)
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let (lead, length, rest) =
            crate::get_normal_header(self.data).map_err(|e| e.to_string())?;
        let field_type = lead.field_type();
        let RionFieldType::Normal(NormalRionType::Object) = field_type else {
            return Err(DeserializeError::InvalidType(
                RionFieldType::Normal(NormalRionType::Object),
                field_type,
            ));
        };
        self.data = rest;
        if self.data.len() < length {
            return Err(DeserializeError::DataLength(length, self.data.len()));
        }
        visitor.visit_map(self)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

impl<'de, 'a> serde::de::SeqAccess<'de> for &'a mut Deserializer<'de> {
    type Error = DeserializeError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        if self.data.is_empty() {
            return Ok(None);
        }
        let value = seed.deserialize(&mut **self)?;
        Ok(Some(value))
    }
}

impl<'de, 'a> serde::de::MapAccess<'de> for &'a mut Deserializer<'de> {
    type Error = DeserializeError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        if self.data.is_empty() {
            return Ok(None);
        }
        // If the next byte is not a key, return None
        let field_val = self.data[0] & 0xF0;
        let field_type = RionFieldType::try_from(field_val)?;
        if !field_type.is_key() {
            return Ok(None);
        }
        let key = seed.deserialize(&mut **self)?;
        Ok(Some(key))
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let value = seed.deserialize(&mut **self)?;
        Ok(value)
    }
}
