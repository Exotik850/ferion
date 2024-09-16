use std::fmt::Display;

use crate::{
    types::{NormalRionType, RionFieldType, ShortRionType},
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

// impl<T: Display> From<T> for DeserializeError {
//     fn from(err: T) -> Self {
//         DeserializeError::Custom(err.to_string())
//     }
// }

#[derive(Debug)]
pub enum DeserializeError {
    Eod,
    InvalidData,
    Custom(String),
}

struct Deserializer<'de> {
    data: &'de [u8],
}

impl<'a> Deserializer<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    fn peek_next(&self) -> Result<u8, DeserializeError> {
        self.data.first().copied().ok_or(DeserializeError::Eod)
    }

    fn next_byte(&mut self) -> Result<u8, DeserializeError> {
        let byte = self.data.first().copied().ok_or(DeserializeError::Eod)?;
        self.data = &self.data[1..];
        Ok(byte)
    }

    fn parse_next_field(&mut self) -> Result<RionField<'a>, DeserializeError> {
        let (field, rest) =
            RionField::parse(self.data).map_err(|_| DeserializeError::InvalidData)?;
        self.data = rest;
        Ok(field)
    }

    fn parse_field<T>(&mut self) -> Result<T, DeserializeError>
    where
        T: TryFrom<RionField<'a>, Error: Display>,
    {
        let field = self.parse_next_field()?;
        Ok(field
            .try_into()
            .map_err(|e: T::Error| DeserializeError::Custom(e.to_string()))?)
    }
}

#[cfg(test)]
mod tests {
    use super::Deserializer;
    #[test]
    fn test_deserialize_uint() {
        let data = vec![0x21, 0x0A]; // 10
        let mut deserializer = Deserializer::new(&data);
        let value: u64 = deserializer.parse_field().unwrap();
        assert_eq!(value, 10);
    }

    #[test]
    fn test_deserialize_string() {
        let data = vec![0xD1, 0x05, b'A', b'l', b'i', b'c', b'e'];
        let mut deserializer = Deserializer::new(&data);
        let name: String = deserializer.parse_field().unwrap();
        assert_eq!(name, "Alice");
    }
}

impl<'de, 'a> serde::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = DeserializeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let field = self.parse_next_field()?;
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
            _ => Err(DeserializeError::InvalidData),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_bool(self.parse_field()?)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_i8(self.parse_field()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_i16(self.parse_field()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_i32(self.parse_field()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_i64(self.parse_field()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_u8(self.parse_field()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_u16(self.parse_field()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_u32(self.parse_field()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_u64(self.parse_field()?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_f32(self.parse_field()?)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_f64(self.parse_field()?)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_char(self.parse_field()?)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let field = self.parse_next_field()?;
        match field.as_str() {
            Some(s) => visitor.visit_str(s),
            None => Err(DeserializeError::InvalidData),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_string(self.parse_field()?)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let field = self.parse_next_field()?;
        let RionFieldType::Normal(NormalRionType::Bytes) = field.field_type() else {
            return Err(DeserializeError::InvalidData);
        };
        visitor.visit_bytes(field.as_bytes())
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
        let field = self.parse_next_field()?;
        if field.is_null() {
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
        visitor.visit_seq(self)
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
        visitor.visit_seq(self)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
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
        if self.data[0] & 0xF0 != 0xE0 | 0xD0 {
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
