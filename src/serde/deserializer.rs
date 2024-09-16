use std::{error::Error, fmt::Display};

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
impl Display for DeserializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeserializeError::Eod => write!(f, "end of available data!")?,
            DeserializeError::InvalidData => write!(f, "invalid data!")?,
            DeserializeError::Custom(msg) => write!(f, "{}", msg)?,
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
    println!("{:?}", deserializer.data);
    T::deserialize(&mut deserializer)
}

#[derive(Debug)]
pub enum DeserializeError {
    Eod,
    InvalidData,
    Custom(String),
}

struct Deserializer<'de> {
    data: &'de [u8],
}

impl<'de> Deserializer<'de> {
    fn new(data: &'de [u8]) -> Self {
        Self { data }
    }

    fn peek_type(&self) -> Option<RionFieldType> {
        RionFieldType::try_from(self.data[0]).ok()
    }

    fn parse_next_field(&mut self) -> Result<RionField<'de>, DeserializeError> {
        let (field, rest) =
            RionField::parse(self.data).map_err(|_| DeserializeError::InvalidData)?;
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
                visitor.visit_map(self)
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
                    _ => Err(DeserializeError::InvalidData),
                }
            }
            RionFieldType::Short(ShortRionType::UTCDateTime) => {
                todo!()
            }
            _ => Err(DeserializeError::InvalidData),
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

    // #[test]
    // fn test_deserialize_float() {
    //     let data = vec![0x44, 0x40, 0x48, 0xF5, 0xC3]; // 3.14 (f32)
    //     let value: f32 = from_bytes(&data).unwrap();
    //     assert!((value - 3.14).abs() < f32::EPSILON);

    //     let data = vec![0x48, 0x40, 0x09, 0x21, 0xFB, 0x54, 0x44, 0x2D, 0x18]; // 3.14159265358979 (f64)
    //     let value: f64 = from_bytes(&data).unwrap();
    //     assert!((value - 3.14159265358979).abs() < f64::EPSILON);
    // }

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
}

impl<'de, 'a> serde::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = DeserializeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        println!("Deserializing any!");
        let field = self.parse_next_field()?;
        if field.is_null() {
            return visitor.visit_none();
        }
        println!("{:?}", field);
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
        let RionFieldType::Normal(NormalRionType::Bytes) = field.field_type() else {
            return Err(DeserializeError::InvalidData);
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
            Err(DeserializeError::InvalidData)
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
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
        println!("Deserializing seq!");
        let (lead, length, rest) =
            crate::get_normal_header(self.data).map_err(|e| e.to_string())?;
        let RionFieldType::Normal(NormalRionType::Array) = lead.field_type() else {
            return Err(DeserializeError::InvalidData);
        };
        self.data = rest;
        if self.data.len() < length {
            return Err(DeserializeError::InvalidData);
        }
        visitor.visit_seq(self)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
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
        println!("Deserializing map!");
        let (lead, length, rest) =
            crate::get_normal_header(self.data).map_err(|e| e.to_string())?;
        let RionFieldType::Normal(NormalRionType::Object) = lead.field_type() else {
            return Err(DeserializeError::InvalidData);
        };
        self.data = rest;
        if self.data.len() < length {
            return Err(DeserializeError::InvalidData);
        }
        visitor.visit_map(self)
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        println!("Deserializing struct!");
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
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
        println!("Deserializing identifier!");
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        println!("Ignoring any!");
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
